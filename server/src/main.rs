mod calculator;
mod classifier;
mod currency;
mod gemini;

use axum::{Json, Router, extract::State, routing::post};
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
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/search", post(handle_search))
        .layer(cors)
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

    match classifier::classify(query) {
        // ---- Math: evaluate expression directly ----
        classifier::QueryType::Math => {
            match calculator::eval_math(query) {
                Ok(result) => Json(SearchResponse {
                    result_type: "math".to_string(),
                    content: serde_json::json!({
                        "expression": query,
                        "result": result
                    }).to_string(),
                }),
                Err(_) => fallback_to_gemini(query, &state.api_key).await,
            }
        }

        // ---- Unit conversion ----
        classifier::QueryType::UnitConversion => {
            match calculator::convert_unit(query) {
                Some(r) => {
                    let result_str = if r.result == r.result.floor() {
                        format!("{}", r.result as i64)
                    } else {
                        format!("{:.4}", r.result).trim_end_matches('0').trim_end_matches('.').to_string()
                    };
                    Json(SearchResponse {
                        result_type: "unit_conversion".to_string(),
                        content: serde_json::json!({
                            "amount": r.amount,
                            "from": r.from,
                            "to": r.to,
                            "result": result_str,
                            "category": r.category
                        }).to_string(),
                    })
                }
                None => fallback_to_gemini(query, &state.api_key).await,
            }
        }

        // ---- Currency conversion ----
        classifier::QueryType::CurrencyConversion => {
            match currency::convert_currency(query).await {
                Ok(r) => Json(SearchResponse {
                    result_type: "currency_conversion".to_string(),
                    content: serde_json::json!({
                        "amount": r.amount,
                        "from": r.from,
                        "to": r.to,
                        "result": format!("{:.2}", r.result),
                        "rate": format!("{:.6}", r.rate)
                    }).to_string(),
                }),
                Err(e) => {
                    println!("Currency error: {}", e);
                    fallback_to_gemini(query, &state.api_key).await
                }
            }
        }

        // ---- General: Gemini ----
        classifier::QueryType::General => {
            fallback_to_gemini(query, &state.api_key).await
        }
    }
}

async fn fallback_to_gemini(query: &str, api_key: &str) -> Json<SearchResponse> {
    let response = gemini::ask_gemini(query, api_key).await;
    Json(SearchResponse {
        result_type: "concept".to_string(),
        content: response,
    })
}
