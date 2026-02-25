use scraper::{Html, Selector};

/// Extract links from HTML content and resolve them to absolute URLs.
/// Filters out non-HTTP(S) links, anchors, and common non-content paths.
pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let base = match url::Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };

    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a[href]").unwrap();

    let mut links: Vec<String> = vec![];
    let mut seen = std::collections::HashSet::new();

    for element in document.select(&link_selector) {
        let href = match element.value().attr("href") {
            Some(h) => h.trim(),
            None => continue,
        };

        // Skip empty, anchors, javascript, mailto
        if href.is_empty()
            || href.starts_with('#')
            || href.starts_with("javascript:")
            || href.starts_with("mailto:")
            || href.starts_with("tel:")
        {
            continue;
        }

        // Resolve relative URLs
        let absolute = match base.join(href) {
            Ok(u) => u,
            Err(_) => continue,
        };

        // Only HTTP(S)
        if absolute.scheme() != "http" && absolute.scheme() != "https" {
            continue;
        }

        // Remove fragment
        let mut clean = absolute.clone();
        clean.set_fragment(None);
        let url_str = clean.to_string();

        // Skip common non-content paths
        if should_skip_url(&url_str) {
            continue;
        }

        // Dedup
        if seen.insert(url_str.clone()) {
            links.push(url_str);
        }
    }

    links
}

/// Returns true for URLs we should skip (login pages, assets, etc.)
fn should_skip_url(url: &str) -> bool {
    let skip_patterns = [
        "/login",
        "/signup",
        "/register",
        "/logout",
        "/admin",
        "/api/",
        "/feed",
        "/rss",
        ".pdf",
        ".jpg",
        ".jpeg",
        ".png",
        ".gif",
        ".svg",
        ".css",
        ".js",
        ".zip",
        ".tar",
        ".gz",
        ".mp3",
        ".mp4",
        ".avi",
        ".exe",
        ".dmg",
        ".iso",
        ".xml",
        ".json",
    ];

    let lower = url.to_lowercase();
    skip_patterns.iter().any(|p| lower.contains(p))
}
