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
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_work))
}

async fn get_work(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<WorkQuery>,
) -> AppResult<Json<Vec<WorkItem>>> {
    let limit = query.limit.unwrap_or(100).min(1000);
    let db = state.db();

    let items = sqlx::query_as!(
        WorkItem,
        r#"
        WITH needed AS (
            SELECT
                sv.id AS shader_version_id,
                s.id AS scene_id,
                p.profile AS profile
            FROM shader_versions sv
            CROSS JOIN scenes s
            CROSS JOIN LATERAL jsonb_array_elements_text(
                CASE
                    WHEN sv.supported_profiles IS NOT NULL AND sv.supported_profiles != '[]'
                    THEN sv.supported_profiles::jsonb
                    ELSE '[]'::jsonb
                END
            ) AS p(profile)
            WHERE s.active = TRUE
              AND sv.capture_failure_count < 3
              AND NOT EXISTS (
                SELECT 1 FROM captures c
                WHERE c.shader_version_id = sv.id
                  AND c.scene_id = s.id
                  AND c.profile = p.profile
                  AND c.status = 'completed'
                  AND c.outdated = FALSE
              )

            UNION ALL

            SELECT
                sv.id AS shader_version_id,
                s.id AS scene_id,
                NULL AS profile
            FROM shader_versions sv
            CROSS JOIN scenes s
            WHERE s.active = TRUE
              AND sv.capture_failure_count < 3
              AND (sv.supported_profiles IS NULL OR sv.supported_profiles = '[]')
              AND NOT EXISTS (
                SELECT 1 FROM captures c
                WHERE c.shader_version_id = sv.id
                  AND c.scene_id = s.id
                  AND c.profile IS NULL
                  AND c.status = 'completed'
                  AND c.outdated = FALSE
              )
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
            COALESCE(sc.definition_json, '{}') AS "scene_definition_json!",
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
        ORDER BY
            EXISTS(
                SELECT 1 FROM captures c2
                WHERE c2.shader_version_id = n.shader_version_id
                  AND c2.status = 'completed'
            ) ASC,
            COALESCE(sh.upstream_downloads, 0) DESC,
            sv.upstream_published_at DESC NULLS LAST,
            sh.name ASC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db)
    .await
    .map_err(|e| crate::error::AppError::Internal(e.into()))?;

    debug!(count = items.len(), "Computed work items");
    Ok(Json(items))
}
