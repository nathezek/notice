use notice_ai::GeminiClient;
use notice_crawler::CrawlerHandle;
use notice_search::SearchClient;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state, injected into every request handler.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub search: SearchClient,
    pub gemini: GeminiClient,
    pub jwt_secret: String,
    /// Optional crawler handle. None if crawler is disabled.
    pub crawler: Arc<RwLock<Option<CrawlerHandle>>>,
}
