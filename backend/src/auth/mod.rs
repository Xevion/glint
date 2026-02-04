use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use axum_extra::extract::CookieJar;

use crate::{
    db::DbPool,
    error::{AppError, AppResult},
    models::{Session, User},
    repo::SessionRepo,
    state::AppState,
};

pub const SESSION_COOKIE_NAME: &str = "glint_session";
pub const SESSION_DURATION_DAYS: i64 = 7;

/// Create a new session in the database (delegates to SessionRepo)
pub async fn create_session(db: &DbPool, user_id: i32) -> AppResult<Session> {
    SessionRepo::create(db, user_id).await
}

/// Validate a session token and return the associated user (delegates to SessionRepo)
pub async fn validate_session(db: &DbPool, token: &str) -> AppResult<(User, Session)> {
    SessionRepo::validate(db, token).await
}

/// Delete a session by token (delegates to SessionRepo)
pub async fn delete_session(db: &DbPool, token: &str) -> AppResult<bool> {
    SessionRepo::delete(db, token).await
}

/// Authenticated user extractor - fails if not authenticated
pub struct AuthUser {
    pub user: User,
    pub session: Session,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts, state).await?;

        let (user, session) = SessionRepo::validate(state.db(), &token).await?;

        Ok(AuthUser { user, session })
    }
}

/// Optional authenticated user extractor - returns None if not authenticated
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for MaybeAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match extract_token(parts, state).await {
            Ok(token) => match SessionRepo::validate(state.db(), &token).await {
                Ok((user, session)) => Ok(MaybeAuthUser(Some(AuthUser { user, session }))),
                Err(_) => Ok(MaybeAuthUser(None)),
            },
            Err(_) => Ok(MaybeAuthUser(None)),
        }
    }
}

/// Extract session token from Authorization header or cookie
async fn extract_token(parts: &mut Parts, state: &AppState) -> AppResult<String> {
    // Try Authorization header first: "Bearer <token>"
    if let Some(auth_header) = parts.headers.get(AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        return Ok(token.to_string());
    }

    // Fall back to cookie
    let jar = CookieJar::from_request_parts(parts, state)
        .await
        .map_err(|_| AppError::Unauthorized("Failed to extract cookies".to_string()))?;

    jar.get(SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("No session token provided".to_string()))
}
