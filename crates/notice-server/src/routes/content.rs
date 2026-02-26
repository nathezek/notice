use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use notice_core::types::{SubmitUrlRequest, SubmitUrlResponse};
use notice_search::MeiliDocumentInput;

use crate::error::ApiError;
use crate::middleware::OptionalAuthUser;
use crate::state::AppState;

// ─── Helper: sync a document to Meilisearch ───

fn doc_row_to_meili(doc: &notice_db::documents::DocumentRow) -> MeiliDocumentInput {
    MeiliDocumentInput {
        id: doc.id,
        url: doc.url.clone(),
        domain: doc.domain.clone(),
        title: doc.title.clone(),
        raw_content: doc.raw_content.clone(),
        summary: doc.summary.clone(),
        status: doc.status.clone(),
    }
}

async fn sync_to_meilisearch(state: &AppState, doc: &notice_db::documents::DocumentRow) {
    let meili_doc = doc_row_to_meili(doc);
    match state.search.add_document(meili_doc).await {
        Ok(()) => tracing::info!(doc_id = %doc.id, "Document synced to Meilisearch"),
        Err(e) => tracing::error!(
            doc_id = %doc.id,
            error = %e,
            "Failed to sync document to Meilisearch"
        ),
    }
}

// ─── Submit URL to crawl queue ───

/// POST /api/submit
pub async fn submit_url(
    State(state): State<AppState>,
    auth: OptionalAuthUser,
    Json(body): Json<SubmitUrlRequest>,
) -> Result<Json<SubmitUrlResponse>, ApiError> {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(notice_core::Error::Validation("URL cannot be empty".into()).into());
    }

    url::Url::parse(&url)
        .map_err(|e| notice_core::Error::Validation(format!("Invalid URL: {}", e)))?;

    if notice_db::documents::get_by_url(&state.db, &url)
        .await?
        .is_some()
    {
        return Ok(Json(SubmitUrlResponse {
            id: Uuid::nil(),
            url: url.clone(),
            status: "exists".to_string(),
            message: "This URL has already been indexed".to_string(),
        }));
    }

    // Use authenticated user_id if available
    let entry = notice_db::crawl_queue::enqueue(&state.db, &url, 0, auth.user_id()).await?;

    match entry {
        Some(row) => {
            tracing::info!(
                url = %url,
                submitted_by = ?auth.user_id(),
                "URL enqueued for crawling"
            );
            Ok(Json(SubmitUrlResponse {
                id: row.id,
                url: row.url,
                status: "queued".to_string(),
                message: "URL has been added to the crawl queue".to_string(),
            }))
        }
        None => Ok(Json(SubmitUrlResponse {
            id: Uuid::nil(),
            url,
            status: "already_queued".to_string(),
            message: "This URL is already in the crawl queue".to_string(),
        })),
    }
}

// ─── Immediate crawl ───

/// POST /api/crawl — Immediately scrape, summarize, store, and index a URL.
pub async fn crawl_url(
    State(state): State<AppState>,
    Json(body): Json<SubmitUrlRequest>,
) -> Result<Json<notice_db::documents::DocumentRow>, ApiError> {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(notice_core::Error::Validation("URL cannot be empty".into()).into());
    }

    url::Url::parse(&url)
        .map_err(|e| notice_core::Error::Validation(format!("Invalid URL: {}", e)))?;

    if let Some(existing) = notice_db::documents::get_by_url(&state.db, &url).await? {
        return Ok(Json(existing));
    }

    tracing::info!(url = %url, "Starting immediate crawl");

    // Build a one-off HTTP client with proper User-Agent
    let client = reqwest::Client::builder()
        .user_agent("NoticeBot/0.1 (+https://github.com/notice-search; notice-search-engine)")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| notice_core::Error::Crawler(e.to_string()))?;

    // Scrape
    let page = notice_crawler::scraper_engine::scrape_url(&client, &url, 5_242_880).await?;

    // Store
    let doc = notice_db::documents::insert(
        &state.db,
        &page.url,
        page.title.as_deref(),
        &page.text_content,
    )
    .await?;

    tracing::info!(doc_id = %doc.id, "Document stored in PostgreSQL");

    // Summarize
    let content_for_summary = notice_core::truncate_utf8(&page.text_content, 8000).to_string();

    let doc = match state.gemini.summarize(&content_for_summary).await {
        Ok(summary) if !summary.is_empty() => {
            tracing::info!(doc_id = %doc.id, "Summary generated");
            notice_db::documents::update_summary(&state.db, doc.id, &summary).await?
        }
        Ok(_) => {
            tracing::warn!(doc_id = %doc.id, "Gemini returned empty summary");
            notice_db::documents::mark_summary_failed(&state.db, doc.id).await?;
            notice_db::documents::get_by_id(&state.db, doc.id)
                .await?
                .unwrap_or(doc)
        }
        Err(e) => {
            tracing::warn!(doc_id = %doc.id, error = %e, "Summarization failed");
            notice_db::documents::mark_summary_failed(&state.db, doc.id).await?;
            notice_db::documents::get_by_id(&state.db, doc.id)
                .await?
                .unwrap_or(doc)
        }
    };

    // Sync to Meilisearch
    sync_to_meilisearch(&state, &doc).await;

    Ok(Json(doc))
}

// ─── Crawler status ───

/// GET /api/crawler/status
pub async fn crawler_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let queue_stats = notice_db::crawl_queue::stats(&state.db).await?;
    let meili_count = state.search.document_count().await.unwrap_or(0);

    let crawler_guard = state.crawler.read().await;
    let crawler_stats = crawler_guard.as_ref().map(|h| h.get_stats());

    Ok(Json(serde_json::json!({
        "crawler": crawler_stats,
        "queue": queue_stats,
        "meilisearch_documents": meili_count
    })))
}

/// POST /api/crawler/stop
pub async fn crawler_stop(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let crawler_guard = state.crawler.read().await;
    if let Some(handle) = crawler_guard.as_ref() {
        handle.stop();
        Ok(Json(serde_json::json!({
            "message": "Crawler stop signal sent"
        })))
    } else {
        Ok(Json(serde_json::json!({
            "message": "Crawler is not running"
        })))
    }
}

// ─── Resync ───

/// POST /api/admin/resync
pub async fn resync_to_meilisearch(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("Starting full resync to Meilisearch");

    let total = notice_db::documents::count(&state.db).await?;

    let batch_size: i64 = 100;
    let mut offset: i64 = 0;
    let mut synced: i64 = 0;
    let mut failed: i64 = 0;

    loop {
        let docs = notice_db::documents::list_full(&state.db, batch_size, offset).await?;

        if docs.is_empty() {
            break;
        }

        let meili_docs: Vec<MeiliDocumentInput> = docs.iter().map(doc_row_to_meili).collect();

        let count = meili_docs.len() as i64;

        match state.search.add_documents(&meili_docs).await {
            Ok(()) => {
                synced += count;
                tracing::info!(
                    "Synced batch: {} documents (total: {}/{})",
                    count,
                    synced,
                    total
                );
            }
            Err(e) => {
                failed += count;
                tracing::error!(error = %e, "Failed to sync batch at offset {}", offset);
            }
        }

        offset += batch_size;
        if count < batch_size {
            break;
        }
    }

    let meili_count = state.search.document_count().await.unwrap_or(0);

    Ok(Json(serde_json::json!({
        "synced": synced,
        "failed": failed,
        "total_in_postgres": total,
        "total_in_meilisearch": meili_count
    })))
}

// ─── Document listing ───

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/documents
pub async fn list_documents(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let docs = notice_db::documents::list(&state.db, limit, offset).await?;
    let total = notice_db::documents::count(&state.db).await?;

    Ok(Json(serde_json::json!({
        "documents": docs,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

/// GET /api/documents/{id}
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<notice_db::documents::DocumentRow>, ApiError> {
    let doc = notice_db::documents::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| notice_core::Error::NotFound(format!("Document {} not found", id)))?;

    Ok(Json(doc))
}

/// GET /api/queue/stats
pub async fn queue_stats(
    State(state): State<AppState>,
) -> Result<Json<notice_db::crawl_queue::QueueStats>, ApiError> {
    let stats = notice_db::crawl_queue::stats(&state.db).await?;
    Ok(Json(stats))
}
