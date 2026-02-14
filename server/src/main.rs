mod calculator;

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
struct SearchResponse {
    result_type: String,
    content: String,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        // .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()) // strictly allow only frontend
        .allow_origin(Any) // For development, we can allow anyone
        .allow_methods(Any) // Allow GET, POST, etc.
        .allow_headers(Any); // Allow Content-Type, etc.

    let app = Router::new()
        .route("/search", post(handle_search))
        .layer(cors); // <--- Add the layer here!

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("🚀 Search Engine running on port 4000");
    axum::serve(listener, app).await.unwrap();
}

async fn handle_search(Json(payload): Json<SearchRequest>) -> Json<SearchResponse> {
    let query = &payload.query;

    // 1. Try Conversion Path
    if let Some((amount, from, to)) = calculator::extract_conversion(query) {
        let result = calculator::calculate(amount, from, to);
        return Json(SearchResponse {
            result_type: "conversion".to_string(),
            content: format!("{:.2} {} is {:.2} {}", amount, from, result, to),
        });
    }

    // 2. Default to Concept Path (Placeholder for Gemini)
    Json(SearchResponse {
        result_type: "concept".to_string(),
        content: "I'll need to ask Gemini about that...".to_string(),
    })
}
