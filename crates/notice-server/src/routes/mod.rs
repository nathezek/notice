pub mod auth;
pub mod health;
pub mod search;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Search
        .route("/api/search", get(search::search))
        // Auth
        .route("/api/auth/register", axum::routing::post(auth::register))
        .route("/api/auth/login", axum::routing::post(auth::login))
        // Attach state
        .with_state(state)
}
