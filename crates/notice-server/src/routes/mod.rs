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
        // ── Public (no auth) ──
        .route("/health", get(health::health_check))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        // ── Optional auth (works anonymous, personalized when logged in) ──
        .route("/api/search", get(search::search))
        .route("/api/submit", post(content::submit_url))
        .route("/api/crawl", post(content::crawl_url))
        .route("/api/documents", get(content::list_documents))
        .route("/api/documents/{id}", get(content::get_document))
        .route("/api/queue/stats", get(content::queue_stats))
        .route("/api/crawler/status", get(content::crawler_status))
        .route("/api/crawler/stop", post(content::crawler_stop))
        // ── Required auth (must be logged in) ──
        .route("/api/auth/me", get(auth::me))
        .route("/api/me/kg", get(kg::get_my_kg))
        .route("/api/me/kg/context", get(kg::get_my_context))
        // ── Admin ──
        .route("/api/admin/resync", post(content::resync_to_meilisearch))
        // State
        .with_state(state)
}
