mod calculator;
mod classifier;
mod currency;
mod db;
mod gemini;
mod indexer;
mod spell;
mod web;

use axum::{Json, Router, extract::State, routing::post};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber; // Required for Identity Matching

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Deserialize)]
struct SearchSummaryRequest {
    query: String,
    urls: Vec<String>,
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
    db_pool: db::DbPool,
    meili_client: meilisearch_sdk::client::Client,
}

#[derive(Serialize)]
struct CalculateResponse {
    result: String,
    error: Option<String>,
}

#[derive(Deserialize)]
struct IndexUrlRequest {
    url: String,
}

#[derive(Serialize)]
struct IndexUrlResponse {
    success: bool,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("Starting Notice Search Engine...");

    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Connecting to PostgreSQL...");
    let db_pool = db::init_db(&database_url)
        .await
        .expect("Failed to connect to PG");

    info!("Connecting to Meilisearch...");
    let meili_client = indexer::init_indexer("http://localhost:7700", None).await;

    let state = AppState {
        api_key,
        db_pool,
        meili_client,
    };

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/search/web", post(handle_search_web))
        .route("/search/summary", post(handle_search_summary))
        .route("/calculate", post(handle_calculate))
        .route("/index-url", post(handle_index_url))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("🚀 Search Engine running on port 4000");
    axum::serve(listener, app).await.unwrap();
}

// ==========================================
// SEARCH HANDLERS
// ==========================================

async fn handle_search_web(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Json<SearchResponse> {
    let query = payload.query.trim();
    if query.is_empty() {
        return Json(SearchResponse {
            result_type: "error".into(),
            content: "Query cannot be empty".into(),
            corrected_query: None,
        });
    }

    match classifier::classify(query) {
        classifier::QueryType::Math => {
            match calculator::eval_math(query) {
                Ok(result) => Json(SearchResponse {
                    result_type: "math".to_string(),
                    content: serde_json::json!({ "expression": query, "result": result }).to_string(),
                    corrected_query: None,
                }),
                Err(_) => fallback_to_gemini(query, &state, None, None, vec![]).await,
            }
        }
        classifier::QueryType::UnitConversion => {
            match calculator::convert_unit(query) {
                Some(r) => Json(SearchResponse {
                    result_type: "unit_conversion".to_string(),
                    content: serde_json::json!({
                        "amount": r.amount, "from": r.from, "to": r.to, "result": r.result, "category": r.category
                    }).to_string(),
                    corrected_query: None,
                }),
                None => fallback_to_gemini(query, &state, None, None, vec![]).await,
            }
        }
        classifier::QueryType::CurrencyConversion => {
            match currency::convert_currency(query).await {
                Ok(r) => Json(SearchResponse {
                    result_type: "currency_conversion".to_string(),
                    content: serde_json::json!({
                        "amount": r.amount, "from": r.from, "to": r.to, "result": r.result, "rate": r.rate
                    }).to_string(),
                    corrected_query: None,
                }),
                Err(_) => fallback_to_gemini(query, &state, None, None, vec![]).await,
            }
        }
        classifier::QueryType::Timer => {
            let bare_timer_re = regex::Regex::new(r"^\s*(?i)(timer|stopwatch)\s*$").unwrap();
                        if bare_timer_re.is_match(query) {

                            return Json(SearchResponse {
                                result_type: "timer".to_string(),
                                content: serde_json::json!({
                                    "seconds": 300,
                                    "query": "5 Minute Timer"
                                })
                                .to_string(),
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
                            })
                            .to_string();
                            return Json(SearchResponse {
                                result_type: "timer".to_string(),
                                content,
                                corrected_query: None,
                            });
                        }
            fallback_to_gemini(query, &state, None, None, vec![]).await
        }
        classifier::QueryType::General => {
            info!("General query check: '{}'", query);

            // 1. EXACT HASH PATH (No more "Wrong Response")
            let mut hasher = Sha256::new();
            hasher.update(query.as_bytes());
            let query_id = hex::encode(hasher.finalize());

            if let Some(hit) = indexer::get_document_by_id(&state.meili_client, &query_id).await {
                if let Some(summary) = hit.summary {
                    info!("Identity HIT for '{}'", query);
                    return Json(SearchResponse {
                        result_type: "concept".to_string(),
                        content: serde_json::json!({
                            "title": hit.title,
                            "summary": summary,
                            "facts": [], "related_topics": [],
                            "websites": [{ "url": hit.url, "title": hit.title }]
                        }).to_string(),
                        corrected_query: None,
                    });
                }
            }

            // 2. FUZZY FALLBACK (With Strict Title Check)
            if let Ok(hits) = indexer::search_index(&state.meili_client, query).await {
                if let Some(hit) = hits.first() {
                    let title_lower = hit.title.to_lowercase();
                    let query_lower = query.to_lowercase();
                    let is_relevant = query_lower.split_whitespace()
                        .filter(|w| w.len() > 3)
                        .any(|word| title_lower.contains(word));

                    if is_relevant {
                        return Json(SearchResponse {
                            result_type: "concept".to_string(),
                            content: serde_json::json!({
                                "title": hit.title,
                                "summary": hit.summary.clone().unwrap_or_default(),
                                "facts": [], "related_topics": [],
                                "websites": [{ "url": hit.url, "title": hit.title }]
                            }).to_string(),
                            corrected_query: None,
                        });
                    }
                }
            }

            // 3. LIVE SEARCH
            let corrected = spell::correct_query(query);
            let effective_str = corrected.as_deref().unwrap_or(query);
            let urls = web::search(effective_str).await;

            Json(SearchResponse {
                result_type: "concept".to_string(),
                content: serde_json::json!({
                    "title": effective_str,
                    "summary": "",
                    "facts": [], "related_topics": [],
                    "websites": urls.iter().map(|u| serde_json::json!({ "url": u, "title": u })).collect::<Vec<_>>()
                }).to_string(),
                corrected_query: corrected,
            })
        }
    }
}

async fn handle_search_summary(
    State(state): State<AppState>,
    Json(payload): Json<SearchSummaryRequest>,
) -> Json<SearchResponse> {
    let query = payload.query.trim();
    let urls = payload.urls;

    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    let query_id = hex::encode(hasher.finalize());

    if let Some(hit) = indexer::get_document_by_id(&state.meili_client, &query_id).await {
        if let Some(summary) = hit.summary {
            if !summary.is_empty() {
                return Json(SearchResponse {
                    result_type: "concept".to_string(),
                    content: serde_json::json!({
                        "title": hit.title,
                        "summary": summary,
                        "facts": [], "related_topics": [],
                        "websites": [{ "url": hit.url, "title": hit.title }]
                    })
                    .to_string(),
                    corrected_query: None,
                });
            }
        }
    }

    let (found_urls, context) = if urls.is_empty() {
        web::gather_context(query).await
    } else {
        let tasks: Vec<_> = urls
            .clone()
            .into_iter()
            .take(2)
            .map(|url| tokio::spawn(async move { web::scrape(&url).await }))
            .collect();
        let mut context_parts = Vec::new();
        for task in tasks {
            if let Ok((_, Some(content))) = task.await {
                context_parts.push(content);
            }
        }
        (urls, context_parts.join("\n\n---\n\n"))
    };

    fallback_to_gemini(query, &state, None, Some(&context), found_urls).await
}

async fn fallback_to_gemini(
    query: &str,
    state: &AppState,
    corrected_query: Option<String>,
    context: Option<&str>,
    urls: Vec<String>,
) -> Json<SearchResponse> {
    let response = gemini::ask_gemini(query, &state.api_key, context, urls.clone()).await;

    if !response.contains("\"error\"") {
        let state_clone = state.clone();
        let response_clone = response.clone();
        let query_clone = query.to_string();
        let urls_clone = urls.clone();

        tokio::spawn(async move {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_clone) {
                let title = json["title"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&query_clone)
                    .to_string();
                let summary = json["summary"].as_str().unwrap_or_default().to_string();

                let mut q_hasher = Sha256::new();
                q_hasher.update(query_clone.as_bytes());
                let query_id = hex::encode(q_hasher.finalize());
                let query_id_clone = query_id.clone();

                let query_doc = indexer::IndexDocument {
                    id: query_id,
                    url: urls_clone.first().cloned().unwrap_or_default(),
                    title: title.clone(),
                    cleaned_text: summary.clone(),
                    summary: Some(summary.clone()),
                };
                match indexer::index_page(&state_clone.meili_client, &query_doc).await {
                    Ok(_) => info!(
                        "Successfully indexed query ID '{}' for cached search",
                        query_id_clone
                    ),
                    Err(e) => error!(
                        "Failed to background index query ID '{}': {}",
                        query_id_clone, e
                    ),
                }
            }
        });
    }

    Json(SearchResponse {
        result_type: "concept".into(),
        content: response,
        corrected_query,
    })
}

// ==========================================
// CALCULATOR & UTILITY HANDLERS
// ==========================================

async fn handle_calculate(Json(payload): Json<CalculateRequest>) -> Json<CalculateResponse> {
    match calculator::eval_math(&payload.expression) {
        Ok(res) => Json(CalculateResponse {
            result: res,
            error: None,
        }),
        Err(e) => Json(CalculateResponse {
            result: "".into(),
            error: Some(e),
        }),
    }
}

async fn handle_index_url(
    State(state): State<AppState>,
    Json(payload): Json<IndexUrlRequest>,
) -> Json<IndexUrlResponse> {
    let url = &payload.url;
    let (title_opt, text_opt) = web::scrape(url).await;
    let title = title_opt.unwrap_or_else(|| "Unknown Title".to_string());

    let clean_text = match text_opt {
        Some(t) => t,
        None => {
            return Json(IndexUrlResponse {
                success: false,
                message: "Scrape failed".into(),
            });
        }
    };

    let summary = gemini::summarize_page(&state.api_key, &title, &clean_text).await;

    let page_data = db::PageData {
        url: url.clone(),
        title: title.clone(),
        raw_html: "OMITTED".into(),
        cleaned_text: clean_text.clone(),
        summary: Some(summary.clone()),
        crawled_at: chrono::Utc::now().naive_utc(),
    };

    let _ = db::insert_page(&state.db_pool, &page_data).await;

    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let id = hex::encode(hasher.finalize());

    let doc = indexer::IndexDocument {
        id,
        url: url.clone(),
        title,
        cleaned_text: clean_text,
        summary: Some(summary),
    };

    match indexer::index_page(&state.meili_client, &doc).await {
        Ok(_) => Json(IndexUrlResponse {
            success: true,
            message: "Indexed successfully".into(),
        }),
        Err(e) => Json(IndexUrlResponse {
            success: false,
            message: e.to_string(),
        }),
    }
}
