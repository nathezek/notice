use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::state::AppState;
use notice_classifier::QueryIntent;
use notice_core::types::{InstantAnswer, SearchResponse};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// GET /api/search?q=your+query
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<SearchResponse> {
    tracing::info!(query = %params.q, "Search request received");

    // Step 1: Classify intent
    let intent = notice_classifier::classify(&params.q, &state.gemini).await;

    match intent {
        QueryIntent::Calculate(expr) => {
            // TODO: evaluate the math expression
            Json(SearchResponse {
                query: params.q,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "calculation".to_string(),
                    value: format!("TODO: evaluate '{}'", expr),
                }),
            })
        }

        QueryIntent::Define(term) => {
            // TODO: look up definition
            Json(SearchResponse {
                query: params.q,
                results: vec![],
                total: 0,
                instant_answer: Some(InstantAnswer {
                    answer_type: "definition".to_string(),
                    value: format!("TODO: define '{}'", term),
                }),
            })
        }

        QueryIntent::Timer(command) => Json(SearchResponse {
            query: params.q,
            results: vec![],
            total: 0,
            instant_answer: Some(InstantAnswer {
                answer_type: "timer".to_string(),
                value: format!("TODO: parse timer from '{}'", command),
            }),
        }),

        QueryIntent::Search(query) => {
            // TODO: query expansion → KG context → Meilisearch → enrich results
            let _ = &state.search; // will be used when we add Meilisearch queries
            let _ = &state.db; // will be used for KG lookups

            Json(SearchResponse {
                query,
                results: vec![],
                total: 0,
                instant_answer: None,
            })
        }
    }
}
