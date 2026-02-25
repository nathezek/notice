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
        }
    }
}
