use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Capture {
    pub id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
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
    #[ts(type = "string | null")]
    pub captured_at: Option<DateTime<Utc>>,
    pub status: String,
    pub error_message: Option<String>,
    pub thumbhash: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub content_type: Option<String>,
    pub world_version_id: Option<String>,
    pub scene_version_id: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureRun {
    pub id: String,
    pub agent_id: Option<String>,
    #[ts(type = "string")]
    pub started_at: DateTime<Utc>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub skipped_items: i32,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureRunItem {
    pub id: String,
    pub run_id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub status: String,
    pub capture_id: Option<String>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    #[ts(type = "string | null")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Capture run item with denormalized shader/scene info for API responses
#[derive(Debug, Clone, Serialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureRunItemWithContext {
    pub id: String,
    pub run_id: String,
    pub shader_version_id: String,
    pub scene_id: String,
    pub profile: Option<String>,
    pub status: String,
    pub capture_id: Option<String>,
    pub error_message: Option<String>,
    pub error_log: Option<String>,
    pub duration_ms: Option<i32>,
    #[ts(type = "string | null")]
    pub started_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
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
#[derive(Debug, Serialize, FromRow, TS)]
#[ts(export)]
pub struct CaptureWithContext {
    pub id: String,
    pub scene_id: String,
    pub shader_slug: String,
    pub shader_name: String,
    pub shader_version: String,
    pub profile: Option<String>,
    pub image_path: Option<String>,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
    #[ts(type = "string | null")]
    pub captured_at: Option<DateTime<Utc>>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub file_size_bytes: Option<i64>,
    // Run context
    pub run_id: Option<String>,
    pub run_status: Option<String>,
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
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CaptureDetail {
    #[serde(flatten)]
    pub context: CaptureWithContext,
    // Technical metadata (detail-only fields from Capture model)
    pub shader_version_id: String,
    pub status: String,
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
    pub world_version_id: Option<String>,
    pub scene_version_id: Option<String>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
    // Related captures
    pub same_shader_scene: Vec<CaptureWithContext>,
    pub same_scene: Vec<CaptureWithContext>,
    pub same_run: Vec<CaptureWithContext>,
}

/// Paginated captures response envelope
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PaginatedCaptures {
    pub items: Vec<CaptureWithContext>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}
