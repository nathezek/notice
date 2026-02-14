mod calculator;
mod gemini;

use crate::gemini::ask_gemini;
use axum::extract::State;
use axum::{Json, Router, routing::post};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
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

#[derive(Clone)]
struct AppState {
    api_key: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let state = AppState { api_key };

    let cors = CorsLayer::new()
        // .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()) // strictly allow only frontend
        .allow_origin(Any) // For development, we can allow anyone
        .allow_methods(Any) // Allow GET, POST, etc.
        .allow_headers(Any); // Allow Content-Type, etc.

    let app = Router::new()
        .route("/search", post(handle_search))
        .layer(cors) // <--- Add the layer here!
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("🚀 Search Engine running on port 4000");
    axum::serve(listener, app).await.unwrap();
}

async fn handle_search(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Json<SearchResponse> {
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
    let gemini_response = ask_gemini(query, &state.api_key).await;

    Json(SearchResponse {
        result_type: "concept".to_string(),
        content: gemini_response,
    })
}
