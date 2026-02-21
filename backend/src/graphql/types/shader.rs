use async_graphql::{ComplexObject, Context, Enum, Json, Result, SimpleObject};
use chrono::{DateTime, Utc};

use crate::graphql::loaders::RequestLoaders;
use crate::id::{ShaderId, ShaderVersionId, ShaderVersionProfileId};
use crate::models::capture::CaptureStatus;
use crate::models::extraction::{ExtractionStatus, ShaderVersionMetadata, ShaderVersionProfile};
use crate::models::{
    CreateShaderVersionRequest, ExtractionSummary, Shader, ShaderAuthor, ShaderVersion,
    ShaderVersionDetail, UpdateShaderRequest,
};
use crate::repo::capture::{CaptureDistinct, CaptureFilters};
use crate::repo::{CaptureRepo, ExtractionRepo, ShaderVersionRepo};
use crate::state::AppState;

use super::capture::CaptureWithContextNode;
use super::connection::{
    Connection, CursorPayload, CursorSource, TotalCount, decode_cursor_as_datetime,
};
use super::taxonomy::{CategoryNode, FeatureNode};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExtractionStatusEnum {
    Pending,
    Completed,
    Failed,
    Skipped,
}

impl From<ExtractionStatus> for ExtractionStatusEnum {
    fn from(s: ExtractionStatus) -> Self {
        match s {
            ExtractionStatus::Pending => Self::Pending,
            ExtractionStatus::Completed => Self::Completed,
            ExtractionStatus::Failed => Self::Failed,
            ExtractionStatus::Skipped => Self::Skipped,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct ShaderNode {
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
    pub upstream_updated_at: Option<DateTime<Utc>>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub view_count: i64,
    pub preferred_version_id: Option<String>,
    pub capture_enabled: bool,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Resolve the "effective" version for a shader.
/// Cascade: explicit version_id → shader's preferred_version_id → fallback_latest.
pub(crate) fn resolve_effective_version_sync(
    preferred_version_id: Option<&str>,
    explicit_version_id: Option<String>,
    fallback_latest: Option<ShaderVersionId>,
) -> Option<ShaderVersionId> {
    if let Some(vid) = explicit_version_id {
        return Some(ShaderVersionId::from(vid));
    }
    if let Some(pvid) = preferred_version_id {
        return Some(ShaderVersionId::from(pvid.to_owned()));
    }
    fallback_latest
}

#[ComplexObject]
impl ShaderNode {
    /// All versions of this shader, ordered by publish date descending.
    /// Not DataLoader-batched — only called on single-shader detail pages.
    async fn versions(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return.")] first: i32,
        #[graphql(desc = "Cursor to paginate after.")] after: Option<String>,
    ) -> Result<Connection<ShaderVersionDetailNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let decoded_after = after.map(|c| decode_cursor_as_datetime(&c)).transpose()?;
        let page = ShaderVersionRepo::list_by_shader_cursor(
            state.db(),
            self.id.as_ref(),
            first,
            decoded_after,
        )
        .await?;
        Ok(page.into_connection())
    }

    /// Authors attributed to this shader from upstream platforms.
    async fn authors(&self, ctx: &Context<'_>) -> Result<Vec<ShaderAuthorNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let authors = loaders
            .shader_authors
            .load_one(self.id.0.clone())
            .await?
            .unwrap_or_default();
        Ok(authors.into_iter().map(Into::into).collect())
    }

    /// Categories (style tags) associated with this shader.
    async fn categories(&self, ctx: &Context<'_>) -> Result<Vec<CategoryNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let cats = loaders
            .shader_categories
            .load_one(self.id.0.clone())
            .await?
            .unwrap_or_default();
        Ok(cats.into_iter().map(Into::into).collect())
    }

    /// Feature tags associated with this shader.
    async fn features(&self, ctx: &Context<'_>) -> Result<Vec<FeatureNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let feats = loaders
            .shader_features
            .load_one(self.id.0.clone())
            .await?
            .unwrap_or_default();
        Ok(feats.into_iter().map(Into::into).collect())
    }

    /// Latest version string (e.g. "1.7.2") for display purposes.
    async fn latest_version(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let version = loaders
            .shader_latest_versions
            .load_one(self.id.0.clone())
            .await?;
        Ok(version.map(|v| v.version))
    }

    /// Representative thumbnail image path for this shader.
    async fn image_path(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let thumb = loaders
            .shader_thumbnails
            .load_one(self.id.0.clone())
            .await?;
        Ok(thumb.map(|t| t.image_path))
    }

    /// Thumbhash placeholder for the representative thumbnail.
    async fn thumbhash(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let thumb = loaders
            .shader_thumbnails
            .load_one(self.id.0.clone())
            .await?;
        Ok(thumb.and_then(|t| t.thumbhash))
    }

    /// Total number of versions for this shader.
    async fn version_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let count = loaders
            .shader_version_counts
            .load_one(self.id.0.clone())
            .await?;
        Ok(count.unwrap_or(0))
    }

    /// Extraction summary across all versions.
    async fn extraction_summary(&self, ctx: &Context<'_>) -> Result<Option<ExtractionSummaryNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let summary = loaders
            .shader_extraction_summaries
            .load_one(self.id.0.clone())
            .await?;
        Ok(summary.map(Into::into))
    }

    /// Capture health for this shader: total targets vs completed captures.
    ///
    /// Returns `null` when the shader has no capture targets (no entries in
    /// `capture_target_matrix` for its latest version).
    async fn capture_health(&self, ctx: &Context<'_>) -> Result<Option<ShaderCaptureHealthNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let health = loaders
            .shader_capture_health
            .load_one(self.id.0.clone())
            .await?;
        Ok(health.map(Into::into))
    }

    /// Completed captures for this shader, optionally filtered by version and profile.
    ///
    /// Returns one capture per scene (deduplication via `CaptureDistinct::PerScene`),
    /// matching the behavior of the REST shader detail endpoint.
    async fn captures(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            desc = "Filter to a specific shader version. Defaults to the preferred or latest version."
        )]
        version_id: Option<String>,
        #[graphql(desc = "Filter to a specific profile within the version.")] profile_id: Option<
            String,
        >,
        #[graphql(default = 20, desc = "Number of items to return.")] first: i32,
        #[graphql(desc = "Cursor to paginate after.")] after: Option<String>,
    ) -> Result<Connection<CaptureWithContextNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();
        let loaders = ctx.data_unchecked::<RequestLoaders>();

        let fallback = loaders
            .shader_latest_version_ids
            .load_one(self.id.0.clone())
            .await?;
        let effective_version_id = resolve_effective_version_sync(
            self.preferred_version_id.as_deref(),
            version_id,
            fallback,
        );

        let filters = CaptureFilters {
            shader_id: Some(self.id.clone()),
            version_id: effective_version_id,
            profile_id,
            statuses: Some(vec![CaptureStatus::Completed]),
            scene_active: Some(true),
            ..Default::default()
        };

        let decoded_after = after.map(|c| decode_cursor_as_datetime(&c)).transpose()?;
        let page = CaptureRepo::list_with_context_cursor(
            db,
            &filters,
            first,
            decoded_after,
            CaptureDistinct::PerScene,
            None,
        )
        .await?;

        Ok(page.into_connection())
    }

    /// Profiles for this shader, optionally filtered to a specific version.
    ///
    /// Defaults to the preferred or latest version's profiles when no `version_id` is given.
    async fn profiles(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            desc = "Filter to a specific shader version. Defaults to the preferred or latest version."
        )]
        version_id: Option<String>,
        #[graphql(default = 20, desc = "Number of items to return.")] first: i32,
        #[graphql(desc = "Cursor to paginate after.")] after: Option<String>,
    ) -> Result<Connection<ShaderVersionProfileNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();
        let loaders = ctx.data_unchecked::<RequestLoaders>();

        let fallback = loaders
            .shader_latest_version_ids
            .load_one(self.id.0.clone())
            .await?;
        let effective_version_id = resolve_effective_version_sync(
            self.preferred_version_id.as_deref(),
            version_id,
            fallback,
        );

        if let Some(ref vid) = effective_version_id {
            let decoded_after = after.map(|c| decode_cursor_as_datetime(&c)).transpose()?;
            let page = ExtractionRepo::list_profiles_by_version_cursor(
                db,
                vid.as_ref(),
                first,
                decoded_after,
            )
            .await?;
            Ok(page.into_connection())
        } else {
            Ok(Connection::with_additional_fields(
                false,
                false,
                TotalCount::default(),
            ))
        }
    }

    /// Extracted metadata for this shader, optionally for a specific version.
    ///
    /// Defaults to the preferred or latest version's metadata when no `version_id` is given.
    async fn metadata(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            desc = "Filter to a specific shader version. Defaults to the preferred or latest version."
        )]
        version_id: Option<String>,
    ) -> Result<Option<ShaderVersionMetadataNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();
        let loaders = ctx.data_unchecked::<RequestLoaders>();

        let fallback = loaders
            .shader_latest_version_ids
            .load_one(self.id.0.clone())
            .await?;
        let effective_version_id = resolve_effective_version_sync(
            self.preferred_version_id.as_deref(),
            version_id,
            fallback,
        );

        let metadata = if let Some(ref vid) = effective_version_id {
            ExtractionRepo::get_metadata_by_version(db, vid.as_ref()).await?
        } else {
            None
        };

        Ok(metadata.map(Into::into))
    }
}

impl From<Shader> for ShaderNode {
    fn from(s: Shader) -> Self {
        Self {
            id: s.id,
            name: s.name,
            slug: s.slug,
            description: s.description,
            modrinth_id: s.modrinth_id,
            curseforge_id: s.curseforge_id,
            website_url: s.website_url,
            icon_url: s.icon_url,
            source_url: s.source_url,
            license_id: s.license_id,
            upstream_downloads: s.upstream_downloads,
            upstream_updated_at: s.upstream_updated_at,
            last_synced_at: s.last_synced_at,
            created_at: s.created_at,
            updated_at: s.updated_at,
            view_count: s.view_count,
            preferred_version_id: s.preferred_version_id,
            capture_enabled: s.capture_enabled,
            deleted_at: s.deleted_at,
        }
    }
}

impl CursorSource for Shader {
    fn to_cursor(&self) -> CursorPayload {
        CursorPayload::new(self.id.as_ref(), self.created_at.timestamp_millis())
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ShaderVersionNode {
    pub id: ShaderVersionId,
    pub shader_id: ShaderId,
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub curseforge_file_id: Option<i32>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub file_size: Option<i64>,
    pub game_versions: Option<Vec<String>>,
    pub release_channel: Option<String>,
    pub upstream_published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub capture_failure_count: i32,
    pub last_capture_error: Option<String>,
    pub extraction_status: ExtractionStatusEnum,
    pub extraction_error: Option<String>,
    pub extracted_at: Option<DateTime<Utc>>,
}

impl From<ShaderVersion> for ShaderVersionNode {
    fn from(v: ShaderVersion) -> Self {
        Self {
            id: v.id,
            shader_id: v.shader_id,
            version: v.version,
            modrinth_version_id: v.modrinth_version_id,
            curseforge_file_id: v.curseforge_file_id,
            download_url: v.download_url,
            file_hash: v.file_hash,
            file_size: v.file_size,
            game_versions: v.game_versions.map(|j| j.0),
            release_channel: v.release_channel,
            upstream_published_at: v.upstream_published_at,
            created_at: v.created_at,
            capture_failure_count: v.capture_failure_count,
            last_capture_error: v.last_capture_error,
            extraction_status: v.extraction_status.into(),
            extraction_error: v.extraction_error,
            extracted_at: v.extracted_at,
        }
    }
}

/// A shader version with its completed capture count.
#[derive(SimpleObject, Debug, Clone)]
pub struct ShaderVersionDetailNode {
    pub version: ShaderVersionNode,
    pub capture_count: i64,
}

impl From<ShaderVersionDetail> for ShaderVersionDetailNode {
    fn from(d: ShaderVersionDetail) -> Self {
        Self {
            version: d.version.into(),
            capture_count: d.capture_count,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ShaderAuthorNode {
    pub id: String,
    pub shader_id: ShaderId,
    pub name: String,
    pub url: Option<String>,
    pub platform: String,
}

impl From<ShaderAuthor> for ShaderAuthorNode {
    fn from(a: ShaderAuthor) -> Self {
        Self {
            id: a.id,
            shader_id: a.shader_id,
            name: a.name,
            url: a.url,
            platform: a.platform,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ExtractionSummaryNode {
    pub completed: i64,
    pub failed: i64,
    pub pending: i64,
    pub skipped: i64,
    pub total: i64,
}

impl From<ExtractionSummary> for ExtractionSummaryNode {
    fn from(s: ExtractionSummary) -> Self {
        Self {
            completed: s.completed,
            failed: s.failed,
            pending: s.pending,
            skipped: s.skipped,
            total: s.total,
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ShaderVersionProfileNode {
    pub id: ShaderVersionProfileId,
    pub shader_version_id: ShaderVersionId,
    pub name: String,
    pub label: Option<String>,
    pub display_name: String,
    pub description: Option<String>,
    /// Profile options as structured JSON.
    pub options: Option<Json<serde_json::Value>>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

impl From<ShaderVersionProfile> for ShaderVersionProfileNode {
    fn from(p: ShaderVersionProfile) -> Self {
        Self {
            id: p.id,
            shader_version_id: p.shader_version_id,
            name: p.name,
            label: p.label,
            display_name: p.display_name,
            description: p.description,
            options: Some(Json(p.options)),
            sort_order: p.sort_order,
            created_at: p.created_at,
        }
    }
}

impl CursorSource for ShaderVersionDetail {
    fn to_cursor(&self) -> CursorPayload {
        CursorPayload::new(
            self.version.id.as_ref(),
            self.version.created_at.timestamp_millis(),
        )
    }
}

impl CursorSource for ShaderVersionProfile {
    fn to_cursor(&self) -> CursorPayload {
        CursorPayload::new(self.id.as_ref(), self.created_at.timestamp_millis())
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct ShaderVersionMetadataNode {
    pub shader_version_id: ShaderVersionId,
    pub has_custom_textures: Option<bool>,
    pub extracted_at: DateTime<Utc>,
    /// Pipeline features (e.g. `{"separateAo": true, "clouds": "fancy"}`).
    pub pipeline_features: Option<Json<serde_json::Value>>,
    /// Iris features required by this shader.
    pub iris_features_required: Option<Json<serde_json::Value>>,
    /// Iris features optionally used by this shader.
    pub iris_features_optional: Option<Json<serde_json::Value>>,
    /// Settings screen configuration.
    pub settings_screen: Option<Json<serde_json::Value>>,
    /// Shader file paths.
    pub file_paths: Option<Json<serde_json::Value>>,
    /// Supported dimensions (e.g. `["overworld", "nether", "end"]`).
    pub dimension_support: Option<Json<serde_json::Value>>,
}

impl From<ShaderVersionMetadata> for ShaderVersionMetadataNode {
    fn from(m: ShaderVersionMetadata) -> Self {
        Self {
            shader_version_id: m.shader_version_id,
            has_custom_textures: m.has_custom_textures,
            extracted_at: m.extracted_at,
            pipeline_features: m.pipeline_features.map(Json),
            iris_features_required: m.iris_features_required.map(Json),
            iris_features_optional: m.iris_features_optional.map(Json),
            settings_screen: m.settings_screen.map(Json),
            file_paths: m.file_paths.map(Json),
            dimension_support: m.dimension_support.map(Json),
        }
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct TrendingShaderNode {
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
    pub upstream_updated_at: Option<DateTime<Utc>>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub view_count: i64,
    pub preferred_version_id: Option<String>,
    pub capture_enabled: bool,
    pub trending_views: i64,
    pub image_path: Option<String>,
    pub thumbhash: Option<String>,
}

impl From<crate::models::TrendingShader> for TrendingShaderNode {
    fn from(t: crate::models::TrendingShader) -> Self {
        Self {
            id: t.shader.id,
            name: t.shader.name,
            slug: t.shader.slug,
            description: t.shader.description,
            modrinth_id: t.shader.modrinth_id,
            curseforge_id: t.shader.curseforge_id,
            website_url: t.shader.website_url,
            icon_url: t.shader.icon_url,
            source_url: t.shader.source_url,
            license_id: t.shader.license_id,
            upstream_downloads: t.shader.upstream_downloads,
            upstream_updated_at: t.shader.upstream_updated_at,
            last_synced_at: t.shader.last_synced_at,
            created_at: t.shader.created_at,
            updated_at: t.shader.updated_at,
            view_count: t.shader.view_count,
            preferred_version_id: t.shader.preferred_version_id,
            capture_enabled: t.shader.capture_enabled,
            trending_views: t.trending_views,
            image_path: t.image_path,
            thumbhash: t.thumbhash,
        }
    }
}

/// Capture health summary for a single shader (total targets vs completed captures).
#[derive(SimpleObject, Debug, Clone)]
pub struct ShaderCaptureHealthNode {
    /// Total capture targets for this shader's latest version.
    pub total: i64,
    /// Captures that are completed with a stored image.
    pub completed: i64,
}

impl From<crate::repo::ShaderCaptureHealth> for ShaderCaptureHealthNode {
    fn from(h: crate::repo::ShaderCaptureHealth) -> Self {
        Self {
            total: h.total,
            completed: h.completed,
        }
    }
}

/// Input for updating a shader.
#[derive(async_graphql::InputObject, Debug)]
pub struct UpdateShaderInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
    pub website_url: Option<String>,
    pub capture_enabled: Option<bool>,
    /// Set to a version ID to pin, empty string to clear, omit to leave unchanged.
    pub preferred_version_id: Option<String>,
}

/// Input for creating a shader version.
#[derive(async_graphql::InputObject, Debug)]
pub struct CreateShaderVersionInput {
    pub version: String,
    pub modrinth_version_id: Option<String>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
}

impl From<CreateShaderVersionInput> for CreateShaderVersionRequest {
    fn from(i: CreateShaderVersionInput) -> Self {
        Self {
            version: i.version,
            modrinth_version_id: i.modrinth_version_id,
            download_url: i.download_url,
            file_hash: i.file_hash,
        }
    }
}

impl From<UpdateShaderInput> for UpdateShaderRequest {
    fn from(i: UpdateShaderInput) -> Self {
        Self {
            name: i.name,
            slug: None,
            description: i.description,
            modrinth_id: i.modrinth_id,
            curseforge_id: i.curseforge_id,
            website_url: i.website_url,
            preferred_version_id: i.preferred_version_id,
            capture_enabled: i.capture_enabled,
        }
    }
}
