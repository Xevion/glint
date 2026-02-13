use anyhow::Context;
use sqlx::Executor;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::id::{
    SceneId, SceneVersionId, ShaderId, ShaderVersionId, ShaderVersionProfileId, WorldId,
    WorldVersionId,
};
use crate::models::WorkItem;

pub struct WorkRepo;

impl WorkRepo {
    /// Compute the list of (shader_version, scene, profile_id) triples that still
    /// need captures. JIT compilation is disabled for this query because the
    /// planner's inflated cost estimate (cross-join fanout) triggers LLVM JIT
    /// that takes ~750ms on a query that executes in <30ms.
    #[instrument(skip(pool), level = "debug")]
    pub async fn get_work_items(
        pool: &DbPool,
        limit: i64,
        force: bool,
        shaders_filter: Option<String>,
        scenes_filter: Option<String>,
    ) -> AppResult<Vec<WorkItem>> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        // Disable JIT for this transaction — the query's estimated cost is
        // inflated by cross-join cardinality, causing Postgres to spend ~750ms
        // on LLVM compilation for a query that runs in <30ms.
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
                SELECT DISTINCT ON (shader_version_id, scene_id, profile_id)
                    shader_version_id, scene_id, profile_id, freshness
                FROM captures_with_freshness
                WHERE status IN ('completed', 'uploading')
                ORDER BY shader_version_id, scene_id, profile_id, captured_at DESC
            )
            SELECT
                tm.shader_version_id AS "shader_version_id!: ShaderVersionId",
                sh.id AS "shader_id!: ShaderId",
                sh.slug AS "shader_slug!",
                sh.name AS "shader_name!",
                sv.version AS "version!",
                sv.download_url,
                sv.file_hash,
                tm.scene_id AS "scene_id!: SceneId",
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
                w.id AS "world_id!: WorldId",
                w.slug AS "world_slug!",
                w.name AS "world_name!",
                lwv.file_url AS world_file_url,
                lwv.file_hash AS world_file_hash,
                lwv.size_bytes AS world_size_bytes,
                lwv.id AS "world_version_id: WorldVersionId",
                lsv.id AS "scene_version_id: SceneVersionId",
                tm.profile_id AS "profile_id: ShaderVersionProfileId",
                svp.name AS "profile_name?"
            FROM capture_target_matrix tm
            JOIN latest_shader_versions sv ON sv.id = tm.shader_version_id
            JOIN shaders sh ON sh.id = sv.shader_id
            JOIN scenes sc ON sc.id = tm.scene_id
            JOIN worlds w ON w.id = tm.world_id
            LEFT JOIN latest_world_versions lwv ON lwv.world_id = w.id
            JOIN latest_scene_versions lsv ON lsv.scene_id = sc.id
            LEFT JOIN shader_version_profiles svp ON svp.id = tm.profile_id
            LEFT JOIN has_captures hc ON hc.shader_version_id = tm.shader_version_id
            LEFT JOIN best_captures bc
                ON bc.shader_version_id = tm.shader_version_id
                AND bc.scene_id = tm.scene_id
                AND (bc.profile_id IS NOT DISTINCT FROM tm.profile_id)
            WHERE ($2 OR sv.capture_failure_count < 3)
              AND ($2 OR bc.shader_version_id IS NULL OR bc.freshness != 'fresh')
              AND ($3::text IS NULL OR sh.slug = ANY(string_to_array($3, ',')))
              AND ($4::text IS NULL OR sc.slug = ANY(string_to_array($4, ',')))
            ORDER BY
                hc.shader_version_id IS NOT NULL ASC,
                COALESCE(sh.upstream_downloads, 0) DESC,
                sv.upstream_published_at DESC NULLS LAST,
                sh.name ASC,
                sc.name ASC,
                tm.profile_id NULLS LAST
            LIMIT $1
            "#,
            limit,
            force,
            shaders_filter as Option<String>,
            scenes_filter as Option<String>,
        )
        .fetch_all(&mut *tx)
        .await
        .context("failed to fetch work items")?;

        tx.commit().await.context("failed to commit transaction")?;

        debug!(count = items.len(), "Fetched work items");
        Ok(items)
    }
}
