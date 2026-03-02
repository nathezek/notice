use scraper::{Html, Selector};
use std::collections::HashSet;

/// Extract links from HTML content and resolve them to absolute URLs.
/// Only returns links on the SAME DOMAIN as the base URL.
/// Filters out non-HTTP(S), anchors, assets, and common non-content paths.
pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let base = match url::Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return vec![],
    };

    let base_domain = match base.host_str() {
        Some(h) => h.to_string(),
        None => return vec![],
    };

    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a[href]").unwrap();

    let mut links: Vec<String> = vec![];
    let mut seen = HashSet::new();

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

        // ── SAME DOMAIN ONLY ──
        // This prevents crawling every language version of Wikipedia,
        // external sites linked from articles, etc.
        let link_domain = match absolute.host_str() {
            Some(h) => h.to_string(),
            None => continue,
        };

        if link_domain != base_domain {
            continue;
        }

        // Remove fragment
        let mut clean = absolute;
        clean.set_fragment(None);
        let url_str = clean.to_string();

        // Skip non-content paths
        if should_skip_url(&url_str) {
            continue;
        }

        // Skip Wikipedia special pages, talk pages, user pages, etc.
        if is_wikipedia_noise(&url_str) {
            continue;
        }

        // Dedup
        if seen.insert(url_str.clone()) {
            links.push(url_str);
        }
    }

    links
}

/// URLs we should never crawl (assets, auth pages, etc.)
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
        ".woff",
        ".woff2",
        ".ttf",
        ".eot",
    ];

    let lower = url.to_lowercase();
    skip_patterns.iter().any(|p| lower.contains(p))
}

/// Filter out Wikipedia-specific noise pages.
/// These are pages that aren't actual content articles.
fn is_wikipedia_noise(url: &str) -> bool {
    let noise_patterns = [
        "/wiki/Special:",
        "/wiki/Talk:",
        "/wiki/User:",
        "/wiki/User_talk:",
        "/wiki/Wikipedia:",
        "/wiki/Wikipedia_talk:",
        "/wiki/File:",
        "/wiki/File_talk:",
        "/wiki/MediaWiki:",
        "/wiki/Template:",
        "/wiki/Template_talk:",
        "/wiki/Help:",
        "/wiki/Help_talk:",
        "/wiki/Category:",
        "/wiki/Category_talk:",
        "/wiki/Portal:",
        "/wiki/Portal_talk:",
        "/wiki/Draft:",
        "/wiki/Module:",
        "/w/index.php",
        "/wiki/Main_Page",
        "action=edit",
        "action=history",
        "oldid=",
        "printable=yes",
        "#cite",
        "#References",
    ];

    noise_patterns.iter().any(|p| url.contains(p))
}
