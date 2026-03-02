use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct SearchHistoryRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub query: String,
    pub intent: String,
    pub results_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Record a search query.
pub async fn record(
    pool: &PgPool,
    user_id: Option<Uuid>,
    session_id: Option<&str>,
    query: &str,
    intent: &str,
    results_count: i32,
) -> Result<SearchHistoryRow, notice_core::Error> {
    sqlx::query_as::<_, SearchHistoryRow>(
        r#"
        INSERT INTO search_history (user_id, session_id, query, intent, results_count)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(session_id)
    .bind(query)
    .bind(intent)
    .bind(results_count)
    .fetch_one(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}
