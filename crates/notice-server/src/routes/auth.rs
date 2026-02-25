use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::state::AppState;
use notice_core::types::{AuthResponse, CreateUserRequest, LoginRequest};

/// POST /api/auth/register
pub async fn register(
    State(_state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    tracing::info!(username = %body.username, "Registration request");

    // TODO: check if username exists in DB
    // TODO: hash password with notice_auth::hash_password
    // TODO: insert user into DB
    // TODO: generate JWT with notice_auth::create_token

    let _ = notice_auth::hash_password(&body.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Placeholder response until we have the DB schema
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// POST /api/auth/login
pub async fn login(
    State(_state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    tracing::info!(username = %body.username, "Login request");

    // TODO: look up user in DB
    // TODO: verify password with notice_auth::verify_password
    // TODO: generate JWT with notice_auth::create_token

    let _ = body;

    // Placeholder response until we have the DB schema
    Err(StatusCode::NOT_IMPLEMENTED)
}
