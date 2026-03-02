use reqwest::Client;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// In-memory robots.txt cache. Stores parsed rules per domain.
pub struct RobotsChecker {
    client: Client,
    /// domain → list of disallowed path prefixes
    cache: RwLock<HashMap<String, Vec<String>>>,
    user_agent: String,
}

impl RobotsChecker {
    pub fn new(client: Client, user_agent: &str) -> Self {
        Self {
            client,
            cache: RwLock::new(HashMap::new()),
            user_agent: user_agent.to_string(),
        }
    }

    /// Check if crawling the given URL is allowed by robots.txt.
    pub async fn is_allowed(&self, url: &str) -> bool {
        let parsed = match url::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let domain = match parsed.host_str() {
            Some(h) => h.to_string(),
            None => return false,
        };

        let path = parsed.path().to_string();

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(disallowed) = cache.get(&domain) {
                return !disallowed.iter().any(|prefix| path.starts_with(prefix));
            }
        }

        // Fetch and parse robots.txt
        let robots_url = format!("{}://{}/robots.txt", parsed.scheme(), domain);
        let disallowed = self.fetch_and_parse(&robots_url).await;

        let allowed = !disallowed.iter().any(|prefix| path.starts_with(prefix));

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(domain, disallowed);
        }

        allowed
    }

    /// Fetch robots.txt and extract Disallow rules for our user agent.
    async fn fetch_and_parse(&self, robots_url: &str) -> Vec<String> {
        let response = match self.client.get(robots_url).send().await {
            Ok(r) if r.status().is_success() => r,
            _ => {
                tracing::debug!(url = robots_url, "No robots.txt found — allowing all");
                return vec![];
            }
        };

        let body = match response.text().await {
            Ok(b) => b,
            Err(_) => return vec![],
        };

        self.parse_robots_txt(&body)
    }

    /// Parse robots.txt content. Returns disallowed path prefixes.
    /// Checks for our specific user agent first, falls back to *.
    fn parse_robots_txt(&self, content: &str) -> Vec<String> {
        let mut disallowed_star: Vec<String> = vec![];
        let mut disallowed_specific: Vec<String> = vec![];
        let mut current_agent = CurrentAgent::None;

        let our_agent_lower = self
            .user_agent
            .split('/')
            .next()
            .unwrap_or("")
            .to_lowercase();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = match line.split_once(':') {
                Some((k, v)) => (k.trim().to_lowercase(), v.trim().to_string()),
                None => continue,
            };

            match key.as_str() {
                "user-agent" => {
                    let agent = value.to_lowercase();
                    if agent == "*" {
                        current_agent = CurrentAgent::Star;
                    } else if !our_agent_lower.is_empty() && agent.contains(&our_agent_lower) {
                        current_agent = CurrentAgent::Specific;
                    } else {
                        current_agent = CurrentAgent::Other;
                    }
                }
                "disallow" if !value.is_empty() => match current_agent {
                    CurrentAgent::Star => disallowed_star.push(value),
                    CurrentAgent::Specific => disallowed_specific.push(value),
                    _ => {}
                },
                _ => {}
            }
        }

        // Prefer specific rules over * rules
        if !disallowed_specific.is_empty() {
            disallowed_specific
        } else {
            disallowed_star
        }
    }
}

#[derive(Clone, Copy)]
enum CurrentAgent {
    None,
    Star,
    Specific,
    Other,
}
