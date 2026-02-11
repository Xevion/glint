use anyhow::Context;
use chrono::{Duration, Utc};
use rand::Rng;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult};
use crate::models::{Session, User};

/// Raw session row returned by list queries (before `is_current` enrichment)
pub struct SessionRow {
    pub token: String,
    pub source: String,
    pub created_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}

pub const SESSION_DURATION_DAYS: i64 = 7;

pub struct SessionRepo;

/// Generate a cryptographically secure session token
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    hex::encode(bytes)
}

/// Helper struct for joined user/session query
struct UserSessionRow {
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

impl SessionRepo {
    /// Create a new session for a user
    /// `source` indicates how the session was created: "web" (browser) or "device" (mod)
    #[instrument(skip(executor), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        user_id: i32,
        source: &str,
    ) -> AppResult<Session> {
        let token = generate_session_token();
        let expires_at = Utc::now() + Duration::days(SESSION_DURATION_DAYS);

        let session = sqlx::query_as!(
            Session,
            r#"
            INSERT INTO sessions (token, user_id, expires_at, source)
            VALUES ($1, $2, $3, $4)
            RETURNING token, user_id, expires_at, created_at
            "#,
            token,
            user_id,
            expires_at,
            source
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to create session for user '{}'", user_id))?;

        debug!(user_id, source, "Session created");
        Ok(session)
    }

    /// Validate a session token and return the associated user and session
    #[instrument(skip(executor, token), level = "debug")]
    pub async fn validate(
        executor: impl sqlx::PgExecutor<'_>,
        token: &str,
    ) -> AppResult<(User, Session)> {
        let result = sqlx::query_as!(
            UserSessionRow,
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
        .fetch_optional(executor)
        .await
        .context("failed to validate session")?;

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
                debug!(user_id = user.id, "Session validated");
                Ok((user, session))
            }
            None => Err(AppError::Unauthorized(
                "Invalid or expired session".to_string(),
            )),
        }
    }

    /// Delete a session by token
    #[instrument(skip(executor, token), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, token: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(executor)
            .await
            .context("failed to delete session")?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            debug!("Session deleted");
        }
        Ok(deleted)
    }

    /// Delete all expired sessions (cleanup task)
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete_expired(executor: impl sqlx::PgExecutor<'_>) -> AppResult<u64> {
        let result = sqlx::query!("DELETE FROM sessions WHERE expires_at < now()")
            .execute(executor)
            .await
            .context("failed to delete expired sessions")?;

        let count = result.rows_affected();
        if count > 0 {
            debug!(count, "Deleted expired sessions");
        }
        Ok(count)
    }

    /// Delete all sessions for a user (e.g., when changing password or role)
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete_for_user(
        executor: impl sqlx::PgExecutor<'_>,
        user_id: i32,
    ) -> AppResult<u64> {
        let result = sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
            .execute(executor)
            .await
            .context(format!("failed to delete sessions for user '{}'", user_id))?;

        let count = result.rows_affected();
        if count > 0 {
            debug!(user_id, count, "Deleted user sessions");
        }
        Ok(count)
    }

    /// List sessions for a user (returns raw rows for caller to enrich with `is_current`)
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_for_user(
        executor: impl sqlx::PgExecutor<'_>,
        user_id: i32,
    ) -> AppResult<Vec<SessionRow>> {
        let sessions = sqlx::query_as!(
            SessionRow,
            r#"
            SELECT token, source, created_at, expires_at
            FROM sessions
            WHERE user_id = $1 AND expires_at > now()
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list sessions for user '{}'", user_id))?;

        Ok(sessions)
    }

    /// Delete a session by token prefix (first 8 chars) for a specific user.
    /// Returns the full token of the deleted session (for cache eviction), or None.
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete_by_prefix(
        executor: impl sqlx::PgExecutor<'_>,
        user_id: i32,
        prefix: &str,
    ) -> AppResult<Option<String>> {
        if prefix.len() != 8 || !prefix.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(AppError::BadRequest(
                "Session prefix must be exactly 8 hex characters".into(),
            ));
        }

        let like_pattern = format!("{}%", prefix);
        let result = sqlx::query_scalar!(
            r#"
            DELETE FROM sessions
            WHERE user_id = $1 AND token LIKE $2
            RETURNING token
            "#,
            user_id,
            like_pattern
        )
        .fetch_optional(executor)
        .await
        .context("failed to delete session by prefix")?;

        if result.is_some() {
            debug!(user_id, prefix, "Deleted session by prefix");
        }
        Ok(result)
    }

    /// Delete all sessions for a user except the specified token.
    #[instrument(skip(executor, except_token), level = "debug")]
    pub async fn delete_others_for_user(
        executor: impl sqlx::PgExecutor<'_>,
        user_id: i32,
        except_token: &str,
    ) -> AppResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM sessions WHERE user_id = $1 AND token != $2",
            user_id,
            except_token
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to delete other sessions for user '{}'",
            user_id
        ))?;

        let count = result.rows_affected();
        if count > 0 {
            debug!(user_id, count, "Deleted other user sessions");
        }
        Ok(count)
    }
}
