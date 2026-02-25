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

    // Text content
    let content_selector = scraper::Selector::parse(
        "p, h1, h2, h3, h4, h5, h6, li, article, td, th, blockquote, pre, code, figcaption",
    )
    .unwrap();

    let text_content: String = document
        .select(&content_selector)
        .map(|el| el.text().collect::<String>())
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

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
