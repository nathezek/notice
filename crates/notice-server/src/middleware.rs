use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use crate::state::AppState;

/// Authenticated user extracted from JWT.
/// Used as an extractor in route handlers.
///
/// Two variants:
/// - `AuthUser` — required auth (returns 401 if missing/invalid)
/// - `OptionalAuthUser` — optional auth (None if not logged in)
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
}

/// Optional auth — extracts user if token is present, None otherwise.
/// Never returns an error — anonymous access is always allowed.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl OptionalAuthUser {
    pub fn user_id(&self) -> Option<Uuid> {
        self.0.as_ref().map(|u| u.user_id)
    }

    #[allow(dead_code)]
    pub fn username(&self) -> Option<&str> {
        self.0.as_ref().map(|u| u.username.as_str())
    }
}

// ─── Required Auth Extractor ───

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        extract_auth_user(parts, &app_state.jwt_secret)
    }
}

// ─── Optional Auth Extractor ───

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        match extract_auth_user(parts, &app_state.jwt_secret) {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

// ─── FromRef trait for AppState ───

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<AppState> for AppState {
    fn from_ref(input: &AppState) -> Self {
        input.clone()
    }
}

// ─── Token Extraction ───

fn extract_auth_user(parts: &Parts, jwt_secret: &str) -> Result<AuthUser, AuthError> {
    // Get the Authorization header
    let auth_header = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // Extract the Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)?;

    // Verify and decode
    let claims =
        notice_auth::verify_token(token, jwt_secret).map_err(|_| AuthError::InvalidToken)?;

    let user_id = notice_auth::user_id_from_claims(&claims).map_err(|_| AuthError::InvalidToken)?;

    Ok(AuthUser {
        user_id,
        username: claims.username,
    })
}

// ─── Auth Error ───

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidFormat,
    InvalidToken,
}

#[derive(Serialize)]
struct AuthErrorResponse {
    error: String,
    status: u16,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Missing authorization token. Use: Authorization: Bearer <token>",
            ),
            AuthError::InvalidFormat => (
                StatusCode::UNAUTHORIZED,
                "Invalid authorization format. Use: Authorization: Bearer <token>",
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token. Please log in again.",
            ),
        };

        let body = AuthErrorResponse {
            error: message.to_string(),
            status: status.as_u16(),
        };

        (status, Json(body)).into_response()
    }
}
