use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};

use crate::id::{
    CaptureId, CaptureRunId, SceneId, ScenePresetId, ShaderVersionId, ShaderVersionProfileId,
};
use crate::models::{CaptureFreshness, CaptureListItem, CaptureStatus, CaptureWithContext};

use super::connection::{CursorEncodable, CursorPage, PageInfo, encode_cursor};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum CaptureStatusEnum {
    Uploading,
    Completed,
    Failed,
}

impl From<CaptureStatus> for CaptureStatusEnum {
    fn from(s: CaptureStatus) -> Self {
        match s {
            CaptureStatus::Uploading => Self::Uploading,
            CaptureStatus::Completed => Self::Completed,
            CaptureStatus::Failed => Self::Failed,
        }
    }
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum CaptureFreshnessEnum {
    Fresh,
    Stale,
    Superseded,
}

impl From<CaptureFreshness> for CaptureFreshnessEnum {
    fn from(f: CaptureFreshness) -> Self {
        match f {
            CaptureFreshness::Fresh => Self::Fresh,
            CaptureFreshness::Stale => Self::Stale,
            CaptureFreshness::Superseded => Self::Superseded,
        }
    }
}

/// Lightweight capture for public list endpoints.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureNode {
    pub id: CaptureId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub status: CaptureStatusEnum,
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
            status: c.status.into(),
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
    pub freshness: CaptureFreshnessEnum,
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
            freshness: c.freshness.into(),
        }
    }
}

impl CursorEncodable for CaptureListItem {
    fn encode_cursor(&self) -> String {
        encode_cursor(
            self.id.as_ref(),
            self.captured_at
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0),
        )
    }
}

/// Relay-style edge for captures.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureEdge {
    pub cursor: String,
    pub node: CaptureNode,
}

/// Relay-style connection for captures.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureConnection {
    pub edges: Vec<CaptureEdge>,
    pub page_info: PageInfo,
    pub total_count: i64,
}

impl From<CursorPage<CaptureListItem>> for CaptureConnection {
    fn from(page: CursorPage<CaptureListItem>) -> Self {
        let (edges, page_info, total_count) = page.into_connection(CaptureNode::from);
        CaptureConnection {
            edges: edges
                .into_iter()
                .map(|(cursor, node)| CaptureEdge { cursor, node })
                .collect(),
            page_info,
            total_count,
        }
    }
}

/// Relay-style edge for captures with context.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureWithContextEdge {
    pub cursor: String,
    pub node: CaptureWithContextNode,
}

/// Relay-style connection for captures with context.
#[derive(SimpleObject, Debug, Clone)]
pub struct CaptureWithContextConnection {
    pub edges: Vec<CaptureWithContextEdge>,
    pub page_info: PageInfo,
    pub total_count: i64,
}
