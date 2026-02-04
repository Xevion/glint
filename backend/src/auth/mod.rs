use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use rand::Rng;

use crate::{
    db::DbPool,
    error::{AppError, AppResult},
    models::{Session, User},
    state::AppState,
};

pub const SESSION_COOKIE_NAME: &str = "glint_session";
pub const SESSION_DURATION_DAYS: i64 = 7;

/// Generate a cryptographically secure session token
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    hex::encode(bytes)
}

/// Create a new session in the database
pub async fn create_session(db: &DbPool, user_id: i32) -> AppResult<Session> {
    let token = generate_session_token();
    let expires_at = Utc::now() + Duration::days(SESSION_DURATION_DAYS);

    let session = sqlx::query_as!(
        Session,
        r#"
        INSERT INTO sessions (token, user_id, expires_at)
        VALUES ($1, $2, $3)
        RETURNING token, user_id, expires_at, created_at
        "#,
        token,
        user_id,
        expires_at
    )
    .fetch_one(db)
    .await?;

    Ok(session)
}

/// Validate a session token and return the associated user
pub async fn validate_session(db: &DbPool, token: &str) -> AppResult<(User, Session)> {
    let result = sqlx::query_as!(
        UserSession,
        r#"
        SELECT
            u.id as user_id,
            u.discord_id,
            u.discord_username,
            u.discord_avatar,
            u.role,
            u.created_at as user_created_at,
            u.updated_at as user_updated_at,
            s.token,
            s.expires_at,
            s.created_at as session_created_at
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token = $1 AND s.expires_at > now()
        "#,
        token
    )
    .fetch_optional(db)
    .await?;

    match result {
        Some(row) => {
            let user = User {
                id: row.user_id,
                discord_id: row.discord_id,
                discord_username: row.discord_username,
                discord_avatar: row.discord_avatar,
                role: row.role,
                created_at: row.user_created_at,
                updated_at: row.user_updated_at,
            };
            let session = Session {
                token: row.token,
                user_id: row.user_id,
                expires_at: row.expires_at,
                created_at: row.session_created_at,
            };
            Ok((user, session))
        }
        None => Err(AppError::Unauthorized(
            "Invalid or expired session".to_string(),
        )),
    }
}

/// Delete a session by token
pub async fn delete_session(db: &DbPool, token: &str) -> AppResult<()> {
    sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
        .execute(db)
        .await?;
    Ok(())
}

/// Upsert a user from Discord OAuth data
pub async fn upsert_user(
    db: &DbPool,
    discord_id: &str,
    discord_username: &str,
    discord_avatar: Option<&str>,
) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (discord_id, discord_username, discord_avatar)
        VALUES ($1, $2, $3)
        ON CONFLICT (discord_id) DO UPDATE SET
            discord_username = EXCLUDED.discord_username,
            discord_avatar = EXCLUDED.discord_avatar,
            updated_at = now()
        RETURNING id, discord_id, discord_username, discord_avatar, role, created_at, updated_at
        "#,
        discord_id,
        discord_username,
        discord_avatar
    )
    .fetch_one(db)
    .await?;

    Ok(user)
}

/// Helper struct for joined user/session query
struct UserSession {
    user_id: i32,
    discord_id: String,
    discord_username: String,
    discord_avatar: Option<String>,
    role: String,
    user_created_at: chrono::DateTime<Utc>,
    user_updated_at: chrono::DateTime<Utc>,
    token: String,
    expires_at: chrono::DateTime<Utc>,
    session_created_at: chrono::DateTime<Utc>,
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

        let (user, session) = validate_session(state.db(), &token).await?;

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
            Ok(token) => match validate_session(state.db(), &token).await {
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
