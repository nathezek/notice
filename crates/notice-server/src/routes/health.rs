use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};

use crate::state::AppState;

/// GET /health — checks all dependencies
pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    let meili_ok = state.search.health().await.is_ok();

    let status = if db_ok && meili_ok { "ok" } else { "degraded" };

    Json(json!({
        "status": status,
        "service": "notice",
        "version": env!("CARGO_PKG_VERSION"),
        "dependencies": {
            "postgres": if db_ok { "up" } else { "down" },
            "meilisearch": if meili_ok { "up" } else { "down" }
        }
    }))
}
