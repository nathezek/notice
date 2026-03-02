use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

/// Per-domain rate limiter. Ensures we wait at least `delay` between
/// requests to the same domain.
pub struct DomainRateLimiter {
    delay: Duration,
    last_request: Mutex<HashMap<String, Instant>>,
}

impl DomainRateLimiter {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay: Duration::from_millis(delay_ms),
            last_request: Mutex::new(HashMap::new()),
        }
    }

    /// Wait until it's safe to make a request to this domain.
    /// Returns immediately if enough time has passed.
    pub async fn wait_for_domain(&self, domain: &str) {
        let mut map = self.last_request.lock().await;
        let now = Instant::now();

        if let Some(last) = map.get(domain) {
            let elapsed = now.duration_since(*last);
            if elapsed < self.delay {
                let wait = self.delay - elapsed;
                drop(map); // Release lock during sleep
                tokio::time::sleep(wait).await;
                let mut map = self.last_request.lock().await;
                map.insert(domain.to_string(), Instant::now());
                return;
            }
        }

        map.insert(domain.to_string(), now);
    }
}
