use anyhow::Context;
use schemars::JsonSchema;
use sqlx::Executor;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::db::DbPool;
use crate::error::AppResult;

/// A single work item: one (shader_version, scene, profile) triple to capture
#[derive(Debug, sqlx::FromRow, serde::Serialize, JsonSchema, TS)]
#[ts(export)]
pub struct WorkItem {
    pub shader_version_id: String,
    pub shader_id: String,
    pub shader_slug: String,
    pub shader_name: String,
    pub version: String,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub scene_id: String,
    pub scene_slug: String,
    pub scene_name: String,
    pub scene_definition_json: String,
    pub world_id: String,
    pub world_slug: String,
    pub world_name: String,
    pub world_file_url: Option<String>,
    pub world_file_hash: Option<String>,
    pub world_size_bytes: Option<i64>,
    pub world_version_id: Option<String>,
    pub profile: Option<String>,
}

pub struct WorkRepo;

impl WorkRepo {
    /// Compute the list of (shader_version, scene, profile) triples that still
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
            WITH latest_versions AS (
                SELECT DISTINCT ON (shader_id)
                    id, shader_id, supported_profiles, capture_failure_count,
                    version, download_url, file_hash, upstream_published_at
                FROM shader_versions
                ORDER BY shader_id, upstream_published_at DESC NULLS LAST, created_at DESC
            ),
            latest_world_versions AS (
                SELECT DISTINCT ON (world_id)
                    id, world_id, file_url, file_hash, size_bytes
                FROM world_versions
                ORDER BY world_id, created_at DESC
            ),
            -- Pre-compute which shader_versions already have completed captures.
            -- Used in ORDER BY to prioritize uncaptured shaders.
            has_captures AS (
                SELECT DISTINCT shader_version_id
                FROM captures
                WHERE status = 'completed'
            ),
            needed AS (
                -- Branch 1: shader versions WITH profiles
                SELECT
                    sv.id AS shader_version_id,
                    s.id AS scene_id,
                    s.world_id,
                    p.profile AS profile
                FROM latest_versions sv
                CROSS JOIN scenes s
                CROSS JOIN LATERAL jsonb_array_elements_text(
                    CASE
                        WHEN sv.supported_profiles IS NOT NULL AND sv.supported_profiles != '[]'
                        THEN sv.supported_profiles::jsonb
                        ELSE '[]'::jsonb
                    END
                ) AS p(profile)
                WHERE s.active = TRUE
                  AND ($2 OR sv.capture_failure_count < 3)
                  AND ($2 OR NOT EXISTS (
                    SELECT 1 FROM captures c
                    LEFT JOIN latest_world_versions lwv ON lwv.world_id = s.world_id
                    WHERE c.shader_version_id = sv.id
                      AND c.scene_id = s.id
                      AND c.profile = p.profile
                      AND c.status IN ('completed', 'uploading')
                      AND c.world_version_id IS NOT DISTINCT FROM lwv.id
                  ))

                UNION ALL

                -- Branch 2: shader versions WITHOUT profiles
                SELECT
                    sv.id AS shader_version_id,
                    s.id AS scene_id,
                    s.world_id,
                    NULL AS profile
                FROM latest_versions sv
                CROSS JOIN scenes s
                WHERE s.active = TRUE
                  AND ($2 OR sv.capture_failure_count < 3)
                  AND (sv.supported_profiles IS NULL OR sv.supported_profiles = '[]')
                  AND ($2 OR NOT EXISTS (
                    SELECT 1 FROM captures c
                    LEFT JOIN latest_world_versions lwv ON lwv.world_id = s.world_id
                    WHERE c.shader_version_id = sv.id
                      AND c.scene_id = s.id
                      AND c.profile IS NULL
                      AND c.status IN ('completed', 'uploading')
                      AND c.world_version_id IS NOT DISTINCT FROM lwv.id
                  ))
            )
            SELECT
                n.shader_version_id AS "shader_version_id!",
                sh.id AS "shader_id!",
                sh.slug AS "shader_slug!",
                sh.name AS "shader_name!",
                sv.version AS "version!",
                sv.download_url,
                sv.file_hash,
                n.scene_id AS "scene_id!",
                sc.slug AS "scene_slug!",
                sc.name AS "scene_name!",
                COALESCE(
                    NULLIF(sc.definition_json, '{}'),
                    jsonb_build_object(
                        'id', sc.slug,
                        'name', sc.name,
                        'position', jsonb_build_object('x', sc.x, 'y', sc.y, 'z', sc.z),
                        'camera', jsonb_build_object('yaw', sc.yaw, 'pitch', sc.pitch),
                        'timeOfDay', sc.time_of_day_ticks,
                        'dimension', sc.dimension,
                        'weather', UPPER(sc.weather),
                        'weatherIntensity', sc.weather_intensity
                    )::text
                ) AS "scene_definition_json!",
                w.id AS "world_id!",
                w.slug AS "world_slug!",
                w.name AS "world_name!",
                lwv.file_url AS world_file_url,
                lwv.file_hash AS world_file_hash,
                lwv.size_bytes AS world_size_bytes,
                lwv.id AS world_version_id,
                n.profile
            FROM needed n
            JOIN shader_versions sv ON sv.id = n.shader_version_id
            JOIN shaders sh ON sh.id = sv.shader_id
            JOIN scenes sc ON sc.id = n.scene_id
            JOIN worlds w ON w.id = n.world_id
            LEFT JOIN latest_world_versions lwv ON lwv.world_id = w.id
            LEFT JOIN has_captures hc ON hc.shader_version_id = n.shader_version_id
            WHERE ($3::text IS NULL OR sh.slug = ANY(string_to_array($3, ',')))
              AND ($4::text IS NULL OR sc.slug = ANY(string_to_array($4, ',')))
            ORDER BY
                hc.shader_version_id IS NOT NULL ASC,
                COALESCE(sh.upstream_downloads, 0) DESC,
                sv.upstream_published_at DESC NULLS LAST,
                sh.name ASC,
                sc.name ASC,
                n.profile NULLS LAST
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
