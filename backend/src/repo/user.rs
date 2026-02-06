use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult};
use crate::models::User;

pub struct UserRepo;

impl UserRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: i32,
    ) -> AppResult<Option<User>> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find user by id '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_id(executor: impl sqlx::PgExecutor<'_>, id: i32) -> AppResult<User> {
        Self::find_by_id(executor, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with id '{}' not found", id)))
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_discord_id(
        executor: impl sqlx::PgExecutor<'_>,
        discord_id: &str,
    ) -> AppResult<Option<User>> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE discord_id = $1",
            discord_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to find user by discord_id '{}'",
            discord_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_discord_id(
        executor: impl sqlx::PgExecutor<'_>,
        discord_id: &str,
    ) -> AppResult<User> {
        Self::find_by_discord_id(executor, discord_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("User with discord_id '{}' not found", discord_id))
            })
    }

    /// Create or update a user from Discord OAuth data
    #[instrument(skip(executor), level = "debug")]
    pub async fn upsert(
        executor: impl sqlx::PgExecutor<'_>,
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
        .fetch_one(executor)
        .await
        .context(format!("failed to upsert user '{}'", discord_id))?;

        debug!(user_id = user.id, discord_id, "User upserted");
        Ok(user)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn update_role(
        executor: impl sqlx::PgExecutor<'_>,
        id: i32,
        role: &str,
    ) -> AppResult<User> {
        sqlx::query_as!(
            User,
            r#"
            UPDATE users SET role = $1, updated_at = now()
            WHERE id = $2
            RETURNING id, discord_id, discord_username, discord_avatar, role, created_at, updated_at
            "#,
            role,
            id
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to update role for user '{}'", id))?
        .ok_or_else(|| AppError::NotFound(format!("User with id '{}' not found", id)))
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn list(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<User>> {
        let users = sqlx::query_as!(User, "SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(executor)
            .await
            .context("failed to list users")?;
        debug!(count = users.len(), "Listed users");
        Ok(users)
    }
}
