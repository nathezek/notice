use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use notice_classifier::QueryIntent;
use notice_core::types::{InstantAnswer, SearchResponse};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub session_id: Option<String>,
    // user_id will come from JWT middleware later
}

/// GET /api/search?q=your+query
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    let query = params.q.trim().to_string();
    if query.is_empty() {
        return Err(notice_core::Error::Validation("Query cannot be empty".into()).into());
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    tracing::info!(query = %query, "Search request");

    // Step 1: Classify intent
    let intent = notice_classifier::classify(&query, &state.gemini).await;

    let response = match intent {
        QueryIntent::Calculate(expr) => {
            record_search(&state, &query, "calculate", 0, params.session_id.as_deref()).await;

            SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "calculation".to_string(),
                    value: evaluate_math(&expr),
                }),
            }
        }

        QueryIntent::Define(term) => {
            record_search(&state, &query, "define", 0, params.session_id.as_deref()).await;

            SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "definition".to_string(),
                    value: format!("Definition lookup for '{}' — coming soon", term),
                }),
            }
        }

        QueryIntent::Timer(command) => {
            record_search(&state, &query, "timer", 0, params.session_id.as_deref()).await;

            SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "timer".to_string(),
                    value: format!("Timer from '{}' — coming soon", command),
                }),
            }
        }

        QueryIntent::Search(search_query) => {
            // Step 2: Search Meilisearch
            let (results, total) = state
                .search
                .search(&search_query, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Meilisearch query failed");
                    (vec![], 0)
                });

            let results_count = results.len() as i32;
            record_search(
                &state,
                &search_query,
                "search",
                results_count,
                params.session_id.as_deref(),
            )
            .await;

            SearchResponse {
                query: search_query,
                results,
                total,
                instant_answer: None,
            }
        }
    };

    Ok(Json(response))
}

/// Record a search in history (fire-and-forget — don't fail the request).
async fn record_search(
    state: &AppState,
    query: &str,
    intent: &str,
    results_count: i32,
    session_id: Option<&str>,
) {
    // TODO: extract user_id from JWT when auth middleware is added
    let user_id: Option<uuid::Uuid> = None;

    if let Err(e) = notice_db::search_history::record(
        &state.db,
        user_id,
        session_id,
        query,
        intent,
        results_count,
    )
    .await
    {
        tracing::warn!(error = %e, "Failed to record search history");
    }
}

/// Basic math expression evaluator.
/// Handles simple arithmetic. We can improve this later.
fn evaluate_math(expr: &str) -> String {
    // Simple evaluation using Rust's built-in parsing
    // For now, handle basic single-operation expressions
    let expr = expr.replace(' ', "");

    // Try parsing as a simple f64 first
    if let Ok(n) = expr.parse::<f64>() {
        return format_number(n);
    }

    // Try to evaluate using a very basic approach
    // This handles: "150*6+7" style expressions
    // For production, use a proper math parser crate like `meval`
    match eval_simple(&expr) {
        Some(result) => format_number(result),
        None => format!("Cannot evaluate: {}", expr),
    }
}

fn eval_simple(expr: &str) -> Option<f64> {
    // Split by + and evaluate each term
    let mut total = 0.0;
    for term in expr.split('+') {
        let term = term.trim();
        if term.contains('*') {
            let parts: Vec<&str> = term.split('*').collect();
            let mut product = 1.0;
            for p in parts {
                product *= p.trim().parse::<f64>().ok()?;
            }
            total += product;
        } else if term.contains('/') {
            let parts: Vec<&str> = term.splitn(2, '/').collect();
            let a = parts[0].trim().parse::<f64>().ok()?;
            let b = parts[1].trim().parse::<f64>().ok()?;
            if b == 0.0 {
                return None;
            }
            total += a / b;
        } else if term.contains('-') && !term.starts_with('-') {
            let parts: Vec<&str> = term.splitn(2, '-').collect();
            let a = parts[0].trim().parse::<f64>().ok()?;
            let b = parts[1].trim().parse::<f64>().ok()?;
            total += a - b;
        } else {
            total += term.parse::<f64>().ok()?;
        }
    }
    Some(total)
}

fn format_number(n: f64) -> String {
    if n == n.floor() {
        format!("{}", n as i64)
    } else {
        format!("{:.6}", n)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
