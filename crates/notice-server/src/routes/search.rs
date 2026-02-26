use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use notice_classifier::QueryIntent;
use notice_core::types::{InstantAnswer, SearchResponse};
use notice_kg::{context, session, updater};

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

/// GET /api/search?q=your+query&session_id=...
/// Optional auth — anonymous search works, personalized search when logged in.
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
        user_id = ?user_id,
        authenticated = user_id.is_some(),
        "Search request"
    );

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
            }
        }

        QueryIntent::Search(search_query) => {
            // Extract entities from the query
            let extracted = notice_kg::extractor::extract_entities(&search_query);
            let query_terms: Vec<String> = extracted.iter().map(|e| e.name.clone()).collect();

            // Layer 1: Session Context
            let session_ctx =
                session::build_session_context(&state.db, user_id, params.session_id.as_deref())
                    .await;
            let session_boost = session::get_session_boost_terms(&search_query, &session_ctx);

            // Layer 2 & 3: KG Context + Expansion
            let (kg_context, kg_overlapping, kg_expansion) = if let Some(uid) = user_id {
                let ctx = context::load_user_context(&state.db, uid, 10)
                    .await
                    .unwrap_or_else(|_| context::UserContext::anonymous());

                let overlapping = if ctx.has_context {
                    context::find_overlapping_entities(&state.db, uid, &query_terms)
                        .await
                        .unwrap_or_default()
                } else {
                    vec![]
                };

                let expansion = if ctx.has_context {
                    context::get_kg_expansion_terms(&state.db, uid, &query_terms)
                        .await
                        .unwrap_or_default()
                } else {
                    vec![]
                };

                (ctx, overlapping, expansion)
            } else {
                (context::UserContext::anonymous(), vec![], vec![])
            };

            // Build augmented query
            let final_query = context::augment_query(
                &search_query,
                &kg_context,
                &kg_overlapping,
                &kg_expansion,
                &session_boost,
            );

            if final_query != search_query {
                tracing::info!(
                    original = %search_query,
                    augmented = %final_query,
                    "Query expanded"
                );
            }

            // Search Meilisearch
            let (results, total) = state
                .search
                .search(&final_query, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Meilisearch query failed");
                    (vec![], 0)
                });

            let results_count = results.len() as i32;

            // Record history
            record_search(
                &state,
                &search_query,
                "search",
                results_count,
                params.session_id.as_deref(),
                user_id,
            )
            .await;

            // Update KG
            if let Some(uid) = user_id {
                updater::spawn_kg_update(state.db.clone(), uid, search_query.clone());
            }

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
