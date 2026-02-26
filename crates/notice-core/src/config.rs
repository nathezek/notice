/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,

    // PostgreSQL
    pub database_url: String,

    // Meilisearch
    pub meili_url: String,
    pub meili_api_key: String,

    // Gemini
    pub gemini_api_key: String,

    // Auth
    pub jwt_secret: String,

    // Crawler
    pub crawler: CrawlerConfig,
}

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    /// Number of concurrent crawler workers
    pub workers: usize,
    /// Minimum delay between requests to the same domain (milliseconds)
    pub politeness_delay_ms: u64,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum content size to download (bytes)
    pub max_content_size: usize,
    /// User-Agent string
    pub user_agent: String,
    /// Whether to discover and enqueue links from crawled pages
    pub discover_links: bool,
    /// Maximum depth for link discovery (0 = don't follow links from discovered pages)
    pub max_link_depth: u32,
    /// Whether the crawler is enabled
    pub enabled: bool,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Panics if required variables are missing — fail fast at startup.
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .expect("PORT must be a valid u16"),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            meili_url: std::env::var("MEILI_URL")
                .unwrap_or_else(|_| "http://localhost:7700".into()),
            meili_api_key: std::env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY must be set"),
            gemini_api_key: std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            crawler: CrawlerConfig {
                workers: std::env::var("CRAWLER_WORKERS")
                    .unwrap_or_else(|_| "2".into())
                    .parse()
                    .unwrap_or(2),
                politeness_delay_ms: std::env::var("CRAWLER_POLITENESS_MS")
                    .unwrap_or_else(|_| "1000".into())
                    .parse()
                    .unwrap_or(1000),
                request_timeout_secs: std::env::var("CRAWLER_TIMEOUT_SECS")
                    .unwrap_or_else(|_| "30".into())
                    .parse()
                    .unwrap_or(30),
                max_content_size: std::env::var("CRAWLER_MAX_SIZE_BYTES")
                    .unwrap_or_else(|_| "5242880".into()) // 5MB
                    .parse()
                    .unwrap_or(5_242_880),
                user_agent: std::env::var("CRAWLER_USER_AGENT").unwrap_or_else(|_| {
                    "NoticeBot/0.1 (+https://github.com/notice-search; notice-search-engine)".into()
                }),
                discover_links: std::env::var("CRAWLER_DISCOVER_LINKS")
                    .unwrap_or_else(|_| "false".into())
                    .parse()
                    .unwrap_or(false),
                max_link_depth: std::env::var("CRAWLER_MAX_LINK_DEPTH")
                    .unwrap_or_else(|_| "1".into())
                    .parse()
                    .unwrap_or(1),
                enabled: std::env::var("CRAWLER_ENABLED")
                    .unwrap_or_else(|_| "true".into())
                    .parse()
                    .unwrap_or(true),
            },
        }
    }
}
