use axum::Json;
use axum::extract::State;

use notice_core::types::{AuthResponse, CreateUserRequest, LoginRequest};

use crate::error::ApiError;
use crate::middleware::AuthUser;
use crate::state::AppState;

/// POST /api/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    if body.username.trim().is_empty() {
        return Err(notice_core::Error::Validation("Username cannot be empty".into()).into());
    }
    if body.password.len() < 8 {
        return Err(notice_core::Error::Validation(
            "Password must be at least 8 characters".into(),
        )
        .into());
    }

    let username = body.username.trim().to_lowercase();
    let password_hash = notice_auth::hash_password(&body.password)?;
    let user = notice_db::users::create(&state.db, &username, &password_hash).await?;
    let token = notice_auth::create_token(&user.id, &user.username, &state.jwt_secret)?;

    tracing::info!(user_id = %user.id, username = %username, "User registered");

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let username = body.username.trim().to_lowercase();

    let user = notice_db::users::get_by_username(&state.db, &username)
        .await?
        .ok_or_else(|| notice_core::Error::Auth("Invalid username or password".into()))?;

    let valid = notice_auth::verify_password(&body.password, &user.password_hash)?;
    if !valid {
        return Err(notice_core::Error::Auth("Invalid username or password".into()).into());
    }

    let token = notice_auth::create_token(&user.id, &user.username, &state.jwt_secret)?;

    tracing::info!(user_id = %user.id, "User logged in");

    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
    }))
}

/// GET /api/auth/me — Get current user info from JWT
pub async fn me(auth: AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": auth.user_id,
        "username": auth.username
    }))
}
