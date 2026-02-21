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
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error, debug};
use tracing_subscriber;

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

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    info!("Starting Notice Search Engine...");

    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // Initialize DB
    info!("Connecting to PostgreSQL...");
    let db_pool = match db::init_db(&database_url).await {
        Ok(pool) => {
            info!("Successfully connected to PostgreSQL.");
            pool
        }
        Err(e) => {
            error!("Failed to connect to PostgreSQL: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize Meilisearch
    info!("Connecting to Meilisearch...");
    let meili_client = indexer::init_indexer("http://localhost:7700", None).await;

    let state = AppState { 
        api_key,
        db_pool,
        meili_client
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap())
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

async fn handle_search_web(
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
                Err(_) => fallback_to_gemini(query, &state, None, None, vec![]).await,
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
                None => fallback_to_gemini(query, &state, None, None, vec![]).await,
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
                    fallback_to_gemini(query, &state, None, None, vec![]).await
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
            fallback_to_gemini(query, &state, None, None, vec![]).await
        }

        // ---- General: search index first, then spell-correct/scrape/Gemini ----
        classifier::QueryType::General => {
            info!("General query detected. Checking Meilisearch index for '{}'", query);
            
            // Try Meilisearch first
            match indexer::search_index(&state.meili_client, query).await {
                Ok(hits) if !hits.is_empty() => {
                    let hit = &hits[0];
                    info!("Index HIT found for '{}': {}", query, hit.url);
                    
                    return Json(SearchResponse {
                        result_type: "concept".to_string(),
                        content: serde_json::json!({
                            "title": hit.title,
                            "summary": hit.summary.clone().unwrap_or_default(),
                            "facts": [],
                            "related_topics": [],
                            "websites": [{ "url": hit.url, "title": hit.title }]
                        }).to_string(),
                        corrected_query: None,
                    });
                }
                Ok(_) => debug!("Index MISS for '{}'", query),
                Err(e) => error!("Meilisearch query error: {}", e),
            }

            let corrected = spell::correct_query(query);
            let effective = corrected.clone().unwrap_or_default();
            let effective_str = if effective.is_empty() { query } else { &effective };

            // For the fast path, we just search DuckDuckGo and return the URLs.
            // We don't call Gemini yet.
            let urls = web::search(effective_str).await;
            
            Json(SearchResponse {
                result_type: "concept".to_string(),
                content: serde_json::json!({
                    "title": effective_str,
                    "summary": "",
                    "facts": [],
                    "related_topics": [],
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
    let query = &payload.query;
    let urls = payload.urls;

    // Fast path for summary: if we have URLs, scrape them concurrently and ask Gemini
    let (found_urls, context) = if urls.is_empty() {
        web::gather_context(query).await
    } else {
        // Scrape the provided URLs
        let scrape_urls = urls.clone();
        let tasks: Vec<_> = scrape_urls
            .into_iter()
            .take(2) // MAX_PAGES
            .map(|url| tokio::spawn(async move { web::scrape(&url).await }))
            .collect();

        let mut context_parts: Vec<String> = Vec::new();
        for task in tasks {
            if let Ok((_title, Some(content))) = task.await {
                context_parts.push(content);
            }
        }
        (urls, context_parts.join("\n\n---\n\n"))
    };

    let context_ref = if context.is_empty() { None } else { Some(context.as_str()) };
    fallback_to_gemini(query, &state, None, context_ref, found_urls).await
}

async fn fallback_to_gemini(
    query: &str, 
    state: &AppState, 
    corrected_query: Option<String>, 
    context: Option<&str>, 
    urls: Vec<String>
) -> Json<SearchResponse> {
    let response = gemini::ask_gemini(query, &state.api_key, context, urls.clone()).await;
    
    // Background task to cache results if we got a valid response and have URLs
    if !urls.is_empty() && !response.contains("\"error\"") {
        let state_clone = state.clone();
        let response_clone = response.clone();
        
        tokio::spawn(async move {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_clone) {
                let title = json["title"].as_str().unwrap_or("Search Result").to_string();
                let summary = json["summary"].as_str().unwrap_or_default().to_string();
                
                for url in json["websites"].as_array().unwrap_or(&vec![]) {
                    if let Some(url_str) = url["url"].as_str() {
                        let (scraped_title, scraped_text) = web::scrape(url_str).await;
                        let final_title = scraped_title.unwrap_or_else(|| title.clone());
                        let final_text = scraped_text.unwrap_or_default();

                        // 1. Vault (Postgres)
                        let page_data = db::PageData {
                            url: url_str.to_string(),
                            title: final_title.clone(),
                            raw_html: "OMITTED".to_string(),
                            cleaned_text: final_text.clone(),
                            summary: Some(summary.clone()),
                            crawled_at: chrono::Utc::now().naive_utc(),
                        };
                        let _ = db::insert_page(&state_clone.db_pool, &page_data).await;

                        // 2. Index (Meilisearch)
                        use sha2::{Sha256, Digest};
                        let mut hasher = Sha256::new();
                        hasher.update(url_str.as_bytes());
                        let id = hex::encode(hasher.finalize());

                        let doc = indexer::IndexDocument {
                            id,
                            url: url_str.to_string(),
                            title: final_title,
                            cleaned_text: final_text,
                            summary: Some(summary.clone()),
                        };
                        let _ = indexer::index_page(&state_clone.meili_client, &doc).await;
                        info!("Auto-cached URL: {}", url_str);
                    }
                }
            }
        });
    }

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

#[derive(Deserialize)]
struct IndexUrlRequest {
    url: String,
}

#[derive(Serialize)]
struct IndexUrlResponse {
    success: bool,
    message: String,
}

// Endpoint specifically to test the indexing pipeline on a single URL
async fn handle_index_url(
    State(state): State<AppState>,
    Json(payload): Json<IndexUrlRequest>,
) -> Json<IndexUrlResponse> {
    let url = &payload.url;
    info!("Received manual request to index URL: {}", url);

    // 1. Scrape the URL
    info!("Step 1: Scraping...");
    let (title_opt, text_opt) = web::scrape(url).await;
    
    let title = title_opt.unwrap_or_else(|| "Unknown Title".to_string());
    let clean_text = match text_opt {
        Some(t) => t,
        None => {
            return Json(IndexUrlResponse {
                success: false,
                message: "Failed to extract meaningful text from the page.".to_string(),
            });
        }
    };

    // 2. Summarize via Gemini
    info!("Step 2: Summarizing via Gemini...");
    let summary = gemini::summarize_page(&state.api_key, &title, &clean_text).await;

    // 3. Save to PostgreSQL (Vault)
    info!("Step 3: Saving to PostgreSQL...");
    let page_data = db::PageData {
        url: url.to_string(),
        title: title.clone(),
        raw_html: "OMITTED_FOR_API_TEST".to_string(), // In production we'd save raw HTML here
        cleaned_text: clean_text.clone(),
        summary: Some(summary.clone()),
        crawled_at: chrono::Utc::now().naive_utc(),
    };

    if let Err(e) = db::insert_page(&state.db_pool, &page_data).await {
        error!("Failed to save to DB: {}", e);
        return Json(IndexUrlResponse {
            success: false,
            message: format!("Failed to save to database: {}", e),
        });
    }

    // 4. Send to Meilisearch (Index)
    info!("Step 4: Indexing in Meilisearch...");
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let id = hex::encode(hasher.finalize()); // Meilisearch ID must be safe chars

    let index_doc = indexer::IndexDocument {
        id,
        url: url.to_string(),
        title,
        cleaned_text: clean_text,
        summary: Some(summary),
    };

    if let Err(e) = indexer::index_page(&state.meili_client, &index_doc).await {
        error!("Failed to index in Meilisearch: {}", e);
        return Json(IndexUrlResponse {
            success: false,
            message: format!("Failed to index in Meilisearch: {}", e),
        });
    }

    info!("Successfully processed and indexed {}", url);
    Json(IndexUrlResponse {
        success: true,
        message: "Successfully scraped, summarized, saved, and indexed.".to_string(),
    })
}
