mod calculator;
mod classifier;
mod currency;
mod gemini;
mod spell;
mod web;

use axum::{Json, Router, extract::State, routing::post};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Deserialize)]
struct CalculateRequest {
    expression: String,
}

#[derive(Serialize)]
struct SearchResponse {
    result_type: String,
    content: String,
    corrected_query: Option<String>,
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
        .route("/calculate", post(handle_calculate))
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

    // Classify on the RAW query first — structured queries (math/units/currency)
    // must never be passed through the English spell corrector
    match classifier::classify(query) {
        // ---- Math: evaluate expression directly ----
        classifier::QueryType::Math => {
            let bare_math_re = regex::Regex::new(r"^\s*(?i)(calculator|calc)\s*$").unwrap();
            if bare_math_re.is_match(query) {
                return Json(SearchResponse {
                    result_type: "math".to_string(),
                    content: serde_json::json!({
                        "expression": "0",
                        "result": "0"
                    }).to_string(),
                    corrected_query: None,
                });
            }

            match calculator::eval_math(query) {
                Ok(result) => Json(SearchResponse {
                    result_type: "math".to_string(),
                    content: serde_json::json!({
                        "expression": query,
                        "result": result
                    }).to_string(),
                    corrected_query: None,
                }),
                Err(_) => fallback_to_gemini(query, &state.api_key, None, None, vec![]).await,
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
                        corrected_query: None,
                    })
                }
                None => fallback_to_gemini(query, &state.api_key, None, None, vec![]).await,
            }
        }

        // ---- Currency conversion ----
        classifier::QueryType::CurrencyConversion => {
            let bare_currency_re = regex::Regex::new(r"^\s*(?i)(converter|currency converter|exchange rates|exchange rate)\s*$").unwrap();
            let actual_query = if bare_currency_re.is_match(query) {
                "1 USD to EUR"
            } else {
                query
            };

            match currency::convert_currency(actual_query).await {
                Ok(r) => Json(SearchResponse {
                    result_type: "currency_conversion".to_string(),
                    content: serde_json::json!({
                        "amount": r.amount,
                        "from": r.from,
                        "to": r.to,
                        "result": format!("{:.2}", r.result),
                        "rate": format!("{:.6}", r.rate)
                    }).to_string(),
                    corrected_query: None,
                }),
                Err(e) => {
                    println!("Currency error: {}", e);
                    fallback_to_gemini(query, &state.api_key, None, None, vec![]).await
                }
            }
        }

        // ---- Timer ----
        classifier::QueryType::Timer => {
            let bare_timer_re = regex::Regex::new(r"^\s*(?i)(timer|stopwatch)\s*$").unwrap();
            if bare_timer_re.is_match(query) {
                return Json(SearchResponse {
                    result_type: "timer".to_string(),
                    content: serde_json::json!({
                        "seconds": 300,
                        "query": "5 Minute Timer"
                    }).to_string(),
                    corrected_query: None,
                });
            }

            let re = regex::Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(s|sec|secs|second|seconds|m|min|mins|minute|minutes|h|hr|hrs|hour|hours)").unwrap();
            let mut total_seconds: f64 = 0.0;
            for cap in re.captures_iter(query) {
                let val: f64 = cap[1].parse().unwrap_or(0.0);
                let unit = &cap[2].to_lowercase();
                if unit.starts_with('h') {
                    total_seconds += val * 3600.0;
                } else if unit.starts_with('m') {
                    total_seconds += val * 60.0;
                } else {
                    total_seconds += val;
                }
            }
            if total_seconds > 0.0 {
                let content = serde_json::json!({
                    "seconds": total_seconds as u64,
                    "query": query.trim()
                }).to_string();
                return Json(SearchResponse {
                    result_type: "timer".to_string(),
                    content,
                    corrected_query: None,
                });
            }
            fallback_to_gemini(query, &state.api_key, None, None, vec![]).await
        }

        // ---- General: spell-correct, scrape, then Gemini ----
        classifier::QueryType::General => {
            let corrected = spell::correct_query(query);
            let effective = corrected.clone().unwrap_or_default();
            let effective = if effective.is_empty() { query } else { &effective };

            // Scrape web content concurrently with building the response
            let (urls, context) = web::gather_context(effective).await;
            let context_ref = if context.is_empty() { None } else { Some(context.as_str()) };

            fallback_to_gemini(effective, &state.api_key, corrected, context_ref, urls).await
        }
    }
}

async fn fallback_to_gemini(query: &str, api_key: &str, corrected_query: Option<String>, context: Option<&str>, urls: Vec<String>) -> Json<SearchResponse> {
    let response = gemini::ask_gemini(query, api_key, context, urls).await;
    Json(SearchResponse {
        result_type: "concept".to_string(),
        content: response,
        corrected_query,
    })
}

#[derive(Serialize)]
struct CalculateResponse {
    result: String,
    error: Option<String>,
}

async fn handle_calculate(
    Json(payload): Json<CalculateRequest>,
) -> Json<CalculateResponse> {
    match calculator::eval_math(&payload.expression) {
        Ok(res) => Json(CalculateResponse { result: res, error: None }),
        Err(e) => Json(CalculateResponse { result: "".to_string(), error: Some(e) })
    }
}
