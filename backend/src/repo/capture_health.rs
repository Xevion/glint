use anyhow::Context;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;
use sqlx::Executor;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::id::{SceneId, ShaderId, ShaderVersionId};

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

/// Why a capture target is stale (only present when status == Stale)
#[derive(Debug, Clone, Serialize, JsonSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum StaleReason {
    /// The world has been updated since the capture
    WorldUpdated,
    /// The scene has been updated since the capture
    SceneUpdated,
    /// Both world and scene have been updated since the capture
    BothUpdated,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TargetHealth {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "missing" => Ok(Self::Missing),
            "stale" => Ok(Self::Stale),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unknown target health value: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for TargetHealth {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

/// A single capture target with its health status
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, optional_fields)]
pub struct CaptureTargetHealth {
    pub shader_id: ShaderId,
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_version_id: ShaderVersionId,
    pub version: String,
    pub scene_id: SceneId,
    pub scene_name: String,
    pub scene_slug: String,
    pub profile_id: Option<String>,
    pub profile_name: Option<String>,
    pub status: TargetHealth,
    pub stale_reason: Option<StaleReason>,
    #[ts(as = "Option<String>")]
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

#[derive(Debug, sqlx::FromRow)]
struct CaptureHealthRow {
    shader_id: ShaderId,
    shader_name: String,
    shader_slug: String,
    shader_version_id: ShaderVersionId,
    version: String,
    scene_id: SceneId,
    scene_name: String,
    scene_slug: String,
    profile_id: Option<String>,
    profile_name: Option<String>,
    capture_failure_count: i32,
    last_capture_at: Option<DateTime<Utc>>,
    status: TargetHealth,
    world_outdated: bool,
    scene_outdated: bool,
}

pub struct CaptureHealthRepo;

impl CaptureHealthRepo {
    /// Compute health status for every capture target in the matrix.
    /// JIT is disabled because the query uses capture_target_matrix (cross-join
    /// fanout inflates cost estimates, same issue as the work query).
    #[instrument(skip(pool), level = "debug")]
    pub async fn get_capture_health(pool: &DbPool) -> AppResult<CaptureHealthResponse> {
        let mut tx = pool.begin().await.context("failed to begin transaction")?;

        tx.execute("SET LOCAL jit = off")
            .await
            .context("failed to disable JIT")?;

        let rows = sqlx::query_as!(
            CaptureHealthRow,
            r#"
            WITH best_captures AS (
                SELECT DISTINCT ON (shader_version_id, scene_id, profile_id)
                    shader_version_id,
                    scene_id,
                    profile_id,
                    captured_at,
                    freshness,
                    world_version_id,
                    scene_version_id
                FROM captures_with_freshness
                WHERE status IN ('completed', 'uploading')
                ORDER BY shader_version_id, scene_id, profile_id, captured_at DESC
            )
            SELECT
                sh.id AS "shader_id!: ShaderId",
                sh.name AS "shader_name!",
                sh.slug AS "shader_slug!",
                sv.id AS "shader_version_id!: ShaderVersionId",
                sv.version AS "version!",
                sc.id AS "scene_id!: SceneId",
                sc.name AS "scene_name!",
                sc.slug AS "scene_slug!",
                tm.profile_id,
                svp.name AS profile_name,
                sv.capture_failure_count AS "capture_failure_count!",
                bc.captured_at AS last_capture_at,
                CASE
                    WHEN sv.capture_failure_count >= 3 THEN 'failed'
                    WHEN bc.captured_at IS NULL THEN 'missing'
                    WHEN bc.freshness != 'fresh' THEN 'stale'
                    ELSE 'completed'
                END AS "status!: TargetHealth",
                COALESCE(bc.world_version_id IS NOT NULL AND bc.world_version_id IS DISTINCT FROM lwv.id, FALSE) AS "world_outdated!",
                COALESCE(bc.scene_version_id IS NOT NULL AND bc.scene_version_id IS DISTINCT FROM lsv.id, FALSE) AS "scene_outdated!"
            FROM capture_target_matrix tm
            JOIN latest_shader_versions sv ON sv.id = tm.shader_version_id
            JOIN shaders sh ON sh.id = sv.shader_id
            JOIN scenes sc ON sc.id = tm.scene_id
            LEFT JOIN best_captures bc
                ON bc.shader_version_id = tm.shader_version_id
                AND bc.scene_id = tm.scene_id
                AND (bc.profile_id IS NOT DISTINCT FROM tm.profile_id)
            LEFT JOIN shader_version_profiles svp ON svp.id = tm.profile_id
            LEFT JOIN latest_world_versions lwv ON lwv.world_id = sc.world_id
            LEFT JOIN latest_scene_versions lsv ON lsv.scene_id = sc.id
            ORDER BY sh.name ASC, sv.version DESC, sc.name ASC, tm.profile_id NULLS LAST
            "#
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await.context("failed to commit transaction")?;

        let targets: Vec<CaptureTargetHealth> = rows
            .into_iter()
            .map(|row| {
                let stale_reason = if row.status == TargetHealth::Stale {
                    Some(match (row.world_outdated, row.scene_outdated) {
                        (true, true) => StaleReason::BothUpdated,
                        (true, false) => StaleReason::WorldUpdated,
                        (false, true) => StaleReason::SceneUpdated,
                        // Fallback: freshness was non-fresh but neither flag tripped
                        // (shouldn't happen, but handle gracefully)
                        (false, false) => StaleReason::WorldUpdated,
                    })
                } else {
                    None
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
                    profile_id: row.profile_id,
                    profile_name: row.profile_name,
                    status: row.status,
                    stale_reason,
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
