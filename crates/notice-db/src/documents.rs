use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

// ─── Row Types ───

/// Full document row (includes raw_content).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct DocumentRow {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub raw_content: String,
    pub summary: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight document row for listings (no raw_content).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct DocumentListRow {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Queries ───

/// Insert a new document. Extracts domain from the URL automatically.
/// Returns Conflict error if URL already exists.
pub async fn insert(
    pool: &PgPool,
    doc_url: &str,
    title: Option<&str>,
    raw_content: &str,
) -> Result<DocumentRow, notice_core::Error> {
    let domain = extract_domain(doc_url)?;

    sqlx::query_as::<_, DocumentRow>(
        r#"
        INSERT INTO documents (url, domain, title, raw_content)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(doc_url)
    .bind(&domain)
    .bind(title)
    .bind(raw_content)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            notice_core::Error::Conflict(format!("Document already exists: {}", doc_url))
        }
        _ => notice_core::Error::Database(e.to_string()),
    })
}

/// Get a document by ID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DocumentRow>, notice_core::Error> {
    sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get a document by URL.
pub async fn get_by_url(
    pool: &PgPool,
    url: &str,
) -> Result<Option<DocumentRow>, notice_core::Error> {
    sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE url = $1")
        .bind(url)
        .fetch_optional(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Update a document's summary and set status to 'summarized'.
pub async fn update_summary(
    pool: &PgPool,
    id: Uuid,
    summary: &str,
) -> Result<DocumentRow, notice_core::Error> {
    sqlx::query_as::<_, DocumentRow>(
        r#"
        UPDATE documents
        SET summary = $2, status = 'summarized'
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(summary)
    .fetch_one(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Mark a document's summarization as failed.
pub async fn mark_summary_failed(pool: &PgPool, id: Uuid) -> Result<(), notice_core::Error> {
    sqlx::query("UPDATE documents SET status = 'failed' WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;
    Ok(())
}

/// List documents with pagination (lightweight — no raw_content).
pub async fn list(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<DocumentListRow>, notice_core::Error> {
    sqlx::query_as::<_, DocumentListRow>(
        r#"
        SELECT id, url, domain, title, summary, status, created_at, updated_at
        FROM documents
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Count total documents.
pub async fn count(pool: &PgPool) -> Result<i64, notice_core::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM documents")
        .fetch_one(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;
    Ok(row.0)
}

// ─── Helpers ───

fn extract_domain(raw_url: &str) -> Result<String, notice_core::Error> {
    let parsed = url::Url::parse(raw_url)
        .map_err(|e| notice_core::Error::Validation(format!("Invalid URL: {}", e)))?;
    Ok(parsed.host_str().unwrap_or("unknown").to_string())
}
