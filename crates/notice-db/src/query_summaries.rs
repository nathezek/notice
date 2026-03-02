use sqlx::PgPool;
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct QuerySummary {
    pub query: String,
    pub answer: Value,
    pub created_at: DateTime<Utc>,
}

pub async fn get_by_query(pool: &PgPool, query: &str) -> Result<Option<QuerySummary>, notice_core::Error> {
    sqlx::query_as::<_, QuerySummary>(
        "SELECT query, answer, created_at FROM query_summaries WHERE query = $1"
    )
    .bind(query)
    .fetch_optional(pool)
    .await
    .map_err(|e: sqlx::Error| notice_core::Error::Database(e.to_string()))
}

pub async fn insert(pool: &PgPool, query: &str, answer: &Value) -> Result<(), notice_core::Error> {
    sqlx::query(
        "INSERT INTO query_summaries (query, answer) VALUES ($1, $2) ON CONFLICT (query) DO UPDATE SET answer = $2, created_at = CURRENT_TIMESTAMP"
    )
    .bind(query)
    .bind(answer)
    .execute(pool)
    .await
    .map_err(|e: sqlx::Error| notice_core::Error::Database(e.to_string()))?;

    Ok(())
}
