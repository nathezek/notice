use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use notice_classifier::QueryIntent;
use notice_core::types::{InstantAnswer, SearchResponse, SummaryResponse};

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

#[derive(Debug, Deserialize)]
pub struct SummaryParams {
    pub q: String,
}

/// GET /api/search?q=your+query
///
/// Pipeline (fast path — no AI blocking):
/// 1. Classify intent (calculate / define / timer / search)
/// 2. If instant answer → return immediately
/// 3. If search → query Meilisearch directly
/// 4. If results insufficient → trigger on-demand discovery (background), set flag
/// 5. Record in search history
/// 6. Return results + discovery_triggered flag (NO ai_answer)
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
                discovery_triggered: false,
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
                discovery_triggered: false,
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
                discovery_triggered: false,
            }
        }

        QueryIntent::Search(search_query) => {
            // Step 2: Search Meilisearch directly (fast)
            let (results, total) = state
                .search
                .search(&search_query, limit, offset)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "Meilisearch query failed");
                    (vec![], 0)
                });

            let results_count = results.len() as i32;
            
            tracing::debug!(
                query = %search_query,
                results = results_count,
                top_results = ?results.iter().take(5).map(|r| r.title.as_deref().unwrap_or("Untitled")).collect::<Vec<_>>(),
                "Search results"
            );

            // Step 3: On-demand discovery (fire-and-forget, but signal the client)
            let top_score = results.first().and_then(|r| r.score).unwrap_or(0.0);
            let needs_discovery = results_count < 5 || top_score < 0.6;

            if needs_discovery {
                tracing::info!(
                    query = %search_query, 
                    count = results_count, 
                    top_score = top_score, 
                    "Insufficient or irrelevant results, triggering discovery"
                );
                let db = state.db.clone();
                let discovery_query = search_query.clone();
                
                tokio::spawn(async move {
                    let discovered_urls = notice_crawler::discovery::find_urls(&discovery_query).await;
                    for url in discovered_urls {
                        if let Err(e) = notice_db::crawl_queue::enqueue(&db, &url, 10, None).await {
                            tracing::warn!(url = %url, error = %e, "Failed to enqueue discovered URL");
                        }
                    }
                });
            }

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

            // Return websites immediately — NO ai_answer here (decoupled)
            SearchResponse {
                query: search_query,
                results,
                total,
                instant_answer: None,
                ai_answer: None,
                discovery_triggered: needs_discovery,
            }
        }
    };

    Ok(Json(response))
}

/// GET /api/search/summary?q=your+query
///
/// Separate endpoint for AI summary (decoupled from search):
/// 1. Check Postgres cache → return cached if found
/// 2. Fetch top search results for context
/// 3. Call Gemini → normalize response → store in cache
/// 4. Return clean title + summary
pub async fn search_summary(
    State(state): State<AppState>,
    Query(params): Query<SummaryParams>,
) -> Result<Json<SummaryResponse>, ApiError> {
    let query = params.q.trim().to_string();
    if query.is_empty() {
        return Err(notice_core::Error::Validation("Query cannot be empty".into()).into());
    }

    // Step 1: Check Postgres cache (explicit error handling)
    match notice_db::query_summaries::get_by_query(&state.db, &query).await {
        Ok(Some(cached)) => {
            tracing::info!(query = %query, "Using cached AI summary");
            let (title, summary) = extract_title_summary_from_value(&cached.answer);
            return Ok(Json(SummaryResponse {
                query,
                title,
                summary,
                cached: true,
            }));
        }
        Ok(None) => {
            tracing::debug!(query = %query, "No cached summary, will generate");
        }
        Err(e) => {
            tracing::warn!(query = %query, error = %e, "DB cache lookup failed, will try generating");
        }
    }

    // Step 2: Get search results for context
    let (results, _) = state
        .search
        .search(&query, 5, 0)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "Meilisearch query failed during summary generation");
            (vec![], 0)
        });

    if results.is_empty() {
        return Ok(Json(SummaryResponse {
            query,
            title: "No Results".to_string(),
            summary: "No relevant content found to generate a summary for this query.".to_string(),
            cached: false,
        }));
    }

    // Step 3: Build context and call Gemini
    let contexts: Vec<String> = results
        .iter()
        .take(5)
        .map(|r| {
            format!(
                "Title: {}\nURL: {}\nSnippet: {}",
                r.title.as_deref().unwrap_or("Untitled"),
                r.url,
                r.snippet
            )
        })
        .collect();

    match state.gemini.answer_query(&query, &contexts).await {
        Ok(answer) => {
            // Step 4: Normalize and store
            let (title, summary) = normalize_ai_response(&answer);

            // Store the normalized version as JSON in Postgres
            let json_val = serde_json::json!({
                "title": title,
                "summary": summary,
            });

            if let Err(e) = notice_db::query_summaries::insert(&state.db, &query, &json_val).await {
                tracing::warn!(error = %e, "Failed to cache AI summary");
            }

            Ok(Json(SummaryResponse {
                query,
                title,
                summary,
                cached: false,
            }))
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to generate AI summary");
            // Return a meaningful error instead of silently dropping
            Ok(Json(SummaryResponse {
                query,
                title: "Summary Unavailable".to_string(),
                summary: "We couldn't generate a summary at this time. Please try again later.".to_string(),
                cached: false,
            }))
        }
    }
}

/// Normalize raw Gemini output into clean (title, summary) strings.
/// Handles: markdown code fences, raw JSON, plain text, and malformed responses.
fn normalize_ai_response(raw: &str) -> (String, String) {
    // Strip markdown code fences if present
    let cleaned = strip_code_fences(raw);

    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cleaned) {
        return extract_title_summary_from_value(&json);
    }

    // Fallback: treat the entire response as plain text summary
    ("Overview".to_string(), cleaned.to_string())
}

/// Extract title and summary from a serde_json::Value.
/// Handles both `{"title": "...", "summary": "..."}` and stringified JSON.
fn extract_title_summary_from_value(val: &serde_json::Value) -> (String, String) {
    // If it's a JSON object with title/summary fields
    if let Some(obj) = val.as_object() {
        let title = obj.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Overview")
            .to_string();
        let summary = obj.get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return (title, summary);
    }

    // If it's a JSON string (double-encoded), try parsing the inner string
    if let Some(s) = val.as_str() {
        let stripped = strip_code_fences(s);
        if let Ok(inner) = serde_json::from_str::<serde_json::Value>(&stripped) {
            return extract_title_summary_from_value(&inner);
        }
        // Plain string as summary
        return ("Overview".to_string(), s.to_string());
    }

    ("Overview".to_string(), val.to_string())
}

/// Strip markdown code fences from a string.
/// Handles ```json\n...\n``` and ```...\n``` patterns.
fn strip_code_fences(s: &str) -> &str {
    let trimmed = s.trim();
    
    // Try ```json\n...\n```
    if let Some(rest) = trimmed.strip_prefix("```json") {
        if let Some(content) = rest.strip_suffix("```") {
            return content.trim();
        }
    }
    
    // Try ```\n...\n```
    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(content) = rest.strip_suffix("```") {
            return content.trim();
        }
    }

    trimmed
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
