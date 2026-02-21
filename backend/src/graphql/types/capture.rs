use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};

use crate::id::{
    CaptureId, CaptureRunId, SceneId, ScenePresetId, ShaderVersionId, ShaderVersionProfileId,
};
use crate::models::{CaptureFreshness, CaptureListItem, CaptureStatus, CaptureWithContext};
use crate::repo::capture::CaptureFilters;

use super::connection::{CursorPayload, CursorSource};

/// Lightweight capture for public list endpoints.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureNode {
    pub id: CaptureId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub status: CaptureStatus,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub image_path: String,
    pub thumbhash: Option<String>,
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
}

impl From<CaptureListItem> for CaptureNode {
    fn from(c: CaptureListItem) -> Self {
        Self {
            id: c.id,
            shader_version_id: c.shader_version_id,
            scene_id: c.scene_id,
            status: c.status,
            profile_id: c.profile_id,
            image_path: c.image_path,
            thumbhash: c.thumbhash,
            captured_at: c.captured_at,
            resolution_width: c.resolution_width,
            resolution_height: c.resolution_height,
        }
    }
}

/// Capture with denormalized shader/scene context.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureWithContextNode {
    pub id: CaptureId,
    pub scene_id: SceneId,
    pub shader_slug: String,
    pub shader_name: String,
    pub shader_version: String,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
    pub profile_display_name: Option<String>,
    pub image_path: String,
    pub thumbhash: Option<String>,
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub run_id: Option<CaptureRunId>,
    pub run_status: Option<String>,
    pub shader_author: Option<String>,
    pub scene_name: Option<String>,
    pub scene_slug: Option<String>,
    pub preset_id: Option<ScenePresetId>,
    pub preset_name: Option<String>,
    pub preset_slug: Option<String>,
    pub freshness: CaptureFreshness,
}

impl From<CaptureWithContext> for CaptureWithContextNode {
    fn from(c: CaptureWithContext) -> Self {
        Self {
            id: c.id,
            scene_id: c.scene_id,
            shader_slug: c.shader_slug,
            shader_name: c.shader_name,
            shader_version: c.shader_version,
            profile_id: c.profile_id,
            profile_name: c.profile_name,
            profile_display_name: c.profile_display_name,
            image_path: c.image_path,
            thumbhash: c.thumbhash,
            captured_at: c.captured_at,
            resolution_width: c.resolution_width,
            resolution_height: c.resolution_height,
            file_size_bytes: c.file_size_bytes,
            run_id: c.run_id,
            run_status: c.run_status.map(|s| s.to_string()),
            shader_author: c.shader_author,
            scene_name: c.scene_name,
            scene_slug: c.scene_slug,
            preset_id: c.preset_id,
            preset_name: c.preset_name,
            preset_slug: c.preset_slug,
            freshness: c.freshness,
        }
    }
}

impl CursorSource for CaptureListItem {
    fn to_cursor(&self) -> CursorPayload {
        CursorPayload::new(
            self.id.as_ref(),
            self.captured_at
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0),
        )
    }
}

impl CursorSource for CaptureWithContext {
    fn to_cursor(&self) -> CursorPayload {
        CursorPayload::new(
            self.id.as_ref(),
            self.captured_at
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0),
        )
    }
}

/// Filters for the admin_captures query.
#[derive(InputObject, Default)]
pub struct AdminCaptureFiltersInput {
    /// Filter by shader slug.
    pub shader_slug: Option<String>,
    /// Filter by scene ID.
    pub scene_id: Option<String>,
    /// Filter by capture status (OR logic — matches any selected status).
    pub statuses: Option<Vec<CaptureStatus>>,
    /// Filter by freshness (OR logic — matches any selected freshness).
    pub freshness: Option<Vec<CaptureFreshness>>,
    /// Only captures after this date.
    pub captured_after: Option<DateTime<Utc>>,
    /// Only captures before this date.
    pub captured_before: Option<DateTime<Utc>>,
    /// Minimum file size in bytes.
    pub min_file_size: Option<i64>,
    /// Maximum file size in bytes.
    pub max_file_size: Option<i64>,
}

impl From<AdminCaptureFiltersInput> for CaptureFilters {
    fn from(f: AdminCaptureFiltersInput) -> Self {
        Self {
            shader_slug: f.shader_slug,
            scene_id: f.scene_id.map(SceneId::from),
            statuses: f.statuses,
            freshness: f.freshness,
            captured_after: f.captured_after,
            captured_before: f.captured_before,
            min_file_size: f.min_file_size,
            max_file_size: f.max_file_size,
            ..Default::default()
        }
    }
}
