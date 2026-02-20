use std::collections::HashMap;

use anyhow::Context;
use tracing::{debug, instrument};

use crate::error::{AppError, AppResult};
use crate::graphql::types::common::Visibility;
use crate::graphql::types::connection::CursorPage;
use crate::id::{SceneId, SceneVersionId};
use chrono::{DateTime, Utc};

use crate::models::{
    CreateSceneRequest, Scene, SceneListItem, ScenePreset, SceneVersion,
    UpdateSceneMetadataRequest, UpdateSceneRequest,
};

/// Helper struct for joined scene query (includes latest version via lateral join)
struct SceneListAdminRow {
    id: String,
    name: String,
    slug: String,
    description: Option<String>,
    dimension: String,
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
    version_minecraft_version: Option<String>,
    version_package_url: Option<String>,
    version_package_hash: Option<String>,
    version_package_size_bytes: Option<i64>,
    version_fov: Option<i32>,
    version_render_distance: Option<i32>,
    version_created_at: Option<DateTime<Utc>>,
    // Enrichment
    image_path: Option<String>,
    thumbhash: Option<String>,
    capture_count: Option<i64>,
}

impl TryFrom<SceneListAdminRow> for SceneListItem {
    type Error = AppError;

    fn try_from(row: SceneListAdminRow) -> Result<Self, Self::Error> {
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
                minecraft_version: row.version_minecraft_version,
                package_url: row.version_package_url,
                package_hash: row.version_package_hash,
                package_size_bytes: row.version_package_size_bytes,
                fov: row.version_fov.unwrap_or(70),
                render_distance: row.version_render_distance.unwrap_or(16),
                created_at: row.version_created_at.unwrap_or(row.created_at),
            })
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "scene '{}' has no version (post-migration invariant violated)",
                    row.id
                ))
            })?;
        Ok(Self {
            scene: Scene {
                id: SceneId(row.id),
                name: row.name,
                slug: row.slug,
                description: row.description,
                dimension: row.dimension,
                active: row.active,
                created_at: row.created_at,
            },
            version,
            tags: vec![],
            image_path: row.image_path,
            thumbhash: row.thumbhash,
            capture_count: row.capture_count.unwrap_or(0),
        })
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

    /// Cursor-paginated list of scenes (for GraphQL).
    /// Ordered by created_at DESC, id DESC.
    #[instrument(skip(pool), level = "debug")]
    pub async fn list_cursor(
        pool: &crate::db::DbPool,
        first: i32,
        after: Option<(DateTime<Utc>, String)>,
        visibility: Visibility,
    ) -> AppResult<CursorPage<Scene>> {
        use sqlx::QueryBuilder;

        let limit = first.clamp(1, 100) as i64;

        let mut qb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM scenes WHERE TRUE");

        match visibility {
            Visibility::Exclude => qb.push(" AND active = TRUE"),
            Visibility::Include => &mut qb, // no filter
            Visibility::Only => qb.push(" AND active = FALSE"),
        };

        if let Some((ref cursor_ts, ref cursor_id)) = after {
            qb.push(" AND (created_at, id) < (");
            qb.push_bind(*cursor_ts);
            qb.push(", ");
            qb.push_bind(cursor_id.as_str());
            qb.push(")");
        }

        qb.push(" ORDER BY created_at DESC, id DESC LIMIT ");
        qb.push_bind(limit + 1);

        let items: Vec<Scene> = qb
            .build_query_as()
            .fetch_all(pool)
            .await
            .context("failed to list scenes (cursor)")?;

        let has_next_page = items.len() as i64 > limit;
        let items: Vec<Scene> = items.into_iter().take(limit as usize).collect();

        let mut cqb: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM scenes WHERE TRUE");

        match visibility {
            Visibility::Exclude => cqb.push(" AND active = TRUE"),
            Visibility::Include => &mut cqb,
            Visibility::Only => cqb.push(" AND active = FALSE"),
        };

        let total_count: i64 = cqb
            .build_query_scalar()
            .fetch_one(pool)
            .await
            .context("failed to count scenes")?;

        Ok(CursorPage {
            items,
            has_next_page,
            has_previous_page: after.is_some(),
            total_count,
        })
    }

    /// Find active scenes by slug (globally unique now)
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

    /// Check if an active scene with the given slug already exists
    #[instrument(skip(executor), level = "debug")]
    pub async fn exists_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query_scalar!(
            "SELECT 1 as one FROM scenes WHERE slug = $1 AND active = TRUE",
            slug
        )
        .fetch_optional(executor)
        .await
        .context(format!("failed to check scene existence '{}'", slug))?;

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

        let scene = sqlx::query_as!(
            Scene,
            r#"
            INSERT INTO scenes (id, name, slug, dimension, active, created_at)
            VALUES ($1, $2, $3, $4, TRUE, now())
            RETURNING *
            "#,
            id,
            req.name,
            req.slug,
            req.dimension,
        )
        .fetch_one(&mut *tx)
        .await
        .context(format!("failed to create scene '{}'", req.slug))?;

        let version_id = crate::id::generate_id();
        let version =
            SceneVersionRepo::create_inner(&mut *tx, &version_id, scene.id.as_ref(), req).await?;

        tx.commit()
            .await
            .context("failed to commit scene creation")?;
        Ok((scene, version))
    }

    /// Update a scene by creating a new scene_version with the new config.
    ///
    /// Only sets positioning/camera/environment fields from the mod's update request.
    /// Package and rendering fields (fov, render_distance) use DB defaults and are
    /// managed through a separate scene package upload flow.
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
        let version_id = crate::id::generate_id();
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

        tx.commit().await.context("failed to commit scene update")?;
        Ok((scene, version))
    }

    /// Disable an active scene by slug (soft delete)
    #[instrument(skip(executor), level = "debug")]
    pub async fn disable_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        slug: &str,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = FALSE WHERE slug = $1 AND active = TRUE",
            slug,
        )
        .execute(executor)
        .await
        .context(format!("failed to disable scene '{}'", slug))?;

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
    pub async fn list_all(executor: impl sqlx::PgExecutor<'_>) -> AppResult<Vec<SceneListItem>> {
        let rows = sqlx::query_as!(
            SceneListAdminRow,
            r#"
            WITH scene_captures_ranked AS (
                SELECT
                    c.scene_id,
                    c.image_path,
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
                WHERE c.status = 'completed' AND c.image_path IS NOT NULL
            ),
            scene_counts AS (
                SELECT scene_id, COUNT(*) AS capture_count
                FROM captures
                WHERE status = 'completed'
                GROUP BY scene_id
            )
            SELECT
                sc.id, sc.name, sc.slug, sc.description,
                sc.dimension, sc.active, sc.created_at,
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
                lsv.minecraft_version AS version_minecraft_version,
                lsv.package_url AS version_package_url,
                lsv.package_hash AS version_package_hash,
                lsv.package_size_bytes AS version_package_size_bytes,
                lsv.fov AS version_fov,
                lsv.render_distance AS version_render_distance,
                lsv.created_at AS version_created_at,
                cr.image_path AS "image_path?",
                cr.thumbhash AS "thumbhash?",
                cnt.capture_count
            FROM scenes sc
            LEFT JOIN LATERAL (
                SELECT * FROM scene_versions sv2
                WHERE sv2.scene_id = sc.id
                ORDER BY sv2.created_at DESC, sv2.id DESC
                LIMIT 1
            ) lsv ON TRUE
            LEFT JOIN scene_captures_ranked cr ON cr.scene_id = sc.id AND cr.rn = 1
            LEFT JOIN scene_counts cnt ON cnt.scene_id = sc.id
            ORDER BY sc.name
            "#
        )
        .fetch_all(executor)
        .await
        .context("failed to list all scenes")?;

        let scenes: Vec<SceneListItem> = rows
            .into_iter()
            .map(SceneListItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;
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

    /// Batch disable scenes by slugs
    #[instrument(skip(executor), level = "debug")]
    pub async fn batch_disable(
        executor: impl sqlx::PgExecutor<'_>,
        slugs: &[String],
    ) -> AppResult<u64> {
        let result = sqlx::query!(
            "UPDATE scenes SET active = FALSE WHERE slug = ANY($1) AND active = TRUE",
            slugs,
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

    /// Create a new scene_version with all fields including package data.
    /// Used by the upload confirmation flow.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(executor), level = "debug")]
    pub async fn create_with_package(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        scene_id: &str,
        x: f64,
        y: f64,
        z: f64,
        pitch: f64,
        yaw: f64,
        time_of_day_ticks: i32,
        weather: &str,
        weather_intensity: f64,
        moon_phase: Option<i32>,
        biome: Option<&str>,
        minecraft_version: &str,
        package_url: &str,
        package_hash: &str,
        package_size_bytes: i64,
        fov: i32,
        render_distance: i32,
    ) -> AppResult<SceneVersion> {
        sqlx::query_as!(
            SceneVersion,
            r#"
            INSERT INTO scene_versions
                (id, scene_id, x, y, z, pitch, yaw,
                 time_of_day_ticks, weather, weather_intensity, moon_phase, biome,
                 minecraft_version, package_url, package_hash, package_size_bytes,
                 fov, render_distance, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, now())
            RETURNING *
            "#,
            id,
            scene_id,
            x,
            y,
            z,
            pitch,
            yaw,
            time_of_day_ticks,
            weather,
            weather_intensity,
            moon_phase,
            biome,
            minecraft_version,
            package_url,
            package_hash,
            package_size_bytes,
            fov,
            render_distance,
        )
        .fetch_one(executor)
        .await
        .context(format!(
            "failed to create scene version with package for '{}'",
            scene_id
        ))
        .map_err(Into::into)
    }

    /// Internal helper: insert a new scene_version row from a CreateSceneRequest.
    ///
    /// Package fields (package_url, package_hash, package_size_bytes) and rendering
    /// overrides (fov, render_distance) are not set here — they use DB defaults.
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

pub struct ScenePresetRepo;

impl ScenePresetRepo {
    /// List all presets for a scene, ordered by sort_order
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_scene(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<Vec<ScenePreset>> {
        let presets = sqlx::query_as!(
            ScenePreset,
            r#"
            SELECT * FROM scene_presets
            WHERE scene_id = $1
            ORDER BY sort_order ASC, name ASC
            "#,
            scene_id
        )
        .fetch_all(executor)
        .await
        .context(format!("failed to list presets for scene '{}'", scene_id))?;

        debug!(count = presets.len(), "Listed scene presets");
        Ok(presets)
    }

    /// Batch load presets for multiple scenes
    #[instrument(skip(executor), level = "debug")]
    pub async fn list_by_scenes(
        executor: impl sqlx::PgExecutor<'_>,
        scene_ids: &[String],
    ) -> AppResult<HashMap<String, Vec<ScenePreset>>> {
        let presets = sqlx::query_as!(
            ScenePreset,
            r#"
            SELECT * FROM scene_presets
            WHERE scene_id = ANY($1)
            ORDER BY scene_id, sort_order ASC, name ASC
            "#,
            scene_ids
        )
        .fetch_all(executor)
        .await
        .context("failed to batch list presets for scenes")?;

        debug!(count = presets.len(), "Batch listed scene presets");
        let mut map: HashMap<String, Vec<ScenePreset>> = HashMap::new();
        for preset in presets {
            map.entry(preset.scene_id.0.clone())
                .or_default()
                .push(preset);
        }
        Ok(map)
    }

    /// Find a single preset by ID
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_id(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
    ) -> AppResult<Option<ScenePreset>> {
        sqlx::query_as!(ScenePreset, "SELECT * FROM scene_presets WHERE id = $1", id)
            .fetch_optional(executor)
            .await
            .context(format!("failed to find preset '{}'", id))
            .map_err(Into::into)
    }

    /// Create a new scene preset
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(executor), level = "debug")]
    pub async fn create(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        scene_id: &str,
        name: &str,
        slug: &str,
        time_of_day_ticks: i32,
        weather: &str,
        weather_intensity: f64,
        moon_phase: Option<i32>,
        sort_order: i32,
    ) -> AppResult<ScenePreset> {
        sqlx::query_as!(
            ScenePreset,
            r#"
            INSERT INTO scene_presets (id, scene_id, name, slug, time_of_day_ticks, weather, weather_intensity, moon_phase, sort_order, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), now())
            RETURNING *
            "#,
            id,
            scene_id,
            name,
            slug,
            time_of_day_ticks,
            weather,
            weather_intensity,
            moon_phase,
            sort_order,
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to create preset '{}' for scene '{}'", name, scene_id))
        .map_err(Into::into)
    }

    /// Delete a preset by ID
    #[instrument(skip(executor), level = "debug")]
    pub async fn delete(executor: impl sqlx::PgExecutor<'_>, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM scene_presets WHERE id = $1", id)
            .execute(executor)
            .await
            .context(format!("failed to delete preset '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// Find a preset by scene_id + preset slug.
    #[instrument(skip(executor), level = "debug")]
    pub async fn find_by_slug(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
        slug: &str,
    ) -> AppResult<Option<ScenePreset>> {
        sqlx::query_as!(
            ScenePreset,
            "SELECT * FROM scene_presets WHERE scene_id = $1 AND slug = $2",
            scene_id,
            slug
        )
        .fetch_optional(executor)
        .await
        .context(format!(
            "failed to find preset '{}' for scene '{}'",
            slug, scene_id
        ))
        .map_err(Into::into)
    }

    /// Count the number of presets for a scene.
    #[instrument(skip(executor), level = "debug")]
    pub async fn count_by_scene(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<i64> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM scene_presets WHERE scene_id = $1",
            scene_id
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to count presets for scene '{}'", scene_id))?;

        Ok(count.unwrap_or(0))
    }

    /// Get the max sort_order for presets in a scene (for appending new presets).
    #[instrument(skip(executor), level = "debug")]
    pub async fn max_sort_order(
        executor: impl sqlx::PgExecutor<'_>,
        scene_id: &str,
    ) -> AppResult<i32> {
        let max = sqlx::query_scalar!(
            "SELECT MAX(sort_order) FROM scene_presets WHERE scene_id = $1",
            scene_id
        )
        .fetch_one(executor)
        .await
        .context(format!(
            "failed to get max sort_order for scene '{}'",
            scene_id
        ))?;

        Ok(max.unwrap_or(-1))
    }

    /// Update a preset's fields. Only non-None fields are updated.
    #[instrument(skip(executor), level = "debug")]
    pub async fn update(
        executor: impl sqlx::PgExecutor<'_>,
        id: &str,
        name: Option<&str>,
        time_of_day_ticks: Option<i32>,
        weather: Option<&str>,
        weather_intensity: Option<f64>,
        moon_phase: Option<i32>,
    ) -> AppResult<ScenePreset> {
        sqlx::query_as!(
            ScenePreset,
            r#"
            UPDATE scene_presets SET
                name = COALESCE($2, name),
                time_of_day_ticks = COALESCE($3, time_of_day_ticks),
                weather = COALESCE($4, weather),
                weather_intensity = COALESCE($5, weather_intensity),
                moon_phase = COALESCE($6, moon_phase),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
            id,
            name,
            time_of_day_ticks,
            weather,
            weather_intensity,
            moon_phase,
        )
        .fetch_one(executor)
        .await
        .context(format!("failed to update preset '{}'", id))
        .map_err(Into::into)
    }

    /// Reorder presets by setting sort_order based on position in the given ID list.
    /// Uses a transaction to update all presets atomically.
    #[instrument(skip(pool), level = "debug")]
    pub async fn reorder(
        pool: &crate::db::DbPool,
        scene_id: &str,
        preset_ids: &[String],
    ) -> AppResult<Vec<ScenePreset>> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        for (idx, preset_id) in preset_ids.iter().enumerate() {
            sqlx::query!(
                "UPDATE scene_presets SET sort_order = $1, updated_at = now() WHERE id = $2 AND scene_id = $3",
                idx as i32,
                preset_id,
                scene_id,
            )
            .execute(&mut *tx)
            .await
            .context(format!("failed to update sort_order for preset '{}'", preset_id))?;
        }

        let presets = sqlx::query_as!(
            ScenePreset,
            "SELECT * FROM scene_presets WHERE scene_id = $1 ORDER BY sort_order ASC, name ASC",
            scene_id
        )
        .fetch_all(&mut *tx)
        .await
        .context(format!(
            "failed to list presets after reorder for scene '{}'",
            scene_id
        ))?;

        tx.commit()
            .await
            .context("failed to commit preset reorder")?;
        Ok(presets)
    }
}
