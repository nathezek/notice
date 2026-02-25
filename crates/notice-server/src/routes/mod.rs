pub mod auth;
pub mod content;
pub mod health;
pub mod kg;
pub mod search;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health::health_check))
        // Search
        .route("/api/search", get(search::search))
        // Auth
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        // Content
        .route("/api/submit", post(content::submit_url))
        .route("/api/crawl", post(content::crawl_url))
        .route("/api/documents", get(content::list_documents))
        .route("/api/documents/{id}", get(content::get_document))
        .route("/api/queue/stats", get(content::queue_stats))
        // Crawler control
        .route("/api/crawler/status", get(content::crawler_status))
        .route("/api/crawler/stop", post(content::crawler_stop))
        // Knowledge Graph
        .route("/api/users/{user_id}/kg", get(kg::get_user_kg))
        .route("/api/users/{user_id}/kg/context", get(kg::get_user_context))
        // Admin
        .route("/api/admin/resync", post(content::resync_to_meilisearch))
        // State
        .with_state(state)
}
