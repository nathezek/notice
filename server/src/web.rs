use scraper::{Html, Selector};
use tracing::{info, error, debug};

const MAX_CONTEXT_CHARS: usize = 1500; // Per page limit
const MAX_PAGES: usize = 2;

// ---- DuckDuckGo HTML Search ----

// ---- Mojeek Search (Very scraper friendly) ----

pub async fn mojeek_search(query: &str) -> Vec<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

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
    let link_sel = Selector::parse("a.ob").unwrap(); // Mojeek uses class 'ob' for result titles
    
    let mut urls: Vec<String> = document
        .select(&link_sel)
        .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
        .filter(|h| h.starts_with("http"))
        .take(5)
        .collect();

    if urls.is_empty() {
        // Fallback selector for Mojeek
        let link_sel_fallback = Selector::parse(".results-standard .title a").unwrap();
        urls = document
            .select(&link_sel_fallback)
            .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
            .filter(|h| h.starts_with("http"))
            .take(5)
            .collect();
    }

    info!("Mojeek found {} URLs for '{}'", urls.len(), query);
    urls
}

// ---- Google Search (Basic HTML Version) ----

pub async fn google_search(query: &str) -> Vec<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)") // Sometimes posing as a crawler helps, or just use standard
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    // gbv=1 is the key for basic HTML
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
            // Extract the actual URL from /url?q=https://...&sa=U&...
            let raw = h.trim_start_matches("/url?q=");
            let end = raw.find('&').unwrap_or(raw.len());
            let encoded_url = &raw[..end];
            urlencoding::decode(encoded_url).unwrap_or(encoded_url.into()).to_string()
        })
        .filter(|h| h.starts_with("http") && !h.contains("google.com/"))
        .take(5)
        .collect();

    info!("Google found {} URLs for '{}'", urls.len(), query);
    urls
}

pub async fn search(query: &str) -> Vec<String> {
    // Try Mojeek first (most reliable for simple scraping)
    let mut urls = mojeek_search(query).await;
    
    // If Mojeek fails or returns too few, try Google
    if urls.len() < 2 {
        let google_urls = google_search(query).await;
        for url in google_urls {
            if !urls.contains(&url) {
                urls.push(url);
            }
        }
    }
    
    // Last resort: Original DDG (might be blocked)
    if urls.is_empty() {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        let html = match client.get(&url).send().await {
            Ok(res) => res.text().await.unwrap_or_default(),
            Err(_) => String::new(),
        };

        let document = Html::parse_document(&html);
        let link_sel = Selector::parse("a.result__url").unwrap();
        let ddg_urls: Vec<String> = document
            .select(&link_sel)
            .filter_map(|el| {
                let href = el.text().collect::<String>().trim().to_string();
                if href.starts_with("http") || href.contains('.') {
                    if href.starts_with("http") { Some(href) } else { Some(format!("https://{}", href)) }
                } else { None }
            })
            .take(5)
            .collect();
        
        urls.extend(ddg_urls);
    }

    urls.truncate(8);
    urls
}

// ---- Page Scraper ----

pub async fn scrape(url: &str) -> (Option<String>, Option<String>) {
    info!("Starting scrape for URL: {}", url);
    let client = match reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(4))
        .build() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to build reqwest client: {}", e);
                return (None, None);
            }
        };

    let html = match client.get(url).send().await {
        Ok(res) => match res.text().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to read text from {}: {}", url, e);
                return (None, None);
            }
        },
        Err(e) => {
            error!("Failed to fetch URL {}: {}", url, e);
            return (None, None);
        }
    };

    let document = Html::parse_document(&html);

    // Extract Title
    let title_sel = Selector::parse("title").unwrap();
    let title = document.select(&title_sel).next().map(|el| el.text().collect::<String>().trim().to_string());
    
    debug!("Extracted title {:?} for URL {}", title, url);

    // Remove script, style, nav, footer, header noise
    let content_sel = Selector::parse(
        "p, article, main, section, h1, h2, h3, h4, li",
    ).unwrap();

    let text: String = document
        .select(&content_sel)
        .map(|el| {
            let t = el.text().collect::<Vec<_>>().join(" ");
            t.trim().to_string()
        })
        .filter(|t| t.len() > 30) // skip tiny fragments
        .collect::<Vec<_>>()
        .join("\n");

    if text.is_empty() {
        info!("No significant text found for URL: {}", url);
        return (title, None);
    }

    // Truncate to limit
    let truncated = if text.len() > MAX_CONTEXT_CHARS {
        format!("{}...", &text[..MAX_CONTEXT_CHARS])
    } else {
        text
    };

    info!("Successfully scraped {} bytes from URL: {}", truncated.len(), url);
    (title, Some(truncated))
}

// ---- Context Builder ----

pub async fn gather_context(query: &str) -> (Vec<String>, String) {
    let urls = search(query).await;
    if urls.is_empty() {
        return (vec![], String::new());
    }

    // Clone urls for scraping tasks
    let scrape_urls = urls.clone();

    let tasks: Vec<_> = scrape_urls
        .into_iter()
        .take(MAX_PAGES)
        .map(|url| tokio::spawn(async move { scrape(&url).await }))
        .collect();

    let mut context_parts: Vec<String> = Vec::new();
    for task in tasks {
        if let Ok((_title, Some(content))) = task.await {
            context_parts.push(content);
        }
    }

    if context_parts.is_empty() {
        return (urls, String::new());
    }

    (urls, context_parts.join("\n\n---\n\n"))
}
