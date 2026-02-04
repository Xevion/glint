use anyhow::Context;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateSceneRequest, Scene, UpdateSceneRequest};

pub struct SceneRepo;

impl SceneRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: &str) -> AppResult<Option<Scene>> {
        sqlx::query_as!(Scene, "SELECT * FROM scenes WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find scene '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<Scene> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Scene '{}' not found", id)))
    }

    /// List all active scenes
    #[instrument(skip(db), level = "debug")]
    pub async fn list_active(db: &DbPool) -> AppResult<Vec<Scene>> {
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE active = TRUE ORDER BY name"
        )
        .fetch_all(db)
        .await
        .context("failed to list active scenes")?;

        debug!(count = scenes.len(), "Listed active scenes");
        Ok(scenes)
    }

    /// List scenes by world
    #[instrument(skip(db), level = "debug")]
    pub async fn list_by_world(db: &DbPool, world_id: &str) -> AppResult<Vec<Scene>> {
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE world_id = $1 AND active = TRUE ORDER BY name",
            world_id
        )
        .fetch_all(db)
        .await
        .context(format!("failed to list scenes for world '{}'", world_id))?;

        debug!(count = scenes.len(), "Listed scenes for world");
        Ok(scenes)
    }

    /// Find active scenes by slug (scenes can share slugs across worlds)
    #[instrument(skip(db), level = "debug")]
    pub async fn find_active_by_slug(db: &DbPool, slug: &str) -> AppResult<Vec<Scene>> {
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE slug = $1 AND active = TRUE",
            slug
        )
        .fetch_all(db)
        .await
        .context(format!("failed to find scenes by slug '{}'", slug))?;

        Ok(scenes)
    }

    /// Find active scene by slug and world_id (unique)
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug_and_world(
        db: &DbPool,
        slug: &str,
        world_id: &str,
    ) -> AppResult<Option<Scene>> {
        sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE slug = $1 AND world_id = $2 AND active = TRUE",
            slug,
            world_id
        )
        .fetch_optional(db)
        .await
        .context(format!(
            "failed to find scene '{}' in world '{}'",
            slug, world_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_slug_and_world(
        db: &DbPool,
        slug: &str,
        world_id: &str,
    ) -> AppResult<Scene> {
        Self::find_by_slug_and_world(db, slug, world_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Scene '{}' not found in this world", slug)))
    }

    /// Check if a scene slug exists in a world (active only)
    #[instrument(skip(db), level = "debug")]
    pub async fn exists_by_slug_in_world(
        db: &DbPool,
        slug: &str,
        world_id: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            "SELECT 1 as one FROM scenes WHERE world_id = $1 AND slug = $2 AND active = TRUE",
            world_id,
            slug
        )
        .fetch_optional(db)
        .await
        .context(format!(
            "failed to check scene existence '{}' in world '{}'",
            slug, world_id
        ))?;

        Ok(result.is_some())
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn create(db: &DbPool, id: &str, req: &CreateSceneRequest) -> AppResult<Scene> {
        sqlx::query!(
            r#"
            INSERT INTO scenes (
                id, name, slug, world_id, x, y, z, pitch, yaw,
                dimension, time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
                active, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, TRUE, now())
            "#,
            id,
            req.name,
            req.slug,
            req.world_id,
            req.position.x,
            req.position.y,
            req.position.z,
            req.camera.pitch,
            req.camera.yaw,
            req.dimension,
            req.time_of_day,
            req.weather,
            req.weather_intensity,
            req.moon_phase,
            req.biome
        )
        .execute(db)
        .await
        .context(format!("failed to create scene '{}'", req.slug))?;

        Self::get_by_id(db, id).await
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn update(db: &DbPool, id: &str, req: &UpdateSceneRequest) -> AppResult<Scene> {
        sqlx::query!(
            r#"
            UPDATE scenes
            SET x = $1, y = $2, z = $3, pitch = $4, yaw = $5,
                dimension = $6, time_of_day_ticks = $7, weather = $8,
                weather_intensity = $9, moon_phase = $10, biome = $11
            WHERE id = $12
            "#,
            req.position.x,
            req.position.y,
            req.position.z,
            req.camera.pitch,
            req.camera.yaw,
            req.dimension,
            req.time_of_day,
            req.weather,
            req.weather_intensity,
            req.moon_phase,
            req.biome,
            id
        )
        .execute(db)
        .await
        .context(format!("failed to update scene '{}'", id))?;

        Self::get_by_id(db, id).await
    }

    /// Disable a scene (soft delete)
    #[instrument(skip(db), level = "debug")]
    pub async fn disable(db: &DbPool, slug: &str, world_id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = FALSE WHERE slug = $1 AND world_id = $2 AND active = TRUE",
            slug,
            world_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to disable scene '{}' in world '{}'",
            slug, world_id
        ))?;

        Ok(result.rows_affected() > 0)
    }

    /// Reactivate a disabled scene
    #[instrument(skip(db), level = "debug")]
    pub async fn reactivate(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = TRUE WHERE id = $1 AND active = FALSE",
            id
        )
        .execute(db)
        .await
        .context(format!("failed to reactivate scene '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }
}
