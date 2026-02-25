use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use notice_core::types::{SubmitUrlRequest, SubmitUrlResponse};

use crate::error::ApiError;
use crate::state::AppState;

// ─── Submit URL to crawl queue ───

/// POST /api/submit — Add a URL to the crawl queue for background processing.
pub async fn submit_url(
    State(state): State<AppState>,
    Json(body): Json<SubmitUrlRequest>,
) -> Result<Json<SubmitUrlResponse>, ApiError> {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(notice_core::Error::Validation("URL cannot be empty".into()).into());
    }

    // Validate URL format
    url::Url::parse(&url)
        .map_err(|e| notice_core::Error::Validation(format!("Invalid URL: {}", e)))?;

    // Check if document already exists
    if notice_db::documents::get_by_url(&state.db, &url)
        .await?
        .is_some()
    {
        return Ok(Json(SubmitUrlResponse {
            id: Uuid::nil(),
            url: url.clone(),
            status: "exists".to_string(),
            message: "This URL has already been indexed".to_string(),
        }));
    }

    // Enqueue for crawling
    let entry = notice_db::crawl_queue::enqueue(&state.db, &url, 0, None).await?;

    match entry {
        Some(row) => {
            tracing::info!(url = %url, "URL enqueued for crawling");
            Ok(Json(SubmitUrlResponse {
                id: row.id,
                url: row.url,
                status: "queued".to_string(),
                message: "URL has been added to the crawl queue".to_string(),
            }))
        }
        None => {
            // ON CONFLICT DO NOTHING — already in queue
            Ok(Json(SubmitUrlResponse {
                id: Uuid::nil(),
                url,
                status: "already_queued".to_string(),
                message: "This URL is already in the crawl queue".to_string(),
            }))
        }
    }
}

// ─── Immediate crawl (for development/testing) ───

/// POST /api/crawl — Immediately scrape a URL, summarize it, and store it.
/// This bypasses the crawl queue. Use /api/submit for production.
pub async fn crawl_url(
    State(state): State<AppState>,
    Json(body): Json<SubmitUrlRequest>,
) -> Result<Json<notice_db::documents::DocumentRow>, ApiError> {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(notice_core::Error::Validation("URL cannot be empty".into()).into());
    }

    // Check if already exists
    if let Some(existing) = notice_db::documents::get_by_url(&state.db, &url).await? {
        return Ok(Json(existing));
    }

    tracing::info!(url = %url, "Starting immediate crawl");

    // Step 1: Scrape
    let page = notice_crawler::scrape_url(&url).await?;

    // Step 2: Store in database
    let doc = notice_db::documents::insert(
        &state.db,
        &page.url,
        page.title.as_deref(),
        &page.text_content,
    )
    .await?;

    tracing::info!(doc_id = %doc.id, "Document stored, requesting summary");

    // Step 3: Summarize with Gemini
    // Truncate content to avoid exceeding Gemini's token limit
    let content_for_summary = truncate_for_summary(&page.text_content, 8000);

    let doc = match state.gemini.summarize(&content_for_summary).await {
        Ok(summary) => {
            tracing::info!(doc_id = %doc.id, "Summary generated");
            notice_db::documents::update_summary(&state.db, doc.id, &summary).await?
        }
        Err(e) => {
            tracing::warn!(doc_id = %doc.id, error = %e, "Summarization failed");
            notice_db::documents::mark_summary_failed(&state.db, doc.id).await?;
            notice_db::documents::get_by_id(&state.db, doc.id)
                .await?
                .unwrap_or(doc)
        }
    };

    Ok(Json(doc))
}

// ─── Document listing ───

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/documents — List documents (paginated).
pub async fn list_documents(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let docs = notice_db::documents::list(&state.db, limit, offset).await?;
    let total = notice_db::documents::count(&state.db).await?;

    Ok(Json(serde_json::json!({
        "documents": docs,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

/// GET /api/documents/:id — Get a single document.
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<notice_db::documents::DocumentRow>, ApiError> {
    let doc = notice_db::documents::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| notice_core::Error::NotFound(format!("Document {} not found", id)))?;

    Ok(Json(doc))
}

/// GET /api/queue/stats — Get crawl queue statistics.
pub async fn queue_stats(
    State(state): State<AppState>,
) -> Result<Json<notice_db::crawl_queue::QueueStats>, ApiError> {
    let stats = notice_db::crawl_queue::stats(&state.db).await?;
    Ok(Json(stats))
}

// ─── Helpers ───

fn truncate_for_summary(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        content.to_string()
    } else {
        content[..max_chars].to_string()
    }
}
