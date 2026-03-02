use chrono::Utc;
use notice_core::config::CrawlerConfig;
use notice_core::types::ScrapedPage;
use reqwest::Client;
use std::time::Duration;

/// Build a reusable HTTP client with proper headers.
pub fn build_http_client(config: &CrawlerConfig) -> Result<Client, notice_core::Error> {
    Client::builder()
        .user_agent(&config.user_agent)
        .timeout(Duration::from_secs(config.request_timeout_secs))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| notice_core::Error::Crawler(format!("Failed to build HTTP client: {}", e)))
}

/// Scrape a URL and extract its text content.
/// Returns the extracted text and the raw HTML (for link extraction).
pub async fn scrape_url(
    client: &Client,
    target_url: &str,
    max_size: usize,
) -> Result<ScrapedPage, notice_core::Error> {
    tracing::debug!(url = target_url, "Fetching URL");

    let response = client.get(target_url).send().await.map_err(|e| {
        notice_core::Error::Crawler(format!("Request failed for {}: {}", target_url, e))
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(notice_core::Error::Crawler(format!(
            "HTTP {} for {}",
            status, target_url
        )));
    }

    // Check content type
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("text/html") && !content_type.is_empty() {
        return Err(notice_core::Error::Crawler(format!(
            "Non-HTML content type '{}' for {}",
            content_type, target_url
        )));
    }

    // Check content length
    if let Some(len) = response.content_length() {
        if len as usize > max_size {
            return Err(notice_core::Error::Crawler(format!(
                "Content too large ({} bytes) for {}",
                len, target_url
            )));
        }
    }

    let html = response.text().await.map_err(|e| {
        notice_core::Error::Crawler(format!("Failed to read body for {}: {}", target_url, e))
    })?;

    if html.len() > max_size {
        return Err(notice_core::Error::Crawler(format!(
            "Content too large ({} bytes) for {}",
            html.len(),
            target_url
        )));
    }

    // Parse
    let document = scraper::Html::parse_document(&html);

    // Title
    let title_selector = scraper::Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty());

    // ── Boileplate Stripping ──
    // We define "noise" elements that often contain navigation, ads, or secondary content.
    let noise_selectors = [
        "nav", "header", "footer", "aside", "script", "style", "noscript",
        ".nav", ".navbar", ".menu", ".footer", ".header", ".sidebar", ".aside",
        ".ad", ".ads", ".advertisement", ".cookie", ".popup", ".modal",
        "#nav", "#navbar", "#menu", "#footer", "#header", "#sidebar", "#aside",
    ];

    let mut noise_nodes = std::collections::HashSet::new();
    for selector_str in noise_selectors {
        if let Ok(selector) = scraper::Selector::parse(selector_str) {
            for element in document.select(&selector) {
                // Store the node ID (POD) to avoid borrowing the selector
                noise_nodes.insert(element.id());
            }
        }
    }

    // Select content elements
    let content_selector = scraper::Selector::parse(
        "p, h1, h2, h3, h4, h5, h6, li, article, td, th, blockquote, pre, code, figcaption",
    ).unwrap();

    let mut text_parts = Vec::new();
    for element in document.select(&content_selector) {
        // Skip if this element or any of its ancestors is a noise node
        let mut is_noise = false;
        let mut current = Some(element);
        while let Some(node) = current {
            if noise_nodes.contains(&node.id()) {
                is_noise = true;
                break;
            }
            current = node.parent().and_then(scraper::ElementRef::wrap);
        }

        if !is_noise {
            let text = element.text().collect::<String>();
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                text_parts.push(trimmed);
            }
        }
    }

    let text_content = text_parts.join("\n");
    let text_content = text_content.trim().to_string();

    if text_content.is_empty() {
        return Err(notice_core::Error::Crawler(format!(
            "No text content extracted from {}",
            target_url
        )));
    }

    tracing::debug!(
        url = target_url,
        title = ?title,
        content_len = text_content.len(),
        "Page scraped"
    );

    Ok(ScrapedPage {
        url: target_url.to_string(),
        title,
        text_content,
        raw_html: html,
        scraped_at: Utc::now(),
    })
}
