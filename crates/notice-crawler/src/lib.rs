use chrono::Utc;
use notice_core::types::ScrapedPage;
use scraper::{Html, Selector};

/// Scrape a URL and extract its text content.
pub async fn scrape_url(target_url: &str) -> Result<ScrapedPage, notice_core::Error> {
    tracing::info!(url = target_url, "Scraping URL");

    // Fetch the page
    let response = reqwest::get(target_url)
        .await
        .map_err(|e| notice_core::Error::Crawler(e.to_string()))?;

    if !response.status().is_success() {
        return Err(notice_core::Error::Crawler(format!(
            "HTTP {} for {}",
            response.status(),
            target_url
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| notice_core::Error::Crawler(e.to_string()))?;

    // Parse HTML
    let document = Html::parse_document(&html);

    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    // Extract text from <p>, <h1>-<h6>, <li>, <article> tags
    let content_selector =
        Selector::parse("p, h1, h2, h3, h4, h5, h6, li, article, td, th, blockquote")
            .unwrap();

    let text_content: String = document
        .select(&content_selector)
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    let text_content = text_content.trim().to_string();

    if text_content.is_empty() {
        return Err(notice_core::Error::Crawler(format!(
            "No text content extracted from {}",
            target_url
        )));
    }

    tracing::info!(
        url = target_url,
        title = ?title,
        content_length = text_content.len(),
        "Page scraped successfully"
    );

    Ok(ScrapedPage {
        url: target_url.to_string(),
        title,
        text_content,
        scraped_at: Utc::now(),
    })
}
