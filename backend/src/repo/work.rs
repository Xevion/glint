use anyhow::Context;
use sqlx::Executor;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::id::{
    SceneId, ScenePresetId, SceneVersionId, ShaderId, ShaderVersionId, ShaderVersionProfileId,
};
use crate::models::WorkItem;
use crate::repo::work_ordering;

/// Safety cap on total items returned to prevent absurdly large responses,
/// even when shader_limit would produce more.
const MAX_ITEMS: i64 = 5000;

pub struct WorkRepo;

impl WorkRepo {
    /// Compute the list of (shader_version, scene, preset, profile) tuples that
    /// still need captures, ordered for optimal capture execution.
    ///
    /// SQL handles filtering and shader-count batching (selecting the top N most
    /// popular shaders globally). Rust post-processing refines scene ordering
    /// within each profile pass using spatial clustering and nearest-neighbor
    /// traversal.
    ///
    /// `shader_limit` controls how many distinct shader versions are included
    /// (ordered by popularity). This guarantees complete shader units in each
    /// batch — no partial shaders.
    ///
    /// JIT compilation is disabled because the planner's inflated cost estimate
    /// (cross-join fanout) triggers LLVM JIT that takes ~750ms on a query that
    /// executes in <30ms.
    #[instrument(skip(pool), level = "debug")]
    pub async fn get_work_items(
        pool: &DbPool,
        shader_limit: i64,
        force: bool,
        shaders_filter: Option<String>,
        scenes_filter: Option<String>,
    ) -> AppResult<Vec<WorkItem>> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        tx.execute("SET LOCAL jit = off")
            .await
            .context("failed to disable JIT")?;

        let items = sqlx::query_as!(
            WorkItem,
            r#"
            WITH has_captures AS (
                SELECT DISTINCT shader_version_id
                FROM captures
                WHERE status = 'completed'
            ),
            best_captures AS (
                SELECT DISTINCT ON (shader_version_id, scene_id, preset_id, profile_id)
                    shader_version_id, scene_id, preset_id, profile_id, freshness
                FROM captures_with_freshness
                WHERE status IN ('completed', 'uploading')
                ORDER BY shader_version_id, scene_id, preset_id, profile_id, captured_at DESC
            ),
            eligible AS (
                SELECT
                    tm.shader_version_id,
                    tm.scene_id,
                    tm.preset_id,
                    tm.profile_id,
                    sh.id AS shader_id,
                    COALESCE(sh.upstream_downloads, 0) AS downloads,
                    sh.name AS shader_name,
                    hc.shader_version_id IS NOT NULL AS has_existing_captures
                FROM capture_target_matrix tm
                JOIN latest_shader_versions sv ON sv.id = tm.shader_version_id
                JOIN shaders sh ON sh.id = sv.shader_id
                JOIN scenes sc ON sc.id = tm.scene_id
                LEFT JOIN has_captures hc ON hc.shader_version_id = tm.shader_version_id
                LEFT JOIN best_captures bc
                    ON bc.shader_version_id = tm.shader_version_id
                    AND bc.scene_id = tm.scene_id
                    AND (bc.preset_id IS NOT DISTINCT FROM tm.preset_id)
                    AND (bc.profile_id IS NOT DISTINCT FROM tm.profile_id)
                WHERE ($2 OR sv.capture_failure_count < 3)
                  AND ($2 OR bc.shader_version_id IS NULL OR bc.freshness != 'fresh')
                  AND ($3::text IS NULL OR sh.slug = ANY(string_to_array($3, ',')))
                  AND ($4::text IS NULL OR sc.slug = ANY(string_to_array($4, ',')))
            ),
            ranked_shaders AS (
                SELECT DISTINCT shader_version_id,
                    DENSE_RANK() OVER (
                        ORDER BY has_existing_captures ASC,
                                 downloads DESC,
                                 shader_name ASC
                    ) AS shader_rank
                FROM eligible
            ),
            selected AS (
                SELECT e.*
                FROM eligible e
                JOIN ranked_shaders rs
                    ON rs.shader_version_id = e.shader_version_id
                WHERE rs.shader_rank <= $1
            )
            SELECT
                s.shader_version_id AS "shader_version_id!: ShaderVersionId",
                sh.id AS "shader_id!: ShaderId",
                sh.slug AS "shader_slug!",
                sh.name AS "shader_name!",
                sv.version AS "version!",
                sv.download_url,
                sv.file_hash,
                s.scene_id AS "scene_id!: SceneId",
                sc.slug AS "scene_slug!",
                sc.name AS "scene_name!",
                sc.dimension AS "scene_dimension!",
                lsv.x AS "scene_x!",
                lsv.y AS "scene_y!",
                lsv.z AS "scene_z!",
                lsv.yaw AS "scene_yaw!",
                lsv.pitch AS "scene_pitch!",
                lsv.time_of_day_ticks AS "scene_time_of_day_ticks!",
                lsv.weather AS "scene_weather!",
                lsv.weather_intensity AS "scene_weather_intensity!",
                lsv.moon_phase AS scene_moon_phase,
                lsv.biome AS scene_biome,
                s.preset_id AS "preset_id: ScenePresetId",
                sp.name AS preset_name,
                sp.slug AS preset_slug,
                lsv.package_url,
                lsv.package_hash,
                lsv.package_size_bytes,
                lsv.fov AS "scene_fov!",
                lsv.render_distance AS "scene_render_distance!",
                lsv.id AS "scene_version_id: SceneVersionId",
                s.profile_id AS "profile_id: ShaderVersionProfileId",
                svp.name AS "profile_name?"
            FROM selected s
            JOIN latest_shader_versions sv ON sv.id = s.shader_version_id
            JOIN shaders sh ON sh.id = s.shader_id
            JOIN scenes sc ON sc.id = s.scene_id
            JOIN latest_scene_versions lsv ON lsv.scene_id = sc.id
            LEFT JOIN scene_presets sp ON sp.id = s.preset_id
            LEFT JOIN shader_version_profiles svp ON svp.id = s.profile_id
            ORDER BY
                s.has_existing_captures ASC,
                s.downloads DESC,
                sh.name ASC,
                svp.sort_order ASC NULLS FIRST
            LIMIT $5
            "#,
            shader_limit,
            force,
            shaders_filter as Option<String>,
            scenes_filter as Option<String>,
            MAX_ITEMS,
        )
        .fetch_all(&mut *tx)
        .await
        .context("failed to fetch work items")?;

        tx.commit().await.context("failed to commit transaction")?;

        // Post-process: spatial clustering + nearest-neighbor scene ordering
        let ordered = work_ordering::optimize_execution_order(items);

        debug!(count = ordered.len(), "Fetched and ordered work items");
        Ok(ordered)
    }
}
