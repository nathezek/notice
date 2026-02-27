use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

// ─── Row Types ───

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public user info (no password hash). Returned in API responses.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct UserPublicRow {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

// ─── Queries ───

/// Create a new user. Password must already be hashed.
pub async fn create(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<UserRow, notice_core::Error> {
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            notice_core::Error::Conflict(format!("Username already taken: {}", username))
        }
        _ => notice_core::Error::Database(e.to_string()),
    })
}

/// Get a user by ID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, notice_core::Error> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))
}

/// Get a user by username (for login).
pub async fn get_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<UserRow>, notice_core::Error> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| notice_core::Error::Database(e.to_string()))
}
