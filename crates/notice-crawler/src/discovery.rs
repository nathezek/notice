use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{debug, error, info};

/// Search discovery using Mojeek and Google (V1 restoration).
/// Returns a list of URLs found for the given query.
pub async fn find_urls(query: &str) -> Vec<String> {
    let mut urls = mojeek_search(query).await;

    // If Mojeek fails or returns too few, try Google
    if urls.len() < 3 {
        let google_urls = google_search(query).await;
        for url in google_urls {
            if !urls.contains(&url) {
                urls.push(url);
            }
        }
    }

    urls.truncate(10);
    urls
}

async fn meine_search_client() -> Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

pub async fn mojeek_search(query: &str) -> Vec<String> {
    let client = meine_search_client().await;
    let url = format!(
        "https://www.mojeek.com/search?q={}",
        urlencoding::encode(query)
    );

    let html = match client.get(&url).send().await {
        Ok(res) => res.text().await.unwrap_or_default(),
        Err(e) => {
            error!("Mojeek search error: {}", e);
            return vec![];
        }
    };

    let document = Html::parse_document(&html);
    let link_sel = Selector::parse("a.ob").unwrap(); // Mojeek result titles
    
    let mut urls: Vec<String> = document
        .select(&link_sel)
        .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
        .filter(|h| h.starts_with("http"))
        .take(10)
        .collect();

    if urls.is_empty() {
        // Fallback selector
        let link_sel_fallback = Selector::parse(".results-standard .title a").unwrap();
        urls = document
            .select(&link_sel_fallback)
            .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
            .filter(|h| h.starts_with("http"))
            .take(10)
            .collect();
    }

    info!("Mojeek found {} URLs for '{}'", urls.len(), query);
    urls
}

pub async fn google_search(query: &str) -> Vec<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // gbv=1 is for basic HTML results
    let url = format!(
        "https://www.google.com/search?q={}&gbv=1",
        urlencoding::encode(query)
    );

    let html = match client.get(&url).send().await {
        Ok(res) => res.text().await.unwrap_or_default(),
        Err(e) => {
            error!("Google search error: {}", e);
            return vec![];
        }
    };

    let document = Html::parse_document(&html);
    let link_sel = Selector::parse("a").unwrap();
    
    let urls: Vec<String> = document
        .select(&link_sel)
        .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
        .filter(|h| h.starts_with("/url?q="))
        .map(|h| {
            // Extract actual URL from /url?q=...
            let raw = h.trim_start_matches("/url?q=");
            let end = raw.find('&').unwrap_or(raw.len());
            let encoded_url = &raw[..end];
            urlencoding::decode(encoded_url).unwrap_or(encoded_url.into()).to_string()
        })
        .filter(|h| h.starts_with("http") && !h.contains("google.com/"))
        .take(10)
        .collect();

    info!("Google found {} URLs for '{}'", urls.len(), query);
    urls
}
