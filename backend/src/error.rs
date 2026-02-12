use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::error;

use crate::platform::PlatformError;
use validator::ValidationErrors;

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

impl From<ValidationErrors> for AppError {
    fn from(e: ValidationErrors) -> Self {
        AppError::BadRequest(format!("Validation failed: {}", e))
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
        let (status, code) = self.status_and_code();

        let message = match &self {
            AppError::Database(e) => {
                error!(error = ?e, "Database error");
                "Database error occurred".to_string()
            }
            AppError::Internal(e) => {
                error!(error = ?e, "Internal error: {:#}", e);
                "Internal server error occurred".to_string()
            }
            other => other.to_string(),
        };

        let body = Json(json!({ "error": message, "code": code }));

        if let AppError::RateLimited { retry_after_secs } = &self {
            (
                status,
                [("retry-after", retry_after_secs.to_string())],
                body,
            )
                .into_response()
        } else {
            (status, body).into_response()
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Extension trait for `Option<T>` to convert `None` into [`AppError::NotFound`].
pub trait OptionNotFoundExt<T> {
    /// Convert `None` into [`AppError::NotFound`] with a message like
    /// `"Shader 'abc123' not found"`.
    fn or_not_found(self, entity: &str, id: impl std::fmt::Display) -> AppResult<T>;
}

impl<T> OptionNotFoundExt<T> for Option<T> {
    fn or_not_found(self, entity: &str, id: impl std::fmt::Display) -> AppResult<T> {
        self.ok_or_else(|| AppError::NotFound(format!("{entity} '{id}' not found")))
    }
}

/// Extension trait for `Result<T, sqlx::Error>` to handle unique constraint violations.
pub trait SqlxResultExt<T> {
    /// Convert a PostgreSQL unique-constraint violation (`23505`) into
    /// [`AppError::Conflict`] with the given message. All other errors
    /// become [`AppError::Database`]; `Ok` values pass through.
    fn conflict_on_unique(self, message: impl Into<String>) -> AppResult<T>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn conflict_on_unique(self, message: impl Into<String>) -> AppResult<T> {
        match self {
            Ok(val) => Ok(val),
            Err(sqlx::Error::Database(ref db_err)) if db_err.code().as_deref() == Some("23505") => {
                Err(AppError::Conflict(message.into()))
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }
}
