use scraper::{Html, Selector};

const MAX_CONTEXT_CHARS: usize = 1500; // Per page limit
const MAX_PAGES: usize = 2;

// ---- DuckDuckGo HTML Search ----

pub async fn search(query: &str) -> Vec<String> {
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
        Err(e) => {
            println!("DDG search error: {}", e);
            return vec![];
        }
    };

    let document = Html::parse_document(&html);

    // DDG HTML page uses <a class="result__url"> for result links
    let link_sel = Selector::parse("a.result__url").unwrap();
    let mut urls: Vec<String> = document
        .select(&link_sel)
        .filter_map(|el| {
            let href = el.text().collect::<String>().trim().to_string();
            // DDG shows URLs as plain text in class="result__url"
            if href.starts_with("http") || href.contains('.') {
                // Normalise: ensure https:// prefix
                if href.starts_with("http") {
                    Some(href)
                } else {
                    Some(format!("https://{}", href))
                }
            } else {
                None
            }
        })
        .take(5)
        .collect();

    // Fallback: try <a.result__a> which has the href attribute
    if urls.is_empty() {
        let link_a = Selector::parse("a.result__a").unwrap();
        urls = document
            .select(&link_a)
            .filter_map(|el| el.value().attr("href").map(|h| h.to_string()))
            .filter(|h| h.starts_with("http"))
            .take(5)
            .collect();
    }

    println!("DDG found {} URLs for '{}'", urls.len(), query);
    urls
}

// ---- Page Scraper ----

pub async fn scrape(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .ok()?;

    let html = client.get(url).send().await.ok()?.text().await.ok()?;

    let document = Html::parse_document(&html);

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
        return None;
    }

    // Truncate to limit
    let truncated = if text.len() > MAX_CONTEXT_CHARS {
        format!("{}...", &text[..MAX_CONTEXT_CHARS])
    } else {
        text
    };

    Some(truncated)
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
        if let Ok(Some(content)) = task.await {
            context_parts.push(content);
        }
    }

    if context_parts.is_empty() {
        return (urls, String::new());
    }

    (urls, context_parts.join("\n\n---\n\n"))
}
