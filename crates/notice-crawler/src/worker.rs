use crate::links;
use crate::rate_limiter::DomainRateLimiter;
use crate::robots::RobotsChecker;
use crate::scraper_engine;

use notice_core::config::CrawlerConfig;
use notice_search::MeiliDocumentInput;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

/// Handle to control the running crawler.
#[derive(Clone)]
pub struct CrawlerHandle {
    cancel: CancellationToken,
    stats: Arc<CrawlerStats>,
}

/// Runtime statistics for the crawler.
pub struct CrawlerStats {
    pub pages_crawled: AtomicU64,
    pub pages_failed: AtomicU64,
    pub links_discovered: AtomicU64,
    pub running: AtomicBool,
}

impl CrawlerHandle {
    /// Stop the crawler gracefully.
    pub fn stop(&self) {
        tracing::info!("Crawler stop requested");
        self.cancel.cancel();
    }

    /// Check if the crawler is running.
    pub fn is_running(&self) -> bool {
        self.stats.running.load(Ordering::Relaxed)
    }

    /// Get crawler statistics.
    pub fn get_stats(&self) -> CrawlerStatsSnapshot {
        CrawlerStatsSnapshot {
            pages_crawled: self.stats.pages_crawled.load(Ordering::Relaxed),
            pages_failed: self.stats.pages_failed.load(Ordering::Relaxed),
            links_discovered: self.stats.links_discovered.load(Ordering::Relaxed),
            running: self.stats.running.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CrawlerStatsSnapshot {
    pub pages_crawled: u64,
    pub pages_failed: u64,
    pub links_discovered: u64,
    pub running: bool,
}

/// Shared context passed to each worker.
struct CrawlerContext {
    db: PgPool,
    search: notice_search::SearchClient,
    gemini: notice_ai::GeminiClient,
    http: Client,
    robots: RobotsChecker,
    rate_limiter: DomainRateLimiter,
    config: CrawlerConfig,
    stats: Arc<CrawlerStats>,
    cancel: CancellationToken,
}

/// Start the background crawler. Returns a handle for control.
pub fn start_crawler(
    db: PgPool,
    search: notice_search::SearchClient,
    gemini: notice_ai::GeminiClient,
    config: CrawlerConfig,
) -> CrawlerHandle {
    let cancel = CancellationToken::new();

    let stats = Arc::new(CrawlerStats {
        pages_crawled: AtomicU64::new(0),
        pages_failed: AtomicU64::new(0),
        links_discovered: AtomicU64::new(0),
        running: AtomicBool::new(true),
    });

    let handle = CrawlerHandle {
        cancel: cancel.clone(),
        stats: stats.clone(),
    };

    let http =
        scraper_engine::build_http_client(&config).expect("Failed to build crawler HTTP client");

    let robots = RobotsChecker::new(http.clone(), &config.user_agent);
    let rate_limiter = DomainRateLimiter::new(config.politeness_delay_ms);

    let ctx = Arc::new(CrawlerContext {
        db,
        search,
        gemini,
        http,
        robots,
        rate_limiter,
        config: config.clone(),
        stats,
        cancel,
    });

    // Spawn worker tasks
    let num_workers = config.workers.max(1);
    tracing::info!("Starting {} crawler worker(s)", num_workers);

    for worker_id in 0..num_workers {
        let ctx = Arc::clone(&ctx);
        tokio::spawn(async move {
            worker_loop(worker_id, ctx).await;
        });
    }

    // Spawn the startup cleanup task
    let ctx_cleanup = Arc::clone(&ctx);
    tokio::spawn(async move {
        if let Ok(reset) = notice_db::crawl_queue::reset_stale(&ctx_cleanup.db).await {
            if reset > 0 {
                tracing::info!("Reset {} stale in_progress crawl queue items", reset);
            }
        }
    });

    handle
}

/// Main loop for a single crawler worker.
async fn worker_loop(worker_id: usize, ctx: Arc<CrawlerContext>) {
    tracing::info!(worker = worker_id, "Crawler worker started");

    loop {
        // Check for cancellation
        if ctx.cancel.is_cancelled() {
            tracing::info!(worker = worker_id, "Crawler worker shutting down");
            break;
        }

        // Try to dequeue a URL
        let item = match notice_db::crawl_queue::dequeue_next(&ctx.db).await {
            Ok(Some(item)) => item,
            Ok(None) => {
                // Queue is empty — wait and retry
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                    _ = ctx.cancel.cancelled() => {
                        tracing::info!(worker = worker_id, "Crawler worker shutting down (idle)");
                        break;
                    }
                }
                continue;
            }
            Err(e) => {
                tracing::error!(worker = worker_id, error = %e, "Failed to dequeue");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        tracing::info!(worker = worker_id, url = %item.url, "Processing URL");

        // Process the URL
        match process_url(&ctx, &item.url).await {
            Ok(discovered) => {
                // Mark completed
                if let Err(e) = notice_db::crawl_queue::mark_completed(&ctx.db, item.id).await {
                    tracing::error!(error = %e, "Failed to mark completed");
                }
                ctx.stats.pages_crawled.fetch_add(1, Ordering::Relaxed);

                // Enqueue discovered links
                if ctx.config.discover_links && !discovered.is_empty() {
                    let new_count = enqueue_discovered_links(&ctx.db, &discovered).await;
                    ctx.stats
                        .links_discovered
                        .fetch_add(new_count, Ordering::Relaxed);
                }
            }
            Err(e) => {
                tracing::warn!(
                    worker = worker_id,
                    url = %item.url,
                    error = %e,
                    retry = item.retry_count + 1,
                    max = item.max_retries,
                    "Crawl failed"
                );
                if let Err(mark_err) =
                    notice_db::crawl_queue::mark_failed(&ctx.db, item.id, &e.to_string()).await
                {
                    tracing::error!(error = %mark_err, "Failed to mark failed");
                }
                ctx.stats.pages_failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    ctx.stats.running.store(false, Ordering::Relaxed);
    tracing::info!(worker = worker_id, "Crawler worker stopped");
}

/// Process a single URL: robots check → rate limit → scrape → store → summarize → index → discover links.
async fn process_url(
    ctx: &CrawlerContext,
    target_url: &str,
) -> Result<Vec<String>, notice_core::Error> {
    // Step 1: Check robots.txt
    if !ctx.robots.is_allowed(target_url).await {
        return Err(notice_core::Error::Crawler(format!(
            "Blocked by robots.txt: {}",
            target_url
        )));
    }

    // Step 2: Rate limit per domain
    let domain = url::Url::parse(target_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default();

    ctx.rate_limiter.wait_for_domain(&domain).await;

    // Step 3: Check if already indexed
    if notice_db::documents::get_by_url(&ctx.db, target_url)
        .await?
        .is_some()
    {
        tracing::debug!(url = target_url, "Already indexed, skipping");
        return Ok(vec![]);
    }

    // Step 4: Scrape
    let page =
        scraper_engine::scrape_url(&ctx.http, target_url, ctx.config.max_content_size).await?;

    // Step 5: Extract links from raw HTML (single fetch, no double request)
    let discovered_links = if ctx.config.discover_links {
        links::extract_links(&page.raw_html, target_url)
    } else {
        vec![]
    };

    // Step 6: Store in PostgreSQL
    let doc = notice_db::documents::insert(
        &ctx.db,
        &page.url,
        page.title.as_deref(),
        &page.text_content,
    )
    .await?;

    tracing::info!(doc_id = %doc.id, url = %target_url, "Document stored");

    // Step 7: Summarize with Gemini
    let content_for_summary = notice_core::truncate_utf8(&page.text_content, 8000).to_string();

    let doc = match ctx.gemini.summarize(&content_for_summary).await {
        Ok(summary) if !summary.is_empty() => {
            tracing::debug!(doc_id = %doc.id, "Summary generated");
            notice_db::documents::update_summary(&ctx.db, doc.id, &summary).await?
        }
        Ok(_) => {
            tracing::debug!(doc_id = %doc.id, "Empty summary from Gemini");
            notice_db::documents::mark_summary_failed(&ctx.db, doc.id).await?;
            doc
        }
        Err(e) => {
            tracing::warn!(doc_id = %doc.id, error = %e, "Summarization failed");
            notice_db::documents::mark_summary_failed(&ctx.db, doc.id).await?;
            doc
        }
    };

    // Step 8: Sync to Meilisearch
    let meili_doc = MeiliDocumentInput {
        id: doc.id,
        url: doc.url.clone(),
        domain: doc.domain.clone(),
        title: doc.title.clone(),
        raw_content: doc.raw_content.clone(),
        summary: doc.summary.clone(),
        status: doc.status.clone(),
    };

    if let Err(e) = ctx.search.add_document(meili_doc).await {
        tracing::error!(doc_id = %doc.id, error = %e, "Failed to index in Meilisearch");
    }

    Ok(discovered_links)
}

/// Filter and enqueue newly discovered links.
async fn enqueue_discovered_links(db: &PgPool, links: &[String]) -> u64 {
    if links.is_empty() {
        return 0;
    }

    // Filter out already-known URLs
    let mut new_urls: Vec<String> = vec![];
    for link in links {
        match notice_db::crawl_queue::url_is_known(db, link).await {
            Ok(false) => new_urls.push(link.clone()),
            Ok(true) => {}
            Err(e) => {
                tracing::debug!(error = %e, url = %link, "Error checking URL");
            }
        }
    }

    if new_urls.is_empty() {
        return 0;
    }

    let count = new_urls.len();

    match notice_db::crawl_queue::enqueue_batch(db, &new_urls, -1).await {
        Ok(inserted) => {
            if inserted > 0 {
                tracing::info!("Discovered and enqueued {} new URLs", inserted);
            }
            inserted
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to enqueue discovered links");
            0
        }
    }
}
