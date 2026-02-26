use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use notice_classifier::QueryIntent;
use notice_core::types::{InstantAnswer, SearchResponse};

use crate::error::ApiError;
use crate::middleware::OptionalAuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub session_id: Option<String>,
}

/// GET /api/search?q=your+query
///
/// Pipeline:
/// 1. Classify intent (calculate / define / timer / search)
/// 2. If instant answer → return immediately
/// 3. If search → query Meilisearch directly (synonyms handle expansion)
/// 4. Record in search history
/// 5. Return results
pub async fn search(
    State(state): State<AppState>,
    auth: OptionalAuthUser,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    let query = params.q.trim().to_string();
    if query.is_empty() {
        return Err(notice_core::Error::Validation("Query cannot be empty".into()).into());
    }

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let user_id = auth.user_id();

    tracing::info!(
        query = %query,
        authenticated = user_id.is_some(),
        "Search request"
    );

    // Step 1: Classify intent
    let intent = notice_classifier::classify(&query);

    let response = match intent {
        QueryIntent::Calculate(expr) => {
            record_search(
                &state,
                &query,
                "calculate",
                0,
                params.session_id.as_deref(),
                user_id,
            )
            .await;

            SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "calculation".to_string(),
                    value: evaluate_math(&expr),
                }),
                ai_answer: None,
            }
        }

        QueryIntent::Define(term) => {
            record_search(
                &state,
                &query,
                "define",
                0,
                params.session_id.as_deref(),
                user_id,
            )
            .await;

            SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "definition".to_string(),
                    value: format!("Definition lookup for '{}' — coming soon", term),
                }),
                ai_answer: None,
            }
        }

        QueryIntent::Timer(command) => {
            record_search(
                &state,
                &query,
                "timer",
                0,
                params.session_id.as_deref(),
                user_id,
            )
            .await;

            SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "timer".to_string(),
                    value: format!("Timer from '{}' — coming soon", command),
                }),
                ai_answer: None,
            }
        }

        QueryIntent::Search(search_query) => {
            // Step 2: Search Meilisearch directly
            let (results, total) = state
                .search
                .search(&search_query, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Meilisearch query failed");
                    (vec![], 0)
                });

            let results_count = results.len() as i32;

            // Step 3: On-demand discovery (Triggered if results are insufficient)
            if results_count < 3 {
                tracing::info!(query = %search_query, "Insufficient results, triggering discovery");
                let db = state.db.clone();
                let discovery_query = search_query.clone();
                
                // We spawn discovery in the background to not block the current search response,
                // BUT the next search (or a refresh) will have these new results.
                tokio::spawn(async move {
                    let discovered_urls = notice_crawler::discovery::find_urls(&discovery_query).await;
                    for url in discovered_urls {
                        if let Err(e) = notice_db::crawl_queue::enqueue(&db, &url, 10, None).await {
                            tracing::warn!(url = %url, error = %e, "Failed to enqueue discovered URL");
                        }
                    }
                });
            }

            // Step 4: Optional RAG Answer
            // If we have results, generate a synthesized answer using the top document snippets
            let ai_answer = if !results.is_empty() {
                let contexts: Vec<String> = results
                    .iter()
                    .take(5) // Use top 5 results for context
                    .map(|r| {
                        format!(
                            "Title: {}\nURL: {}\nSnippet: {}",
                            r.title.as_deref().unwrap_or("Untitled"),
                            r.url,
                            r.snippet
                        )
                    })
                    .collect();

                match state.gemini.answer_query(&search_query, &contexts).await {
                    Ok(answer) => Some(answer),
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to generate AI RAG answer");
                        None
                    }
                }
            } else {
                None
            };

            // Step 4: Record in search history
            record_search(
                &state,
                &search_query,
                "search",
                results_count,
                params.session_id.as_deref(),
                user_id,
            )
            .await;

            SearchResponse {
                query: search_query,
                results,
                total,
                instant_answer: None,
                ai_answer,
            }
        }
    };

    Ok(Json(response))
}

async fn record_search(
    state: &AppState,
    query: &str,
    intent: &str,
    results_count: i32,
    session_id: Option<&str>,
    user_id: Option<Uuid>,
) {
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

// ─── Math evaluator ───

fn evaluate_math(expr: &str) -> String {
    let expr = expr.replace(' ', "");
    if let Ok(n) = expr.parse::<f64>() {
        return format_number(n);
    }
    match eval_simple(&expr) {
        Some(result) => format_number(result),
        None => format!("Cannot evaluate: {}", expr),
    }
}

fn eval_simple(expr: &str) -> Option<f64> {
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
