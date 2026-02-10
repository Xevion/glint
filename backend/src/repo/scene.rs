use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};

use crate::models::{
    CreateSceneRequest, Scene, SceneWithWorld, UpdateSceneMetadataRequest, UpdateSceneRequest,
};

/// Helper struct for joined scene/world query
struct SceneWithWorldRow {
    id: String,
    name: String,
    slug: String,
    description: Option<String>,
    world_id: String,
    x: f64,
    y: f64,
    z: f64,
    pitch: f64,
    yaw: f64,
    dimension: String,
    time_of_day_ticks: i32,
    weather: String,
    weather_intensity: f64,
    moon_phase: Option<i32>,
    biome: Option<String>,
    definition_json: Option<String>,
    active: bool,
    created_at: DateTime<Utc>,
    world_name: Option<String>,
    world_slug: Option<String>,
    image_url: Option<String>,
    thumbhash: Option<String>,
    capture_count: Option<i64>,
}

impl From<SceneWithWorldRow> for SceneWithWorld {
    fn from(row: SceneWithWorldRow) -> Self {
        Self {
            scene: Scene {
                id: row.id,
                name: row.name,
                slug: row.slug,
                description: row.description,
                world_id: row.world_id,
                x: row.x,
                y: row.y,
                z: row.z,
                pitch: row.pitch,
                yaw: row.yaw,
                dimension: row.dimension,
                time_of_day_ticks: row.time_of_day_ticks,
                weather: row.weather,
                weather_intensity: row.weather_intensity,
                moon_phase: row.moon_phase,
                biome: row.biome,
                definition_json: row.definition_json,
                active: row.active,
                created_at: row.created_at,
            },
            world_name: row.world_name,
            image_url: row.image_url,
            thumbhash: row.thumbhash,
            capture_count: row.capture_count.unwrap_or(0),
            world_slug: row.world_slug,
        }
    }
}

pub struct SceneRepo;

impl SceneRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<Scene>> {
        sqlx::query_as!(Scene, "SELECT * FROM scenes WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find scene '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<Scene> {
        Self::find_by_id(executor, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Scene '{}' not found", id)))
    }

    /// List all active scenes
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_active(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<Scene>> {
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE active = TRUE ORDER BY name"
        )
        .fetch_all(executor)
        .await
        .context("failed to list active scenes")?;

        debug!(count = scenes.len(), "Listed active scenes");
        Ok(scenes)
    }

    /// List scenes by world
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_world(
        executor: impl sqlx::PgExecutor<'_>,
        world_id: &str,
    ) -> AppResult<Vec<Scene>> {
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE world_id = $1 AND active = TRUE ORDER BY name",
            world_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list scenes for world '{}'", world_id))?;

        debug!(count = scenes.len(), "Listed scenes for world");
        Ok(scenes)
    }

    /// Find active scenes by slug (scenes can share slugs across worlds)
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_active_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<Vec<Scene>> {
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE slug = $1 AND active = TRUE",
            slug
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to find scenes by slug '{}'", slug))?;

        Ok(scenes)
    }

    /// Find active scene by slug and world_id (unique)
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_slug_and_world(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
        world_id: &str,
    ) -> AppResult<Option<Scene>> {
        sqlx::query_as!(
            Scene,
            "SELECT * FROM scenes WHERE slug = $1 AND world_id = $2 AND active = TRUE",
            slug,
            world_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to find scene '{}' in world '{}'",
            slug, world_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(executor), level = "debug")]
    pub async fn get_by_slug_and_world(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
        world_id: &str,
    ) -> AppResult<Scene> {
        Self::find_by_slug_and_world(executor, slug, world_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Scene '{}' not found in this world", slug)))
    }

    /// Check if a scene slug exists in a world (active only)
    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_slug_in_world(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
        world_id: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            "SELECT 1 as one FROM scenes WHERE world_id = $1 AND slug = $2 AND active = TRUE",
            world_id,
            slug
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to check scene existence '{}' in world '{}'",
            slug, world_id
        ))?;

        Ok(result.is_some())
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &CreateSceneRequest,
    ) -> AppResult<Scene> {
        sqlx::query_as!(
            Scene,
            r#"
            INSERT INTO scenes (
                id, name, slug, world_id, x, y, z, pitch, yaw,
                dimension, time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
                active, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, TRUE, now())
            RETURNING *
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
        .fetch_one(executor)
        .await
        .context(format!("failed to create scene '{}'", req.slug))
        .map_err(Into::into)
    }

    #[instrument(skip(executor, req), level = "debug")]
    pub async fn update(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &UpdateSceneRequest,
    ) -> AppResult<Scene> {
        sqlx::query_as!(
            Scene,
            r#"
            UPDATE scenes
            SET x = $1, y = $2, z = $3, pitch = $4, yaw = $5,
                dimension = $6, time_of_day_ticks = $7, weather = $8,
                weather_intensity = $9, moon_phase = $10, biome = $11
            WHERE id = $12
            RETURNING *
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
        .fetch_one(executor)
        .await
        .context(format!("failed to update scene '{}'", id))
        .map_err(Into::into)
    }

    /// Disable a scene (soft delete)
    #[instrument(skip(executor), level = "debug")]
    pub async fn disable(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
        world_id: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = FALSE WHERE slug = $1 AND world_id = $2 AND active = TRUE",
            slug,
            world_id
        )
        .execute(executor)
        .await
        .context(format!(
            "failed to disable scene '{}' in world '{}'",
            slug, world_id
        ))?;

        Ok(result.rows_affected() > 0)
    }

    /// Reactivate a disabled scene
    #[instrument(skip(executor), level = "debug")]
    pub async fn reactivate(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = TRUE WHERE id = $1 AND active = FALSE",
            id
        )
        .execute(executor)
        .await
        .context(format!("failed to reactivate scene '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// List all scenes (including inactive) for admin dashboard.
    ///
    /// Includes a preview thumbnail per scene, preferring the vanilla shader's
    /// latest capture, then falling back to the most-downloaded shader's capture.
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_all(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<SceneWithWorld>> {
        let rows = sqlx::query_as!(
            SceneWithWorldRow,
            r#"
            WITH scene_captures_ranked AS (
                SELECT
                    c.scene_id,
                    c.image_url,
                    c.thumbhash,
                    ROW_NUMBER() OVER (
                        PARTITION BY c.scene_id
                        ORDER BY
                            -- Prefer vanilla shader (slug = 'vanilla')
                            CASE WHEN sh.slug = 'vanilla' THEN 0 ELSE 1 END,
                            -- Then by most downloaded shader
                            COALESCE(sh.upstream_downloads, 0) DESC,
                            -- Then most recent capture
                            c.captured_at DESC NULLS LAST
                    ) AS rn
                FROM captures c
                JOIN shader_versions sv ON sv.id = c.shader_version_id
                JOIN shaders sh ON sh.id = sv.shader_id
                WHERE c.status = 'completed' AND c.image_url IS NOT NULL
            ),
            scene_counts AS (
                SELECT scene_id, COUNT(*) AS capture_count
                FROM captures
                WHERE status = 'completed'
                GROUP BY scene_id
            )
            SELECT
                sc.id, sc.name, sc.slug, sc.description, sc.world_id,
                sc.x, sc.y, sc.z, sc.pitch, sc.yaw,
                sc.dimension, sc.time_of_day_ticks, sc.weather, sc.weather_intensity,
                sc.moon_phase, sc.biome, sc.definition_json, sc.active, sc.created_at,
                w.name as world_name,
                w.slug as world_slug,
                cr.image_url,
                cr.thumbhash,
                cnt.capture_count
            FROM scenes sc
            LEFT JOIN worlds w ON sc.world_id = w.id
            LEFT JOIN scene_captures_ranked cr ON cr.scene_id = sc.id AND cr.rn = 1
            LEFT JOIN scene_counts cnt ON cnt.scene_id = sc.id
            ORDER BY sc.name
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to list all scenes")?;

        let scenes: Vec<SceneWithWorld> = rows.into_iter().map(Into::into).collect();
        debug!(count = scenes.len(), "Listed all scenes");
        Ok(scenes)
    }

    /// Update scene metadata (name/description only)
    #[instrument(skip(executor, req), level = "debug")]
    pub async fn update_metadata(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        req: &UpdateSceneMetadataRequest,
    ) -> AppResult<Scene> {
        sqlx::query_as!(
            Scene,
            r#"
            UPDATE scenes SET
                name = COALESCE($1, name),
                description = COALESCE($2, description)
            WHERE id = $3
            RETURNING *
            "#,
            req.name,
            req.description,
            id
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to update scene metadata '{}'", id))
        .map_err(Into::into)
    }

    /// Batch disable scenes by slugs within a world
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_disable(
        executor: impl sqlx::PgExecutor<'_>,
        slugs: &[String],
        world_id: &str,
    ) -> AppResult<u64> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = FALSE WHERE slug = ANY($1) AND world_id = $2 AND active = TRUE",
            slugs,
            world_id
        )
        .execute(executor)
        .await
        .context("failed to batch disable scenes")?;

        debug!(count = result.rows_affected(), "Batch disabled scenes");
        Ok(result.rows_affected())
    }

    /// Disable a scene by ID (for admin)
    #[instrument(skip(executor), level = "debug")]
    pub async fn disable_by_id(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = FALSE WHERE id = $1 AND active = TRUE",
            id
        )
        .execute(executor)
        .await
        .context(format!("failed to disable scene '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }
}
