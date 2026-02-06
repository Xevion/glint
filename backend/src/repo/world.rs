use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult};
use crate::models::{CreateWorldRequest, UpdateWorldRequest, World};

pub struct WorldRepo;

impl WorldRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<World>> {
        sqlx::query_as!(World, "SELECT * FROM worlds WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find world '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<World> {
        Self::find_by_id(executor, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("World '{}' not found", id)))
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<Option<World>> {
        sqlx::query_as!(World, "SELECT * FROM worlds WHERE slug = $1", slug)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find world by slug '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_slug(executor: impl sqlx::PgExecutor<'_>, slug: &str) -> AppResult<World> {
        Self::find_by_slug(executor, slug)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("World '{}' not found", slug)))
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM worlds WHERE slug = $1", slug)
            .fetch_optional(executor)
            .await
            .context(format!("failed to check world existence '{}'", slug))?;

        Ok(result.is_some())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM worlds WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to check world existence by id '{}'", id))?;

        Ok(result.is_some())
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn list(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<World>> {
        let worlds = sqlx::query_as!(World, "SELECT * FROM worlds ORDER BY created_at DESC")
            .fetch_all(executor)
            .await
            .context("failed to list worlds")?;

        debug!(count = worlds.len(), "Listed worlds");
        Ok(worlds)
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &CreateWorldRequest<'_>,
    ) -> AppResult<World> {
        let result = sqlx::query_as!(
            World,
            r#"
            INSERT INTO worlds (id, name, slug, description, minecraft_version, file_url, file_hash, size_bytes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
            RETURNING *
            "#,
            id,
            req.name,
            req.slug,
            req.description,
            req.minecraft_version,
            req.file_url,
            req.file_hash,
            req.size_bytes
        )
        .fetch_one(executor)
        .await;

        match result {
            Err(sqlx::Error::Database(ref db_err)) if db_err.code().as_deref() == Some("23505") => {
                Err(AppError::Conflict(format!(
                    "World with slug '{}' already exists",
                    req.slug
                )))
            }
            Err(e) => Err(e)
                .context(format!("failed to create world '{}'", req.slug))
                .map_err(Into::into),
            Ok(world) => Ok(world),
        }
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM worlds WHERE id = $1", id)
            .execute(executor)
            .await
            .context(format!("failed to delete world '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn update(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &UpdateWorldRequest,
    ) -> AppResult<World> {
        sqlx::query_as!(
            World,
            r#"
            UPDATE worlds SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                updated_at = now()
            WHERE id = $3
            RETURNING *
            "#,
            req.name,
            req.description,
            id
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to update world '{}'", id))
        .map_err(Into::into)
    }
}
