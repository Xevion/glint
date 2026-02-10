use anyhow::Context;
use schemars::JsonSchema;
use tracing::{debug, instrument};
use ts_rs::TS;

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
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_work_items(
        executor: impl sqlx::PgExecutor<'_>,
        limit: i64,
        force: bool,
        shaders_filter: Option<String>,
        scenes_filter: Option<String>,
    ) -> AppResult<Vec<WorkItem>> {
        let items = sqlx::query_as!(
            WorkItem,
            r#"
            WITH latest_versions AS (
                -- Only consider the most recent version per shader
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
            needed AS (
                -- Branch 1: shader versions WITH profiles
                SELECT
                    sv.id AS shader_version_id,
                    s.id AS scene_id,
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
                    JOIN scenes cs ON cs.id = c.scene_id
                    LEFT JOIN latest_world_versions lwv ON lwv.world_id = cs.world_id
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
                    NULL AS profile
                FROM latest_versions sv
                CROSS JOIN scenes s
                WHERE s.active = TRUE
                  AND ($2 OR sv.capture_failure_count < 3)
                  AND (sv.supported_profiles IS NULL OR sv.supported_profiles = '[]')
                  AND ($2 OR NOT EXISTS (
                    SELECT 1 FROM captures c
                    JOIN scenes cs ON cs.id = c.scene_id
                    LEFT JOIN latest_world_versions lwv ON lwv.world_id = cs.world_id
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
            JOIN worlds w ON w.id = sc.world_id
            LEFT JOIN latest_world_versions lwv ON lwv.world_id = w.id
            WHERE ($3::text IS NULL OR sh.slug = ANY(string_to_array($3, ',')))
              AND ($4::text IS NULL OR sc.slug = ANY(string_to_array($4, ',')))
            ORDER BY
                -- Shader-level priority: shaders without any captures first
                EXISTS(
                    SELECT 1 FROM captures c2
                    WHERE c2.shader_version_id = n.shader_version_id
                      AND c2.status = 'completed'
                ) ASC,
                -- Then by popularity and recency
                COALESCE(sh.upstream_downloads, 0) DESC,
                sv.upstream_published_at DESC NULLS LAST,
                sh.name ASC,
                -- Within a shader: group scenes together
                sc.name ASC,
                n.profile NULLS LAST
            LIMIT $1
            "#,
            limit,
            force,
            shaders_filter as Option<String>,
            scenes_filter as Option<String>,
        )
        .fetch_all(executor)
        .await
        .context("failed to fetch work items")?;

        debug!(count = items.len(), "Fetched work items");
        Ok(items)
    }
}
