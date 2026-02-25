use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

// ─── Row Types ───

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

// ─── Queries ───

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

/// Get recent queries for a user (for session context / disambiguation).
pub async fn get_recent_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<SearchHistoryRow>, notice_core::Error> {
    sqlx::query_as::<_, SearchHistoryRow>(
        r#"
        SELECT * FROM search_history
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get recent queries for a session (for anonymous context).
pub async fn get_recent_for_session(
    pool: &PgPool,
    session_id: &str,
    limit: i64,
) -> Result<Vec<SearchHistoryRow>, notice_core::Error> {
    sqlx::query_as::<_, SearchHistoryRow>(
        r#"
        SELECT * FROM search_history
        WHERE session_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(session_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get recent search queries for a user within the last N minutes.
/// Used for session-based disambiguation.
pub async fn get_recent_queries(
    pool: &PgPool,
    user_id: Option<Uuid>,
    session_id: Option<&str>,
    minutes: i64,
    limit: i64,
) -> Result<Vec<String>, notice_core::Error> {
    // Try user-based first, fall back to session-based
    if let Some(uid) = user_id {
        let rows = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT DISTINCT query FROM search_history
            WHERE user_id = $1
            AND intent = 'search'
            AND created_at > NOW() - INTERVAL '1 minute' * $2
            ORDER BY query
            LIMIT $3
            "#,
        )
        .bind(uid)
        .bind(minutes as f64)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

        let queries: Vec<String> = rows.into_iter().map(|(q,)| q).collect();
        if !queries.is_empty() {
            return Ok(queries);
        }
    }

    // Fall back to session-based
    if let Some(sid) = session_id {
        let rows = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT DISTINCT query FROM search_history
            WHERE session_id = $1
            AND intent = 'search'
            AND created_at > NOW() - INTERVAL '1 minute' * $2
            ORDER BY query
            LIMIT $3
            "#,
        )
        .bind(sid)
        .bind(minutes as f64)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))?;

        return Ok(rows.into_iter().map(|(q,)| q).collect());
    }

    Ok(vec![])
}
