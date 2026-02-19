use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};

use crate::id::{
    CaptureId, CaptureRunId, SceneId, ScenePresetId, ShaderVersionId, ShaderVersionProfileId,
};
use crate::models::{CaptureRunItemStatus, CaptureRunStatus};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum CaptureRunStatusEnum {
    Running,
    Completed,
    Partial,
    Failed,
    TimedOut,
}

impl From<CaptureRunStatus> for CaptureRunStatusEnum {
    fn from(s: CaptureRunStatus) -> Self {
        match s {
            CaptureRunStatus::Running => Self::Running,
            CaptureRunStatus::Completed => Self::Completed,
            CaptureRunStatus::Partial => Self::Partial,
            CaptureRunStatus::Failed => Self::Failed,
            CaptureRunStatus::TimedOut => Self::TimedOut,
        }
    }
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum CaptureRunItemStatusEnum {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl From<CaptureRunItemStatus> for CaptureRunItemStatusEnum {
    fn from(s: CaptureRunItemStatus) -> Self {
        match s {
            CaptureRunItemStatus::Pending => Self::Pending,
            CaptureRunItemStatus::Running => Self::Running,
            CaptureRunItemStatus::Completed => Self::Completed,
            CaptureRunItemStatus::Failed => Self::Failed,
            CaptureRunItemStatus::Skipped => Self::Skipped,
        }
    }
}

/// A capture run with aggregated item counts.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureRunNode {
    pub id: CaptureRunId,
    pub agent_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: CaptureRunStatusEnum,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub skipped_items: i32,
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
    pub status: CaptureRunItemStatusEnum,
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
