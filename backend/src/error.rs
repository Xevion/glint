use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::error;

use crate::platform::PlatformError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Gone: {0}")]
    Gone(String),

    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Database error")]
    Database(#[source] sqlx::Error),

    #[error("{0}")]
    Internal(#[source] anyhow::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

impl From<PlatformError> for AppError {
    fn from(e: PlatformError) -> Self {
        match e {
            PlatformError::NotFound => AppError::NotFound("Resource not found on platform".into()),
            PlatformError::RateLimited { retry_after_secs } => {
                AppError::RateLimited { retry_after_secs }
            }
            PlatformError::BadStatus { status, body } => {
                error!(status, body = %body, "Platform API error");
                AppError::ServiceUnavailable(format!("Platform API returned status {}", status))
            }
            PlatformError::Http(e) => {
                error!(error = ?e, "Platform HTTP error");
                AppError::ServiceUnavailable("Failed to connect to platform API".into())
            }
            PlatformError::Deserialize { url, source, .. } => {
                error!(url = %url, error = ?source, "Failed to parse platform response");
                AppError::Internal(source.into())
            }
        }
    }
}

impl AppError {
    /// Extract the HTTP status code and error code string for this error.
    pub fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            AppError::Gone(_) => (StatusCode::GONE, "GONE"),
            AppError::RateLimited { .. } => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            AppError::ServiceUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE")
            }
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Handle RateLimited separately to add Retry-After header
        if let AppError::RateLimited { retry_after_secs } = &self {
            let body = Json(json!({
                "error": self.to_string(),
                "code": "RATE_LIMITED"
            }));
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [("retry-after", retry_after_secs.to_string())],
                body,
            )
                .into_response();
        }

        let (status, error_code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::Gone(msg) => (StatusCode::GONE, "GONE", msg.clone()),
            AppError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                msg.clone(),
            ),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            AppError::Database(e) => {
                error!(error = %e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database error occurred".to_string(),
                )
            }
            AppError::Internal(e) => {
                // Log full error chain for debugging
                error!(error = ?e, "Internal error: {:#}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error occurred".to_string(),
                )
            }
            AppError::RateLimited { .. } => unreachable!("handled above"),
        };

        let body = Json(json!({
            "error": message,
            "code": error_code
        }));
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
