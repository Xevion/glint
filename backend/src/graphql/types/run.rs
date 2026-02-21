use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};

use crate::id::{
    CaptureId, CaptureRunId, SceneId, ScenePresetId, ShaderVersionId, ShaderVersionProfileId,
};
use crate::models::{
    CaptureRun, CaptureRunItemStatus, CaptureRunItemWithContext, CaptureRunStatus,
};
use crate::repo::capture_run::CaptureRunFilters;

use super::connection::{CursorPayload, CursorSource};

/// A capture run with aggregated item counts.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureRunNode {
    pub id: CaptureRunId,
    pub agent_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: CaptureRunStatus,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub skipped_items: i32,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub image_format: String,
    pub items: Vec<CaptureRunItemNode>,
}

/// A single item within a capture run, with denormalized context.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureRunItemNode {
    pub id: String,
    pub run_id: CaptureRunId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
    pub profile_display_name: Option<String>,
    pub preset_id: Option<ScenePresetId>,
    pub status: CaptureRunItemStatus,
    pub capture_id: Option<CaptureId>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    // Denormalized context
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_version: String,
    pub scene_name: String,
    // Capture image data (from joined captures table)
    pub image_path: Option<String>,
    pub thumbhash: Option<String>,
}

/// Lightweight capture run node for list queries (no embedded items).
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureRunListNode {
    pub id: CaptureRunId,
    pub agent_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: CaptureRunStatus,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub skipped_items: i32,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub image_format: String,
    /// Duration in seconds (null if the run hasn't completed).
    pub duration_secs: Option<f64>,
}

impl From<CaptureRunItemWithContext> for CaptureRunItemNode {
    fn from(item: CaptureRunItemWithContext) -> Self {
        Self {
            id: item.id,
            run_id: item.run_id,
            shader_version_id: item.shader_version_id,
            scene_id: item.scene_id,
            profile_id: item.profile_id,
            profile_name: item.profile_name,
            profile_display_name: item.profile_display_name,
            preset_id: item.preset_id,
            status: item.status,
            capture_id: item.capture_id,
            error_message: item.error_message,
            error_log: item.error_log,
            duration_ms: item.duration_ms,
            started_at: item.started_at,
            completed_at: item.completed_at,
            shader_name: item.shader_name,
            shader_slug: item.shader_slug,
            shader_version: item.shader_version,
            scene_name: item.scene_name,
            image_path: item.image_path,
            thumbhash: item.thumbhash,
        }
    }
}

impl From<(CaptureRun, Vec<CaptureRunItemNode>)> for CaptureRunNode {
    fn from((run, items): (CaptureRun, Vec<CaptureRunItemNode>)) -> Self {
        Self {
            id: run.id,
            agent_id: run.agent_id,
            started_at: run.started_at,
            completed_at: run.completed_at,
            status: run.status,
            total_items: run.total_items,
            completed_items: run.completed_items,
            failed_items: run.failed_items,
            skipped_items: run.skipped_items,
            resolution_width: run.resolution_width,
            resolution_height: run.resolution_height,
            image_format: run.image_format,
            items,
        }
    }
}

impl From<CaptureRun> for CaptureRunListNode {
    fn from(r: CaptureRun) -> Self {
        let duration_secs = r
            .completed_at
            .map(|end| (end - r.started_at).num_milliseconds() as f64 / 1000.0);

        Self {
            id: r.id,
            agent_id: r.agent_id,
            started_at: r.started_at,
            completed_at: r.completed_at,
            status: r.status,
            total_items: r.total_items,
            completed_items: r.completed_items,
            failed_items: r.failed_items,
            skipped_items: r.skipped_items,
            resolution_width: r.resolution_width,
            resolution_height: r.resolution_height,
            image_format: r.image_format,
            duration_secs,
        }
    }
}

impl CursorSource for CaptureRun {
    fn to_cursor(&self) -> CursorPayload {
        CursorPayload::new(self.id.as_ref(), self.started_at.timestamp_millis())
    }
}

/// Filters for listing capture runs.
#[derive(InputObject, Default)]
pub struct AdminRunFiltersInput {
    /// Filter by run status (OR logic).
    pub statuses: Option<Vec<CaptureRunStatus>>,
    /// Only runs started after this date.
    pub started_after: Option<DateTime<Utc>>,
    /// Only runs started before this date.
    pub started_before: Option<DateTime<Utc>>,
    /// Minimum duration in seconds (only applies to completed runs).
    pub min_duration_secs: Option<i64>,
    /// Maximum duration in seconds (only applies to completed runs).
    pub max_duration_secs: Option<i64>,
}

impl From<AdminRunFiltersInput> for CaptureRunFilters {
    fn from(f: AdminRunFiltersInput) -> Self {
        Self {
            statuses: f.statuses,
            started_after: f.started_after,
            started_before: f.started_before,
            min_duration_secs: f.min_duration_secs,
            max_duration_secs: f.max_duration_secs,
        }
    }
}
