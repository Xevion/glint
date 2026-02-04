use anyhow::Context;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateWorldRequest, UpdateWorldRequest, World};

pub struct WorldRepo;

impl WorldRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: &str) -> AppResult<Option<World>> {
        sqlx::query_as!(World, "SELECT * FROM worlds WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find world '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<World> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("World '{}' not found", id)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<World>> {
        sqlx::query_as!(World, "SELECT * FROM worlds WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to find world by slug '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_slug(db: &DbPool, slug: &str) -> AppResult<World> {
        Self::find_by_slug(db, slug)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("World '{}' not found", slug)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn exists_by_slug(db: &DbPool, slug: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM worlds WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to check world existence '{}'", slug))?;

        Ok(result.is_some())
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn exists_by_id(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM worlds WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to check world existence by id '{}'", id))?;

        Ok(result.is_some())
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn list(db: &DbPool) -> AppResult<Vec<World>> {
        let worlds = sqlx::query_as!(World, "SELECT * FROM worlds ORDER BY created_at DESC")
            .fetch_all(db)
            .await
            .context("failed to list worlds")?;

        debug!(count = worlds.len(), "Listed worlds");
        Ok(worlds)
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn create(db: &DbPool, id: &str, req: &CreateWorldRequest<'_>) -> AppResult<World> {
        let result = sqlx::query!(
            r#"
            INSERT INTO worlds (id, name, slug, description, minecraft_version, file_url, file_hash, size_bytes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())
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
        .execute(db)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result
            && db_err.code().as_deref() == Some("23505")
        {
            return Err(AppError::Conflict(format!(
                "World with slug '{}' already exists",
                req.slug
            )));
        }
        result.context(format!("failed to create world '{}'", req.slug))?;

        Self::get_by_id(db, id).await
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM worlds WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete world '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn update(db: &DbPool, id: &str, req: &UpdateWorldRequest) -> AppResult<World> {
        sqlx::query!(
            r#"
            UPDATE worlds SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                updated_at = now()
            WHERE id = $3
            "#,
            req.name,
            req.description,
            id
        )
        .execute(db)
        .await
        .context(format!("failed to update world '{}'", id))?;

        Self::get_by_id(db, id).await
    }
}
