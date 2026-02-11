use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use axum_extra::extract::CookieJar;

use crate::{
    cache::SessionCache,
    db::DbPool,
    error::{AppError, AppResult},
    models::{Session, User},
    repo::SessionRepo,
    state::AppState,
};

pub const SESSION_COOKIE_NAME: &str = "glint_session";

/// Create a new session in the database (delegates to SessionRepo)
/// `source` indicates how the session was created: "web" (browser) or "device" (mod)
pub async fn create_session(db: &DbPool, user_id: i32, source: &str) -> AppResult<Session> {
    SessionRepo::create(db, user_id, source).await
}

/// Validate a session token and return the associated user (cache-through)
pub async fn validate_session(
    db: &DbPool,
    cache: &SessionCache,
    token: &str,
) -> AppResult<(User, Session)> {
    cache.get_or_validate(db, token).await
}

/// Delete a session by token and evict from cache
pub async fn delete_session(db: &DbPool, cache: &SessionCache, token: &str) -> AppResult<bool> {
    let deleted = SessionRepo::delete(db, token).await?;
    if deleted {
        cache.invalidate_token(token);
    }
    Ok(deleted)
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
        let (user, session) = state
            .session_cache()
            .get_or_validate(state.db(), &token)
            .await?;
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
            Ok(token) => {
                match state
                    .session_cache()
                    .get_or_validate(state.db(), &token)
                    .await
                {
                    Ok((user, session)) => Ok(MaybeAuthUser(Some(AuthUser { user, session }))),
                    Err(_) => Ok(MaybeAuthUser(None)),
                }
            }
            Err(_) => Ok(MaybeAuthUser(None)),
        }
    }
}

/// Agent user extractor - fails if not authenticated or not an agent/admin.
/// Agents can operate the capture pipeline (runs, work, uploads) but not
/// perform full admin operations like shader/scene CRUD.
pub struct AgentUser {
    pub user: User,
    pub session: Session,
}

impl FromRequestParts<AppState> for AgentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts, state).await?;
        let (user, session) = state
            .session_cache()
            .get_or_validate(state.db(), &token)
            .await?;

        if user.role != "agent" && user.role != "admin" {
            return Err(AppError::Forbidden("Agent access required".to_string()));
        }

        Ok(AgentUser { user, session })
    }
}

/// Admin user extractor - fails if not authenticated or not an admin
pub struct AdminUser {
    pub user: User,
    pub session: Session,
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts, state).await?;
        let (user, session) = state
            .session_cache()
            .get_or_validate(state.db(), &token)
            .await?;

        if user.role != "admin" {
            return Err(AppError::Forbidden("Admin access required".to_string()));
        }

        Ok(AdminUser { user, session })
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
