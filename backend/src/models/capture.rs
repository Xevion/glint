use std::fmt;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use ts_rs::TS;

use crate::id::{
    CaptureId, CaptureRunId, SceneId, SceneVersionId, ShaderVersionId, ShaderVersionProfileId,
    WorldVersionId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CaptureStatus {
    Uploading,
    Completed,
    Failed,
}

impl fmt::Display for CaptureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CaptureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uploading => "uploading",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CaptureStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "uploading" => Ok(Self::Uploading),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unknown capture status: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CaptureStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CaptureStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CaptureRunStatus {
    Running,
    Completed,
    Partial,
    Failed,
    TimedOut,
}

impl fmt::Display for CaptureRunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CaptureRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CaptureRunStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "partial" => Ok(Self::Partial),
            "failed" => Ok(Self::Failed),
            "timed_out" => Ok(Self::TimedOut),
            _ => Err(format!("unknown capture run status: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CaptureRunStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CaptureRunStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CaptureRunItemStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl fmt::Display for CaptureRunItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CaptureRunItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CaptureRunItemStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(format!("unknown capture run item status: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CaptureRunItemStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CaptureRunItemStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.as_str(), buf)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct Capture {
    pub id: CaptureId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub image_url: Option<String>,
    pub image_path: Option<String>,
    pub video_url: Option<String>,
    pub avg_fps: Option<f64>,
    pub min_fps: Option<f64>,
    pub max_fps: Option<f64>,
    pub frame_time_avg: Option<f64>,
    pub frame_time_p99: Option<f64>,
    pub minecraft_version: Option<String>,
    pub iris_version: Option<String>,
    pub gpu_model: Option<String>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    #[ts(as = "Option<String>")]
    pub captured_at: Option<DateTime<Utc>>,
    pub status: CaptureStatus,
    pub error_message: Option<String>,
    pub thumbhash: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub content_type: Option<String>,
    pub world_version_id: Option<WorldVersionId>,
    pub scene_version_id: Option<SceneVersionId>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct CaptureRun {
    pub id: CaptureRunId,
    pub agent_id: Option<String>,
    #[ts(type = "string")]
    pub started_at: DateTime<Utc>,
    #[ts(as = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,
    pub status: CaptureRunStatus,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub skipped_items: i32,
    #[ts(optional, type = "Record<string, unknown>")]
    pub metadata_json: Option<serde_json::Value>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct CaptureRunItem {
    pub id: String,
    pub run_id: CaptureRunId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub status: CaptureRunItemStatus,
    pub capture_id: Option<CaptureId>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    #[ts(as = "Option<String>")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(as = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Capture run item with denormalized shader/scene info for API responses
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct CaptureRunItemWithContext {
    pub id: String,
    pub run_id: CaptureRunId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
    pub status: CaptureRunItemStatus,
    pub capture_id: Option<CaptureId>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    #[ts(as = "Option<String>")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(as = "Option<String>")]
    pub completed_at: Option<DateTime<Utc>>,
    // Denormalized context
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_version: String,
    pub scene_name: String,
}

/// Freshness status of a capture relative to current versions and newer captures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CaptureFreshness {
    /// Latest capture for its target, world + scene versions match current
    Fresh,
    /// Latest capture for its target, but world or scene version is outdated
    Stale,
    /// A newer capture exists for this target — this capture is obsolete
    Superseded,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CaptureFreshness {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "fresh" => Ok(Self::Fresh),
            "stale" => Ok(Self::Stale),
            "superseded" => Ok(Self::Superseded),
            _ => Err(format!("unknown freshness value: {s}").into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CaptureFreshness {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

/// Capture with denormalized shader/version info for API responses
#[skip_serializing_none]
#[derive(Debug, Serialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct CaptureWithContext {
    pub id: CaptureId,
    pub scene_id: SceneId,
    pub shader_slug: String,
    pub shader_name: String,
    pub shader_version: String,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub profile_name: Option<String>,
    pub image_path: Option<String>,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
    #[ts(as = "Option<String>")]
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub file_size_bytes: Option<i64>,
    // Run context
    pub run_id: Option<CaptureRunId>,
    pub run_status: Option<CaptureRunStatus>,
    // Shader author
    pub shader_author: Option<String>,
    // Scene context
    pub scene_name: Option<String>,
    pub scene_slug: Option<String>,
    // Freshness status
    pub freshness: CaptureFreshness,
}

/// Full capture details for admin detail view, including technical metadata
/// and related captures for cross-referencing.
#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct CaptureDetail {
    #[serde(flatten)]
    pub context: CaptureWithContext,
    // Technical metadata (detail-only fields from Capture model)
    pub shader_version_id: ShaderVersionId,
    pub status: CaptureStatus,
    pub error_message: Option<String>,
    pub video_url: Option<String>,
    pub avg_fps: Option<f64>,
    pub min_fps: Option<f64>,
    pub max_fps: Option<f64>,
    pub frame_time_avg: Option<f64>,
    pub frame_time_p99: Option<f64>,
    pub minecraft_version: Option<String>,
    pub iris_version: Option<String>,
    pub gpu_model: Option<String>,
    pub content_type: Option<String>,
    pub world_version_id: Option<WorldVersionId>,
    pub scene_version_id: Option<SceneVersionId>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
    // Related captures
    pub same_shader_scene: Vec<CaptureWithContext>,
    pub same_scene: Vec<CaptureWithContext>,
    pub same_run: Vec<CaptureWithContext>,
}

/// Lightweight capture summary for public list endpoints.
///
/// Omits performance metrics, GPU info, error details, and other fields
/// that are only needed on the detail view (`GET /api/captures/{id}`).
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct CaptureListItem {
    pub id: CaptureId,
    pub shader_version_id: ShaderVersionId,
    pub scene_id: SceneId,
    pub status: CaptureStatus,
    pub profile_id: Option<ShaderVersionProfileId>,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
    #[ts(as = "Option<String>")]
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
}
