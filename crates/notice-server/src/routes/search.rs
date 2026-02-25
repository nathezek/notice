use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use notice_classifier::QueryIntent;
use notice_core::types::{InstantAnswer, SearchResponse};
use notice_kg::{context, updater};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub session_id: Option<String>,
    /// Optional user ID for personalized search.
    /// Will come from JWT middleware later.
    pub user_id: Option<Uuid>,
}

/// GET /api/search?q=your+query&user_id=...
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

    tracing::info!(query = %query, user_id = ?params.user_id, "Search request");

    // Step 1: Classify intent
    let intent = notice_classifier::classify(&query, &state.gemini).await;

    let response = match intent {
        QueryIntent::Calculate(expr) => {
            record_search(
                &state,
                &query,
                "calculate",
                0,
                params.session_id.as_deref(),
                params.user_id,
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
            }
        }

        QueryIntent::Define(term) => {
            record_search(
                &state,
                &query,
                "define",
                0,
                params.session_id.as_deref(),
                params.user_id,
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
            }
        }

        QueryIntent::Timer(command) => {
            record_search(
                &state,
                &query,
                "timer",
                0,
                params.session_id.as_deref(),
                params.user_id,
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
            }
        }

        QueryIntent::Search(search_query) => {
            // Step 2: Load user's KG context (if logged in)
            let (user_context, search_query_final) = if let Some(user_id) = params.user_id {
                // Load context
                let ctx = context::load_user_context(&state.db, user_id, 10)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = %e, "Failed to load user context");
                        context::UserContext::anonymous()
                    });

                if ctx.has_context {
                    // Extract query terms for overlap detection
                    let extracted = notice_kg::extractor::extract_entities(&search_query);
                    let query_terms: Vec<String> =
                        extracted.iter().map(|e| e.name.clone()).collect();

                    // Find which query terms the user has searched before
                    let overlapping =
                        context::find_overlapping_entities(&state.db, user_id, &query_terms)
                            .await
                            .unwrap_or_default();

                    // Augment query with context
                    let augmented = context::augment_query(&search_query, &ctx, &overlapping);

                    tracing::info!(
                        user_id = %user_id,
                        has_context = true,
                        top_interests = ?ctx.top_interests.iter().take(5).map(|t| &t.term).collect::<Vec<_>>(),
                        query_augmented = %augmented,
                        "KG context loaded"
                    );

                    (Some(ctx), augmented)
                } else {
                    tracing::debug!(user_id = %user_id, "No KG context yet for user");
                    (Some(ctx), search_query.clone())
                }
            } else {
                (None, search_query.clone())
            };

            // Step 3: Search Meilisearch with the (possibly augmented) query
            let (results, total) = state
                .search
                .search(&search_query_final, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Meilisearch query failed");
                    (vec![], 0)
                });

            let results_count = results.len() as i32;

            // Step 4: Record search in history
            record_search(
                &state,
                &search_query,
                "search",
                results_count,
                params.session_id.as_deref(),
                params.user_id,
            )
            .await;

            // Step 5: Update KG asynchronously (fire and forget)
            if let Some(user_id) = params.user_id {
                updater::spawn_kg_update(state.db.clone(), user_id, search_query.clone());
            }

            let _ = user_context; // used for logging above

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

/// Record a search in history (fire-and-forget).
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

/// Basic math expression evaluator.
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
