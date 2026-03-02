use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

// ─── Row Types ───

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct CrawlQueueRow {
    pub id: Uuid,
    pub url: String,
    pub status: String,
    pub priority: i32,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueStats {
    pub pending: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub failed: i64,
}

// ─── Queries ───

/// Add a URL to the crawl queue. Ignores duplicates (ON CONFLICT DO NOTHING).
pub async fn enqueue(
    pool: &PgPool,
    url: &str,
    priority: i32,
    submitted_by: Option<Uuid>,
) -> Result<Option<CrawlQueueRow>, notice_core::Error> {
    sqlx::query_as::<_, CrawlQueueRow>(
        r#"
        INSERT INTO crawl_queue (url, priority, submitted_by)
        VALUES ($1, $2, $3)
        ON CONFLICT (url) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(url)
    .bind(priority)
    .bind(submitted_by)
    .fetch_optional(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Enqueue multiple URLs at once. Ignores duplicates.
/// Returns the count of newly inserted URLs.
pub async fn enqueue_batch(
    pool: &PgPool,
    urls: &[String],
    priority: i32,
) -> Result<u64, notice_core::Error> {
    if urls.is_empty() {
        return Ok(0);
    }

    let mut inserted: u64 = 0;
    for url in urls {
        let result = sqlx::query(
            r#"
            INSERT INTO crawl_queue (url, priority)
            VALUES ($1, $2)
            ON CONFLICT (url) DO NOTHING
            "#,
        )
        .bind(url)
        .bind(priority)
        .execute(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

        inserted += result.rows_affected();
    }

    Ok(inserted)
}

/// Automically dequeue the next pending URL.
/// Uses FOR UPDATE SKIP LOCKED for safe concurrent access.
pub async fn dequeue_next(pool: &PgPool) -> Result<Option<CrawlQueueRow>, notice_core::Error> {
    sqlx::query_as::<_, CrawlQueueRow>(
        r#"
        UPDATE crawl_queue
        SET status = 'in_progress', updated_at = NOW()
        WHERE id = (
            SELECT id FROM crawl_queue
            WHERE status = 'pending'
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING *
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Mark a crawl queue entry as completed.
pub async fn mark_completed(pool: &PgPool, id: Uuid) -> Result<(), notice_core::Error> {
    sqlx::query("UPDATE crawl_queue SET status = 'completed' WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;
    Ok(())
}

/// Mark a crawl queue entry as failed with an error message.
/// Increments retry_count. If retries exhausted, stays failed.
/// Otherwise resets to pending for retry.
pub async fn mark_failed(pool: &PgPool, id: Uuid, error: &str) -> Result<(), notice_core::Error> {
    sqlx::query(
        r#"
        UPDATE crawl_queue
        SET
            retry_count = retry_count + 1,
            last_error = $2,
            status = CASE
                WHEN retry_count + 1 >= max_retries THEN 'failed'
                ELSE 'pending'
            END
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(error)
    .execute(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))?;
    Ok(())
}

/// Check if a URL already exists in documents OR in the crawl queue.
/// Used to avoid enqueuing URLs we already know about.
pub async fn url_is_known(pool: &PgPool, url: &str) -> Result<bool, notice_core::Error> {
    let in_docs: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM documents WHERE url = $1)")
        .bind(url)
        .fetch_one(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    if in_docs.0 {
        return Ok(true);
    }

    let in_queue: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM crawl_queue WHERE url = $1)")
            .bind(url)
            .fetch_one(pool)
            .await
            .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    Ok(in_queue.0)
}

/// Reset any stale in_progress items back to pending.
/// Called at startup in case the server crashed while processing.
pub async fn reset_stale(pool: &PgPool) -> Result<u64, notice_core::Error> {
    let result =
        sqlx::query("UPDATE crawl_queue SET status = 'pending' WHERE status = 'in_progress'")
            .execute(pool)
            .await
            .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    Ok(result.rows_affected())
}

/// Get queue statistics.
pub async fn stats(pool: &PgPool) -> Result<QueueStats, notice_core::Error> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'pending'),
            COUNT(*) FILTER (WHERE status = 'in_progress'),
            COUNT(*) FILTER (WHERE status = 'completed'),
            COUNT(*) FILTER (WHERE status = 'failed')
        FROM crawl_queue
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))?;

    Ok(QueueStats {
        pending: row.0,
        in_progress: row.1,
        completed: row.2,
        failed: row.3,
    })
}
