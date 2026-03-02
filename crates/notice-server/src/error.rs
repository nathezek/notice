use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Wrapper around notice_core::Error that implements IntoResponse.
/// Allows routes to return Result<T, ApiError> and use the ? operator.
pub struct ApiError(pub notice_core::Error);

impl From<notice_core::Error> for ApiError {
    fn from(e: notice_core::Error) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            notice_core::Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            notice_core::Error::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            notice_core::Error::Auth(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            notice_core::Error::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            other => {
                tracing::error!(error = %other, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = json!({
            "error": message,
            "status": status.as_u16()
        });

        (status, Json(body)).into_response()
    }
}
