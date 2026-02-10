use std::collections::HashMap;

use anyhow::Context;
use tracing::{debug, instrument, warn};

use crate::error::{AppError, AppResult};
use crate::id::{SceneId, SceneVersionId, WorldId};
use chrono::{DateTime, Utc};

use crate::models::{
    CreateSceneRequest, Scene, SceneVersion, SceneWithWorld, UpdateSceneMetadataRequest,
    UpdateSceneRequest,
};

/// Helper struct for joined scene/world query (includes latest version via lateral join)
struct SceneWithWorldRow {
    id: String,
    name: String,
    slug: String,
    description: Option<String>,
    world_id: String,
    dimension: String,
    parent_scene_id: Option<String>,
    active: bool,
    created_at: DateTime<Utc>,
    // Latest version fields (nullable — scene might have no versions yet)
    version_id: Option<String>,
    version_x: Option<f64>,
    version_y: Option<f64>,
    version_z: Option<f64>,
    version_pitch: Option<f64>,
    version_yaw: Option<f64>,
    version_time_of_day_ticks: Option<i32>,
    version_weather: Option<String>,
    version_weather_intensity: Option<f64>,
    version_moon_phase: Option<i32>,
    version_biome: Option<String>,
    version_created_at: Option<DateTime<Utc>>,
    // World + enrichment
    world_name: Option<String>,
    world_slug: Option<String>,
    image_url: Option<String>,
    thumbhash: Option<String>,
    capture_count: Option<i64>,
}

impl From<SceneWithWorldRow> for SceneWithWorld {
    fn from(row: SceneWithWorldRow) -> Self {
        let version = row
            .version_id
            .map(|vid| SceneVersion {
                id: SceneVersionId(vid),
                scene_id: SceneId(row.id.clone()),
                x: row.version_x.unwrap_or(0.0),
                y: row.version_y.unwrap_or(0.0),
                z: row.version_z.unwrap_or(0.0),
                pitch: row.version_pitch.unwrap_or(0.0),
                yaw: row.version_yaw.unwrap_or(0.0),
                time_of_day_ticks: row.version_time_of_day_ticks.unwrap_or(0),
                weather: row.version_weather.unwrap_or_default(),
                weather_intensity: row.version_weather_intensity.unwrap_or(0.0),
                moon_phase: row.version_moon_phase,
                biome: row.version_biome,
                created_at: row.version_created_at.unwrap_or(row.created_at),
            })
            .expect("scene must have at least one version (post-migration invariant)");
        Self {
            scene: Scene {
                id: SceneId(row.id),
                name: row.name,
                slug: row.slug,
                description: row.description,
                world_id: WorldId(row.world_id),
                dimension: row.dimension,
                parent_scene_id: row.parent_scene_id,
                active: row.active,
                created_at: row.created_at,
            },
            version,
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

    /// Create a scene and its initial version in a single transaction.
    /// Returns both the scene and its initial version.
    #[instrument(skip(pool, req), level = "debug")]
    pub async fn create(
        pool: &crate::db::DbPool,
        id: &str,
        req: &CreateSceneRequest,
    ) -> AppResult<(Scene, SceneVersion)> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        // Enforce max derivative depth of 1 (no grandchildren: A→B is ok, A→B→C is not)
        if let Some(parent_id) = &req.parent_scene_id {
            let parent_has_parent = sqlx::query_scalar!(
                "SELECT parent_scene_id FROM scenes WHERE id = $1",
                parent_id
            )
            .fetch_optional(&mut *tx)
            .await
            .context("failed to check parent scene")?
            .flatten();

            if parent_has_parent.is_some() {
                return Err(AppError::BadRequest(
                    "Cannot create a derivative of a derivative scene".into(),
                ));
            }
        }

        let scene = sqlx::query_as!(
            Scene,
            r#"
            INSERT INTO scenes (id, name, slug, world_id, dimension, parent_scene_id, active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE, now())
            RETURNING *
            "#,
            id,
            req.name,
            req.slug,
            req.world_id.as_ref(),
            req.dimension,
            req.parent_scene_id,
        )
        .fetch_one(&mut *tx)
        .await
        .context(format!("failed to create scene '{}'", req.slug))?;

        let version_id = uuid::Uuid::new_v4().to_string();
        let version =
            SceneVersionRepo::create_inner(&mut *tx, &version_id, scene.id.as_ref(), req).await?;

        tx.commit()
            .await
            .context("failed to commit scene creation")?;
        Ok((scene, version))
    }

    /// Update a scene by creating a new scene_version with the new config.
    /// Also cascades to derivatives: any scene with parent_scene_id = id
    /// gets a new version with the same config.
    #[instrument(skip(pool, req), level = "debug")]
    pub async fn update(
        pool: &crate::db::DbPool,
        id: &str,
        req: &UpdateSceneRequest,
    ) -> AppResult<(Scene, SceneVersion)> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        // Fetch scene (dimension is immutable — set at creation, never updated)
        let scene = sqlx::query_as!(Scene, r#"SELECT * FROM scenes WHERE id = $1"#, id)
            .fetch_one(&mut *tx)
            .await
            .context(format!("failed to find scene '{}'", id))?;

        // Create new version
        let version_id = uuid::Uuid::new_v4().to_string();
        let version = sqlx::query_as!(
            SceneVersion,
            r#"
            INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity, moon_phase, biome, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, now())
            RETURNING *
            "#,
            version_id,
            id,
            req.position.x,
            req.position.y,
            req.position.z,
            req.camera.pitch,
            req.camera.yaw,
            req.time_of_day,
            req.weather,
            req.weather_intensity,
            req.moon_phase,
            req.biome,
        )
        .fetch_one(&mut *tx)
        .await
        .context(format!("failed to create scene version for '{}'", id))?;

        // Cascade to derivative scenes (children with parent_scene_id = id)
        let derivatives: Vec<String> = sqlx::query_scalar!(
            "SELECT id FROM scenes WHERE parent_scene_id = $1 AND active = TRUE",
            id
        )
        .fetch_all(&mut *tx)
        .await
        .context("failed to list derivative scenes")?;

        // For each derivative, create a new version that inherits position/camera
        // from the parent but preserves the derivative's own environment overrides
        // (time_of_day, weather, etc.) from its latest version.
        for child_id in &derivatives {
            let child_version_id = uuid::Uuid::new_v4().to_string();
            let result = sqlx::query!(
                r#"
                INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity, moon_phase, biome, created_at)
                SELECT $1, $2, $3, $4, $5, $6, $7,
                    child_v.time_of_day_ticks,
                    child_v.weather,
                    child_v.weather_intensity,
                    child_v.moon_phase,
                    child_v.biome,
                    now()
                FROM (
                    SELECT * FROM scene_versions
                    WHERE scene_id = $2
                    ORDER BY created_at DESC, id DESC
                    LIMIT 1
                ) child_v
                "#,
                child_version_id,
                child_id,
                req.position.x,
                req.position.y,
                req.position.z,
                req.camera.pitch,
                req.camera.yaw,
            )
            .execute(&mut *tx)
            .await
            .context(format!("failed to cascade version to derivative '{}'", child_id))?;

            if result.rows_affected() == 0 {
                warn!(
                    parent_id = id,
                    child_id = child_id.as_str(),
                    "Derivative cascade skipped: child scene has no existing versions"
                );
            }
        }

        if !derivatives.is_empty() {
            debug!(
                count = derivatives.len(),
                "Cascaded version to derivative scenes"
            );
        }

        tx.commit().await.context("failed to commit scene update")?;
        Ok((scene, version))
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
                            CASE WHEN sh.slug = 'vanilla' THEN 0 ELSE 1 END,
                            COALESCE(sh.upstream_downloads, 0) DESC,
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
                sc.dimension, sc.parent_scene_id, sc.active, sc.created_at,
                lsv.id AS version_id,
                lsv.x AS version_x,
                lsv.y AS version_y,
                lsv.z AS version_z,
                lsv.pitch AS version_pitch,
                lsv.yaw AS version_yaw,
                lsv.time_of_day_ticks AS version_time_of_day_ticks,
                lsv.weather AS version_weather,
                lsv.weather_intensity AS version_weather_intensity,
                lsv.moon_phase AS version_moon_phase,
                lsv.biome AS version_biome,
                lsv.created_at AS version_created_at,
                w.name as world_name,
                w.slug as world_slug,
                cr.image_url,
                cr.thumbhash,
                cnt.capture_count
            FROM scenes sc
            LEFT JOIN LATERAL (
                SELECT * FROM scene_versions sv2
                WHERE sv2.scene_id = sc.id
                ORDER BY sv2.created_at DESC, sv2.id DESC
                LIMIT 1
            ) lsv ON TRUE
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

pub struct SceneVersionRepo;

impl SceneVersionRepo {
    /// Get the latest version for a scene
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_latest(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<Option<SceneVersion>> {
        sqlx::query_as!(
            SceneVersion,
            r#"
            SELECT * FROM scene_versions
            WHERE scene_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
            scene_id
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to get latest scene version for '{}'",
            scene_id
        ))
        .map_err(Into::into)
    }

    /// Batch-fetch the latest version for each of the given scene IDs.
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_latest_batch(
        executor: impl sqlx::PgExecutor<'_>,
        scene_ids: &[String],
    ) -> AppResult<HashMap<SceneId, SceneVersion>> {
        let rows = sqlx::query_as!(
            SceneVersion,
            r#"
            SELECT DISTINCT ON (scene_id) *
            FROM scene_versions
            WHERE scene_id = ANY($1)
            ORDER BY scene_id, created_at DESC, id DESC
            "#,
            scene_ids
        )
        .fetch_all(executor)
        .await
        .context("failed to batch-fetch latest scene versions")?;

        Ok(rows.into_iter().map(|v| (v.scene_id.clone(), v)).collect())
    }

    /// Internal helper: insert a new scene_version row from a CreateSceneRequest.
    pub(crate) async fn create_inner(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        scene_id: &str,
        req: &CreateSceneRequest,
    ) -> AppResult<SceneVersion> {
        sqlx::query_as!(
            SceneVersion,
            r#"
            INSERT INTO scene_versions (id, scene_id, x, y, z, pitch, yaw, time_of_day_ticks, weather, weather_intensity, moon_phase, biome, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, now())
            RETURNING *
            "#,
            id,
            scene_id,
            req.position.x,
            req.position.y,
            req.position.z,
            req.camera.pitch,
            req.camera.yaw,
            req.time_of_day,
            req.weather,
            req.weather_intensity,
            req.moon_phase,
            req.biome,
        )
        .fetch_one(executor)
        .await
        .context(format!(
            "failed to create scene version for '{}'",
            scene_id
        ))
        .map_err(Into::into)
    }
}
