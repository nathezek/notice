use axum::Json;
use serde_json::{Value, json};

/// GET /health
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "notice",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
