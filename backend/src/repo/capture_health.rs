use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::error::AppResult;

/// Health status of a single capture target
#[derive(Debug, Clone, Serialize, JsonSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum TargetHealth {
    /// No capture exists for this target
    Missing,
    /// Capture exists but marked for recapture
    Stale,
    /// Valid, up-to-date capture exists
    Completed,
    /// Shader version hit the failure cap (capture_failure_count >= 3)
    Failed,
}

/// A single capture target with its health status
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export)]
pub struct CaptureTargetHealth {
    pub shader_id: String,
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_version_id: String,
    pub version: String,
    pub scene_id: String,
    pub scene_name: String,
    pub scene_slug: String,
    pub profile: Option<String>,
    pub status: TargetHealth,
    #[ts(type = "string | null")]
    pub last_capture_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
}

/// Summary counts for the capture health matrix
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export)]
pub struct CaptureHealthSummary {
    pub total_targets: i32,
    pub completed: i32,
    pub missing: i32,
    pub stale: i32,
    pub failed: i32,
}

/// Full capture health response
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export)]
pub struct CaptureHealthResponse {
    pub targets: Vec<CaptureTargetHealth>,
    pub summary: CaptureHealthSummary,
}

/// Raw row from the health query before status classification
#[derive(Debug, sqlx::FromRow)]
struct CaptureHealthRow {
    shader_id: String,
    shader_name: String,
    shader_slug: String,
    shader_version_id: String,
    version: String,
    scene_id: String,
    scene_name: String,
    scene_slug: String,
    profile: Option<String>,
    capture_failure_count: i32,
    last_capture_at: Option<DateTime<Utc>>,
    capture_outdated: Option<bool>,
    has_capture: bool,
}

pub struct CaptureHealthRepo;

impl CaptureHealthRepo {
    #[instrument(skip(executor), level = "debug")]
    pub async fn get_capture_health(
        executor: impl sqlx::PgExecutor<'_>,
    ) -> AppResult<CaptureHealthResponse> {
        let rows = sqlx::query_as!(
            CaptureHealthRow,
            r#"
            WITH latest_versions AS (
                SELECT DISTINCT ON (shader_id)
                    id, shader_id, version, capture_failure_count, supported_profiles
                FROM shader_versions
                ORDER BY shader_id, created_at DESC
            ),
            target_matrix AS (
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

                UNION ALL

                -- Branch 2: shader versions WITHOUT profiles
                SELECT
                    sv.id AS shader_version_id,
                    s.id AS scene_id,
                    NULL AS profile
                FROM latest_versions sv
                CROSS JOIN scenes s
                WHERE s.active = TRUE
                  AND (sv.supported_profiles IS NULL OR sv.supported_profiles = '[]')
            ),
            best_captures AS (
                SELECT DISTINCT ON (c.shader_version_id, c.scene_id, c.profile)
                    c.shader_version_id,
                    c.scene_id,
                    c.profile,
                    c.captured_at,
                    c.outdated
                FROM captures c
                WHERE c.status IN ('completed', 'uploading')
                ORDER BY c.shader_version_id, c.scene_id, c.profile, c.captured_at DESC
            )
            SELECT
                sh.id AS "shader_id!",
                sh.name AS "shader_name!",
                sh.slug AS "shader_slug!",
                sv.id AS "shader_version_id!",
                sv.version AS "version!",
                sc.id AS "scene_id!",
                sc.name AS "scene_name!",
                sc.slug AS "scene_slug!",
                tm.profile,
                sv.capture_failure_count AS "capture_failure_count!",
                bc.captured_at AS last_capture_at,
                bc.outdated AS capture_outdated,
                (bc.captured_at IS NOT NULL) AS "has_capture!"
            FROM target_matrix tm
            JOIN latest_versions sv ON sv.id = tm.shader_version_id
            JOIN shaders sh ON sh.id = sv.shader_id
            JOIN scenes sc ON sc.id = tm.scene_id
            LEFT JOIN best_captures bc
                ON bc.shader_version_id = tm.shader_version_id
                AND bc.scene_id = tm.scene_id
                AND (bc.profile IS NOT DISTINCT FROM tm.profile)
            ORDER BY sh.name ASC, sv.version DESC, sc.name ASC, tm.profile NULLS LAST
            "#
        )
        .fetch_all(executor)
        .await?;

        let targets: Vec<CaptureTargetHealth> = rows
            .into_iter()
            .map(|row| {
                let status = if row.capture_failure_count >= 3 {
                    TargetHealth::Failed
                } else if !row.has_capture {
                    TargetHealth::Missing
                } else if row.capture_outdated.unwrap_or(false) {
                    TargetHealth::Stale
                } else {
                    TargetHealth::Completed
                };

                CaptureTargetHealth {
                    shader_id: row.shader_id,
                    shader_name: row.shader_name,
                    shader_slug: row.shader_slug,
                    shader_version_id: row.shader_version_id,
                    version: row.version,
                    scene_id: row.scene_id,
                    scene_name: row.scene_name,
                    scene_slug: row.scene_slug,
                    profile: row.profile,
                    status,
                    last_capture_at: row.last_capture_at,
                    failure_count: row.capture_failure_count,
                }
            })
            .collect();

        let summary = CaptureHealthSummary {
            total_targets: targets.len() as i32,
            completed: targets
                .iter()
                .filter(|t| t.status == TargetHealth::Completed)
                .count() as i32,
            missing: targets
                .iter()
                .filter(|t| t.status == TargetHealth::Missing)
                .count() as i32,
            stale: targets
                .iter()
                .filter(|t| t.status == TargetHealth::Stale)
                .count() as i32,
            failed: targets
                .iter()
                .filter(|t| t.status == TargetHealth::Failed)
                .count() as i32,
        };

        debug!(
            total = summary.total_targets,
            completed = summary.completed,
            missing = summary.missing,
            stale = summary.stale,
            failed = summary.failed,
            "Computed capture health"
        );

        Ok(CaptureHealthResponse { targets, summary })
    }
}
