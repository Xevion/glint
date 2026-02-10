use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    db::DbTransaction,
    error::{AppError, AppResult, OptionNotFoundExt},
    models::{AdoptPreviewAuthor, AdoptPreviewResponse, Shader},
    platform::{self, Platform, PlatformAuthor, PlatformVersion, ProjectData},
    repo::{ShaderAuthorRepo, ShaderRepo, ShaderVersionRepo},
    state::AppState,
};

pub struct PlatformService;

impl PlatformService {
    /// Fetch platform data (metadata, authors, versions) from the appropriate platform client
    #[instrument(skip(state), level = "debug")]
    async fn fetch_platform_data(
        state: &AppState,
        platform_ref: &platform::PlatformRef,
    ) -> AppResult<(Platform, ProjectData, Vec<PlatformVersion>)> {
        match platform_ref {
            platform::PlatformRef::Modrinth { id_or_slug } => {
                let data = state.modrinth().fetch_shader_data(id_or_slug).await?;
                let versions = state
                    .modrinth()
                    .fetch_shader_versions(&data.metadata.platform_id)
                    .await?;
                Ok((Platform::Modrinth, data, versions))
            }
            platform::PlatformRef::CurseForge { slug } => {
                let cf = state.curseforge().ok_or_else(|| {
                    AppError::ServiceUnavailable("CurseForge API key not configured".into())
                })?;
                let data = cf.fetch_shader_data(slug).await?;
                let versions = cf.fetch_shader_versions(&data.metadata.platform_id).await?;
                Ok((Platform::CurseForge, data, versions))
            }
        }
    }

    /// Preview a shader from a platform URL
    pub async fn preview_shader(
        state: &AppState,
        platform_ref: &platform::PlatformRef,
    ) -> AppResult<AdoptPreviewResponse> {
        let (platform, data, versions) = Self::fetch_platform_data(state, platform_ref).await?;

        Ok(AdoptPreviewResponse {
            platform: platform.to_string(),
            name: data.metadata.name,
            slug: data.metadata.slug,
            description: data.metadata.description.unwrap_or_default(),
            icon_url: data.metadata.icon_url,
            downloads: data.metadata.downloads,
            version_count: versions.len(),
            authors: data
                .authors
                .into_iter()
                .map(|a| AdoptPreviewAuthor {
                    name: a.name,
                    url: a.url,
                })
                .collect(),
        })
    }

    /// Adopt a shader from a platform
    pub async fn adopt_shader(
        state: &AppState,
        platform_ref: &platform::PlatformRef,
    ) -> AppResult<Shader> {
        let (platform, data, versions) = Self::fetch_platform_data(state, platform_ref).await?;

        if ShaderRepo::find_by_slug(state.db(), &data.metadata.slug)
            .await?
            .is_some()
        {
            if let Some(analytics) = state.analytics() {
                analytics.track(
                    "shader_adoption_failed",
                    "system",
                    serde_json::json!({
                        "slug": data.metadata.slug,
                        "platform": platform.to_string(),
                        "reason": "already_exists",
                    }),
                );
            }
            return Err(AppError::Conflict(format!(
                "Shader with slug '{}' already exists",
                data.metadata.slug
            )));
        }

        let shader_id = Uuid::new_v4().to_string();
        let slug = data.metadata.slug.clone();
        let version_count = versions.len();
        let author_count = data.authors.len();

        let mut tx = state.begin_tx().await?;
        ShaderRepo::create_from_platform(&mut *tx, &shader_id, platform, &data.metadata).await?;
        Self::upsert_versions(&mut tx, &shader_id, &versions).await?;
        Self::upsert_authors(&mut tx, &shader_id, &platform.to_string(), &data.authors).await?;
        tx.commit().await?;

        info!(
            shader_id = %shader_id,
            slug = %slug,
            versions = version_count,
            authors = author_count,
            platform = %platform,
            "Adopted shader from platform"
        );

        if let Some(analytics) = state.analytics() {
            analytics.track(
                "shader_adoption_succeeded",
                "system",
                serde_json::json!({
                    "shader_id": shader_id,
                    "slug": slug,
                    "platform": platform.to_string(),
                    "version_count": version_count,
                    "author_count": author_count,
                }),
            );
        }

        ShaderRepo::find_by_id(state.db(), &shader_id)
            .await?
            .or_not_found("Shader", &shader_id)
    }

    /// Link another platform to an existing shader
    pub async fn link_platform(
        state: &AppState,
        shader: &Shader,
        platform_ref: &platform::PlatformRef,
    ) -> AppResult<Shader> {
        // Validate platform isn't already linked before fetching
        match platform_ref {
            platform::PlatformRef::Modrinth { .. } if shader.modrinth_id.is_some() => {
                return Err(AppError::Conflict(
                    "Shader already linked to Modrinth".into(),
                ));
            }
            platform::PlatformRef::CurseForge { .. } if shader.curseforge_id.is_some() => {
                return Err(AppError::Conflict(
                    "Shader already linked to CurseForge".into(),
                ));
            }
            _ => {}
        }

        let (platform, data, versions) = Self::fetch_platform_data(state, platform_ref).await?;

        let mut tx = state.begin_tx().await?;
        ShaderRepo::link_platform(
            &mut *tx,
            shader.id.as_ref(),
            platform,
            &data.metadata.platform_id,
        )
        .await?;
        Self::upsert_versions(&mut tx, shader.id.as_ref(), &versions).await?;
        Self::upsert_authors(
            &mut tx,
            shader.id.as_ref(),
            &platform.to_string(),
            &data.authors,
        )
        .await?;
        tx.commit().await?;

        info!(shader_id = %shader.id, platform = %platform, "Linked platform to shader");
        ShaderRepo::find_by_id(state.db(), shader.id.as_ref())
            .await?
            .or_not_found("Shader", &shader.id)
    }

    /// Sync a shader from all linked platforms
    pub async fn sync_shader(state: &AppState, shader: &Shader) -> AppResult<Shader> {
        let mut total_downloads: i64 = 0;
        let mut name = shader.name.clone();
        let mut description = shader.description.clone();
        let mut icon_url = shader.icon_url.clone();
        let mut source_url = shader.source_url.clone();
        let mut license_id = shader.license_id.clone();
        let mut upstream_updated_at = shader.upstream_updated_at;

        let mut tx = state.begin_tx().await?;

        // Sync from Modrinth (primary metadata source if linked)
        if let Some(ref modrinth_id) = shader.modrinth_id {
            match state.modrinth().fetch_shader_data(modrinth_id).await {
                Ok(data) => {
                    name = data.metadata.name;
                    description = data.metadata.description;
                    icon_url = data.metadata.icon_url;
                    source_url = data.metadata.source_url;
                    license_id = data.metadata.license_id;
                    upstream_updated_at = data.metadata.updated_at;
                    total_downloads += data.metadata.downloads as i64;

                    match state.modrinth().fetch_shader_versions(modrinth_id).await {
                        Ok(versions) => {
                            Self::upsert_versions(&mut tx, shader.id.as_ref(), &versions).await?
                        }
                        Err(e) => {
                            tracing::warn!(error = ?e, modrinth_id = %modrinth_id, "Failed to fetch versions from Modrinth")
                        }
                    }
                    Self::upsert_authors(&mut tx, shader.id.as_ref(), "modrinth", &data.authors)
                        .await?;
                }
                Err(e) => {
                    tracing::warn!(error = ?e, modrinth_id = %modrinth_id, "Failed to sync from Modrinth");
                }
            }
        }

        // Sync from CurseForge
        if let Some(ref curseforge_id) = shader.curseforge_id
            && let Some(cf) = state.curseforge()
        {
            match cf.fetch_shader_data(curseforge_id).await {
                Ok(data) => {
                    total_downloads += data.metadata.downloads as i64;

                    // CurseForge fills gaps if Modrinth isn't linked
                    if shader.modrinth_id.is_none() {
                        name = data.metadata.name;
                        description = data.metadata.description;
                        icon_url = data.metadata.icon_url;
                        source_url = data.metadata.source_url;
                    }

                    match cf.fetch_shader_versions(curseforge_id).await {
                        Ok(versions) => {
                            Self::upsert_versions(&mut tx, shader.id.as_ref(), &versions).await?
                        }
                        Err(e) => {
                            tracing::warn!(error = ?e, curseforge_id = %curseforge_id, "Failed to fetch versions from CurseForge")
                        }
                    }
                    Self::upsert_authors(&mut tx, shader.id.as_ref(), "curseforge", &data.authors)
                        .await?;
                }
                Err(e) => {
                    tracing::warn!(error = ?e, curseforge_id = %curseforge_id, "Failed to sync from CurseForge");
                }
            }
        }

        ShaderRepo::update_sync_metadata(
            &mut *tx,
            shader.id.as_ref(),
            &name,
            description.as_deref(),
            icon_url.as_deref(),
            source_url.as_deref(),
            license_id.as_deref(),
            total_downloads,
            upstream_updated_at,
        )
        .await?;

        tx.commit().await?;

        info!(shader_id = %shader.id, "Synced shader from upstream platforms");
        ShaderRepo::find_by_id(state.db(), shader.id.as_ref())
            .await?
            .or_not_found("Shader", &shader.id)
    }

    #[instrument(skip(tx, versions), level = "debug")]
    async fn upsert_versions(
        tx: &mut DbTransaction<'_>,
        shader_id: &str,
        versions: &[PlatformVersion],
    ) -> AppResult<()> {
        ShaderVersionRepo::upsert_versions_batch(&mut **tx, shader_id, versions).await
    }

    #[instrument(skip(tx, authors), level = "debug")]
    async fn upsert_authors(
        tx: &mut DbTransaction<'_>,
        shader_id: &str,
        platform: &str,
        authors: &[PlatformAuthor],
    ) -> AppResult<()> {
        ShaderAuthorRepo::upsert_batch(&mut **tx, shader_id, authors, platform).await
    }
}
