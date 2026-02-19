use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::FromRow;
use sqlx::types::Json;
use ts_rs::TS;

use super::capture::CaptureWithContext;
use super::extraction::{ExtractionStatus, ShaderVersionMetadata, ShaderVersionProfile};
use super::taxonomy::{Category, Feature};
use crate::id::{ShaderId, ShaderVersionId};

/// Shader pack identity (not version-specific)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
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
    #[ts(as = "Option<String>")]
    pub upstream_updated_at: Option<DateTime<Utc>>,
    #[ts(as = "Option<String>")]
    pub last_synced_at: Option<DateTime<Utc>>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
    pub view_count: i64,
}

/// Specific release of a shader pack
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
pub struct ShaderVersion {
    pub id: ShaderVersionId,
    pub shader_id: ShaderId,
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub curseforge_file_id: Option<i32>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub file_size: Option<i64>,
    #[ts(optional, type = "Array<string>")]
    pub game_versions: Option<Json<Vec<String>>>,
    pub release_channel: Option<String>,
    #[ts(as = "Option<String>")]
    pub upstream_published_at: Option<DateTime<Utc>>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    pub capture_failure_count: i32,
    pub last_capture_error: Option<String>,
    pub extraction_status: ExtractionStatus,
    pub extraction_error: Option<String>,
    #[ts(as = "Option<String>")]
    pub extracted_at: Option<DateTime<Utc>>,
}

/// Aggregate extraction status counts for a shader's versions
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ExtractionSummary {
    pub completed: i64,
    pub failed: i64,
    pub pending: i64,
    pub skipped: i64,
    pub total: i64,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct ShaderListItem {
    #[serde(flatten)]
    pub shader: Shader,
    pub authors: Vec<ShaderAuthor>,
    pub categories: Vec<Category>,
    pub features: Vec<Feature>,
    pub latest_version: Option<String>,
    pub image_path: Option<String>,
    pub thumbhash: Option<String>,
    pub version_count: i64,
    pub extraction_summary: Option<ExtractionSummary>,
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
    pub authors: Vec<ShaderAuthor>,
    pub versions: Vec<ShaderVersionDetail>,
    pub captures: Vec<CaptureWithContext>,
    pub profiles: Vec<ShaderVersionProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub metadata: Option<ShaderVersionMetadata>,
}

/// A shader that is currently trending (high view count in a recent time window).
/// The `view_count` from the flattened `Shader` represents the all-time total;
/// `trending_views` is the count within the requested time window.
#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct TrendingShader {
    #[serde(flatten)]
    pub shader: Shader,
    pub trending_views: i64,
    pub image_path: Option<String>,
    pub thumbhash: Option<String>,
}

/// Shader author from upstream platform
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, optional_fields)]
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

#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
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
    #[ts(as = "Option<String>")]
    pub updated_at: Option<DateTime<Utc>>,
    /// Present when this shader has already been adopted into Glint
    pub adopted: Option<ShaderAdopted>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, TS)]
#[ts(export, optional_fields)]
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
    #[serde(flatten)]
    pub page: crate::models::pagination::PageBody,
    /// Sort order for browse mode (ignored when query is provided)
    pub sort: Option<ShaderSearchSort>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShaderRequest {
    pub name: String,
    pub slug: Option<String>,
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

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct UpdateShaderRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
}
