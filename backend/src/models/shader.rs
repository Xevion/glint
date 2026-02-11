use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

use super::capture::CaptureWithContext;
use super::taxonomy::{Category, Feature};
use crate::id::{ShaderId, ShaderVersionId};

/// Shader pack identity (not version-specific)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct Shader {
    pub id: ShaderId,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
    pub icon_url: Option<String>,
    pub source_url: Option<String>,
    pub license_id: Option<String>,
    pub upstream_downloads: Option<i64>,
    #[ts(type = "string | null")]
    pub upstream_updated_at: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub last_synced_at: Option<DateTime<Utc>>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
    pub view_count: i64,
}

/// Specific release of a shader pack
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct ShaderVersion {
    pub id: ShaderVersionId,
    pub shader_id: ShaderId,
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub curseforge_file_id: Option<i32>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub file_size: Option<i64>,
    #[ts(type = "Array<string> | null")]
    pub game_versions: Option<serde_json::Value>,
    pub release_channel: Option<String>,
    /// Array of profile names, discovered after first capture
    #[ts(type = "Array<string> | null")]
    pub supported_profiles: Option<serde_json::Value>,
    #[ts(type = "string | null")]
    pub upstream_published_at: Option<DateTime<Utc>>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    pub capture_failure_count: i32,
    pub last_capture_error: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderListItem {
    #[serde(flatten)]
    pub shader: Shader,
    pub authors: Vec<ShaderAuthor>,
    pub categories: Vec<Category>,
    pub features: Vec<Feature>,
    pub latest_version: Option<String>,
    #[ts(type = "Array<string> | null")]
    pub game_versions: Option<serde_json::Value>,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderWithVersions {
    #[serde(flatten)]
    pub shader: Shader,
    pub versions: Vec<ShaderVersion>,
}

/// ShaderVersion enriched with capture count for detail endpoints
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderVersionDetail {
    #[serde(flatten)]
    pub version: ShaderVersion,
    pub capture_count: i64,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderWithCaptures {
    #[serde(flatten)]
    pub shader: Shader,
    pub versions: Vec<ShaderVersionDetail>,
    pub captures: Vec<CaptureWithContext>,
}

/// A shader that is currently trending (high view count in a recent time window).
/// The `view_count` from the flattened `Shader` represents the all-time total;
/// `trending_views` is the count within the requested time window.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct TrendingShader {
    #[serde(flatten)]
    pub shader: Shader,
    pub trending_views: i64,
    pub image_url: Option<String>,
    pub thumbhash: Option<String>,
}

/// Shader author from upstream platform
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export)]
pub struct ShaderAuthor {
    pub id: String,
    pub shader_id: ShaderId,
    pub name: String,
    pub url: Option<String>,
    pub platform: String,
}

/// Reference to an already-adopted shader in Glint
#[derive(Debug, Clone, Serialize, FromRow, TS)]
#[ts(export)]
pub struct ShaderAdopted {
    pub id: ShaderId,
    pub slug: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderSearchResult {
    pub platform: String,
    pub platform_id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub author: String,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub platform_url: String,
    /// When the shader was last updated on its platform
    #[ts(type = "string | null")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Present when this shader has already been adopted into Glint
    pub adopted: Option<ShaderAdopted>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ShaderSearchResponse {
    pub results: Vec<ShaderSearchResult>,
    pub total_modrinth: u32,
    pub total_curseforge: Option<u32>,
}

/// Sort order for shader browsing (when no search query is provided)
#[derive(Debug, Default, Deserialize, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ShaderSearchSort {
    #[default]
    Popular,
    Recent,
}

#[derive(Debug, Deserialize)]
pub struct ShaderSearchRequest {
    pub query: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    /// Sort order for browse mode (ignored when query is provided)
    pub sort: Option<ShaderSearchSort>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShaderRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShaderVersionRequest {
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShaderRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
}
