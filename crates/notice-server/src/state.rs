use notice_ai::GeminiClient;
use notice_search::SearchClient;
use sqlx::PgPool;

/// Shared application state, injected into every request handler.
/// All fields are cheaply cloneable (Arc'd internally).
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub search: SearchClient,
    pub gemini: GeminiClient,
    pub jwt_secret: String,
}
