use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::AppResult;
use crate::state::AppState;

/// A single work item: one (shader_version, scene, profile) triple to capture
#[derive(Debug, Serialize)]
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
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkQuery {
    pub limit: Option<i64>,
    pub force: Option<bool>,
    pub shaders: Option<String>,
    pub scenes: Option<String>,
    /// If true, returns work items without side effects.
    /// Currently the endpoint is stateless, but this parameter documents intent
    /// and will prevent future reservation/locking logic from triggering.
    pub dry_run: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_work))
}

async fn get_work(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<WorkQuery>,
) -> AppResult<Json<Vec<WorkItem>>> {
    let limit = query.limit.unwrap_or(100).min(1000);
    let force = query.force.unwrap_or(false);
    let dry_run = query.dry_run.unwrap_or(false);

    // "!" and "+" are wildcard sentinels meaning "all" — normalize to None
    let shaders_filter: Option<String> = query.shaders.filter(|s| !matches!(s.as_str(), "!" | "+"));
    let scenes_filter: Option<String> = query.scenes.filter(|s| !matches!(s.as_str(), "!" | "+"));
    let db = state.db();

    let items = sqlx::query_as!(
        WorkItem,
        r#"
        WITH latest_versions AS (
            -- Only consider the most recent version per shader
            SELECT DISTINCT ON (shader_id)
                id, shader_id, supported_profiles, capture_failure_count,
                version, download_url, file_hash, upstream_published_at
            FROM shader_versions
            ORDER BY shader_id, created_at DESC
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
                WHERE c.shader_version_id = sv.id
                  AND c.scene_id = s.id
                  AND c.profile = p.profile
                  AND c.status IN ('completed', 'uploading')
                  AND c.outdated = FALSE
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
                WHERE c.shader_version_id = sv.id
                  AND c.scene_id = s.id
                  AND c.profile IS NULL
                  AND c.status IN ('completed', 'uploading')
                  AND c.outdated = FALSE
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
            w.file_url AS world_file_url,
            w.file_hash AS world_file_hash,
            w.size_bytes AS world_size_bytes,
            n.profile
        FROM needed n
        JOIN shader_versions sv ON sv.id = n.shader_version_id
        JOIN shaders sh ON sh.id = sv.shader_id
        JOIN scenes sc ON sc.id = n.scene_id
        JOIN worlds w ON w.id = sc.world_id
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
    .fetch_all(db)
    .await
    .map_err(|e| crate::error::AppError::Internal(e.into()))?;

    debug!(count = items.len(), force, dry_run, "Computed work items");
    Ok(Json(items))
}
