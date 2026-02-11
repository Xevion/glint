use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult, SqlxResultExt};
use crate::models::{CreateWorldRequest, UpdateWorldRequest, World};

/// Aggregate counts per world for the list endpoint.
#[derive(Debug, sqlx::FromRow)]
pub struct WorldAggregate {
    pub world_id: String,
    pub scene_count: i64,
    pub version_count: i64,
    pub capture_count: i64,
}

/// Preview capture for a world (most recent available image).
#[derive(Debug, sqlx::FromRow)]
pub struct WorldPreviewRow {
    pub world_id: String,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
}

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

    /// Fetch scene, version, and capture counts for every world.
    #[instrument(skip(executor), level = "debug")]
    pub async fn aggregate_counts(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<WorldAggregate>> {
        sqlx::query_as::<_, WorldAggregate>(
            r#"
            SELECT
                w.id AS world_id,
                COALESCE(s.scene_count, 0) AS scene_count,
                COALESCE(v.version_count, 0) AS version_count,
                COALESCE(c.capture_count, 0) AS capture_count
            FROM worlds w
            LEFT JOIN (
                SELECT world_id, COUNT(*) AS scene_count FROM scenes GROUP BY world_id
            ) s ON s.world_id = w.id
            LEFT JOIN (
                SELECT world_id, COUNT(*) AS version_count FROM world_versions GROUP BY world_id
            ) v ON v.world_id = w.id
            LEFT JOIN (
                SELECT s2.world_id, COUNT(*) AS capture_count
                FROM captures cap
                JOIN scenes s2 ON s2.id = cap.scene_id
                GROUP BY s2.world_id
            ) c ON c.world_id = w.id
            "#,
        )
        .fetch_all(executor)
        .await
        .context("failed to fetch world aggregates")
        .map_err(AppError::from)
    }

    /// Fetch the most recent capture preview image for every world.
    #[instrument(skip(executor), level = "debug")]
    pub async fn preview_captures(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<Vec<WorldPreviewRow>> {
        sqlx::query_as::<_, WorldPreviewRow>(
            r#"
            SELECT DISTINCT ON (s.world_id)
                s.world_id,
                cap.image_url,
                cap.thumbhash
            FROM captures cap
            JOIN scenes s ON s.id = cap.scene_id
            WHERE cap.image_url IS NOT NULL OR cap.thumbhash IS NOT NULL
            ORDER BY s.world_id, cap.created_at DESC
            "#,
        )
        .fetch_all(executor)
        .await
        .context("failed to fetch world preview captures")
        .map_err(AppError::from)
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
            INSERT INTO worlds (id, name, slug, description, minecraft_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, now(), now())
            RETURNING *
            "#,
            id,
            req.name,
            req.slug,
            req.description,
            req.minecraft_version,
        )
        .fetch_one(executor)
        .await;

        result.conflict_on_unique(format!("World with slug '{}' already exists", req.slug))
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
