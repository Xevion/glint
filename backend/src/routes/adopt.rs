use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{
        AdoptPreviewAuthor, AdoptPreviewResponse, AdoptShaderRequest, LinkShaderRequest, Shader,
        ShaderSearchRequest, ShaderSearchResponse, ShaderSearchResult,
    },
    platform::{self, Platform, PlatformError},
    repo::{ShaderAuthorRepo, ShaderRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/adopt/preview", post(adopt_preview))
        .route("/adopt", post(adopt))
        .route("/search", post(search_platforms))
        .route("/{id}/link", post(link_platform))
        .route("/{id}/sync", post(sync_shader))
}

/// POST /api/shaders/search - Search both platforms for shaders (admin)
async fn search_platforms(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<ShaderSearchRequest>,
) -> AppResult<Json<ShaderSearchResponse>> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err(AppError::BadRequest("Search query cannot be empty".into()));
    }

    let limit = request.limit.unwrap_or(20).min(100);
    let offset = request.offset.unwrap_or(0);

    // Search both platforms concurrently
    let modrinth_fut = state.modrinth().search_shaders(query, offset, limit);
    let cf_fut = async {
        if let Some(cf) = state.curseforge() {
            Some(cf.search_shaders(query, limit, offset).await)
        } else {
            None
        }
    };

    let (modrinth_result, cf_result) = tokio::join!(modrinth_fut, cf_fut);

    let mut results: Vec<ShaderSearchResult> = Vec::new();
    let total_modrinth;

    // Process Modrinth results
    match modrinth_result {
        Ok(search) => {
            total_modrinth = search.total_hits;
            for hit in search.hits {
                let slug = hit.slug.unwrap_or_default();
                results.push(ShaderSearchResult {
                    platform: Platform::Modrinth.to_string(),
                    platform_id: hit.project_id,
                    platform_url: format!("https://modrinth.com/shader/{slug}"),
                    slug,
                    name: hit.title,
                    description: hit.description,
                    icon_url: hit.icon_url,
                    author: hit.author,
                    downloads: hit.downloads,
                    categories: hit.categories,
                });
            }
        }
        Err(e) => {
            tracing::warn!(error = ?e, "Modrinth search failed");
            total_modrinth = 0;
        }
    }

    // Process CurseForge results
    let total_curseforge = match cf_result {
        Some(Ok(search)) => {
            let total = search.pagination.map(|p| p.total_count);
            for cf_mod in search.data {
                results.push(ShaderSearchResult {
                    platform: Platform::CurseForge.to_string(),
                    platform_id: cf_mod.id.to_string(),
                    slug: cf_mod.slug.clone(),
                    name: cf_mod.name,
                    description: cf_mod.summary,
                    icon_url: cf_mod.logo.map(|l| l.url),
                    author: cf_mod
                        .authors
                        .first()
                        .map(|a| a.name.clone())
                        .unwrap_or_default(),
                    downloads: cf_mod.download_count,
                    categories: cf_mod.categories.into_iter().map(|c| c.name).collect(),
                    platform_url: format!(
                        "https://www.curseforge.com/minecraft/shaders/{}",
                        cf_mod.slug
                    ),
                });
            }
            total
        }
        Some(Err(e)) => {
            tracing::warn!(error = ?e, "CurseForge search failed");
            Some(0)
        }
        None => None,
    };

    // Sort by downloads descending
    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    Ok(Json(ShaderSearchResponse {
        results,
        total_modrinth,
        total_curseforge,
    }))
}

/// POST /api/shaders/adopt/preview - Preview a shader from a platform URL (admin)
async fn adopt_preview(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<AdoptShaderRequest>,
) -> AppResult<Json<AdoptPreviewResponse>> {
    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    match platform_ref {
        platform::PlatformRef::Modrinth { id_or_slug } => {
            let project = state
                .modrinth()
                .get_project(&id_or_slug)
                .await
                .map_err(platform_error_to_app_error)?;

            let members = state
                .modrinth()
                .get_team_members(&project.id)
                .await
                .unwrap_or_default();

            let versions = state
                .modrinth()
                .list_versions(&project.id, None, None)
                .await
                .map_err(platform_error_to_app_error)?;

            Ok(Json(AdoptPreviewResponse {
                platform: Platform::Modrinth.to_string(),
                name: project.title,
                slug: project.slug,
                description: project.description,
                icon_url: project.icon_url,
                downloads: project.downloads,
                version_count: versions.len(),
                authors: members
                    .into_iter()
                    .map(|m| AdoptPreviewAuthor {
                        name: m.user.username,
                        url: m.user.avatar_url,
                    })
                    .collect(),
            }))
        }
        platform::PlatformRef::CurseForge { slug } => {
            let cf = state.curseforge().ok_or_else(|| {
                AppError::ServiceUnavailable("CurseForge API key not configured".into())
            })?;

            let cf_mod = cf
                .get_mod_by_slug(&slug)
                .await
                .map_err(platform_error_to_app_error)?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Shader '{}' not found on CurseForge", slug))
                })?;

            let files = cf
                .list_files(cf_mod.id, None)
                .await
                .map_err(platform_error_to_app_error)?;

            Ok(Json(AdoptPreviewResponse {
                platform: Platform::CurseForge.to_string(),
                name: cf_mod.name,
                slug: cf_mod.slug,
                description: cf_mod.summary,
                icon_url: cf_mod.logo.map(|l| l.url),
                downloads: cf_mod.download_count,
                version_count: files.len(),
                authors: cf_mod
                    .authors
                    .into_iter()
                    .map(|a| AdoptPreviewAuthor {
                        name: a.name,
                        url: Some(a.url),
                    })
                    .collect(),
            }))
        }
    }
}

/// POST /api/shaders/adopt - Adopt a shader from a platform URL (admin)
async fn adopt(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<AdoptShaderRequest>,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    match platform_ref {
        platform::PlatformRef::Modrinth { id_or_slug } => {
            adopt_from_modrinth(&state, &id_or_slug).await
        }
        platform::PlatformRef::CurseForge { slug } => adopt_from_curseforge(&state, &slug).await,
    }
}

async fn adopt_from_modrinth(
    state: &AppState,
    id_or_slug: &str,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let project = state
        .modrinth()
        .get_project(id_or_slug)
        .await
        .map_err(platform_error_to_app_error)?;

    // Check for duplicate
    if let Some(_existing) = ShaderRepo::find_by_slug(state.db(), &project.slug).await? {
        return Err(AppError::Conflict(format!(
            "Shader with slug '{}' already exists",
            project.slug
        )));
    }

    let shader_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Create shader
    sqlx::query!(
        r#"
        INSERT INTO shaders (
            id, name, slug, description, modrinth_id, website_url,
            icon_url, source_url, license_id, upstream_downloads,
            upstream_updated_at, last_synced_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)
        "#,
        shader_id,
        project.title,
        project.slug,
        project.description,
        project.id,
        project.source_url,
        project.icon_url,
        project.source_url,
        project.license.as_ref().map(|l| l.id.as_str()),
        project.downloads as i64,
        project.updated,
        now,
        now,
    )
    .execute(state.db())
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    // Fetch and insert versions
    let versions = state
        .modrinth()
        .list_versions(&project.id, None, None)
        .await
        .map_err(platform_error_to_app_error)?;

    for version in &versions {
        let version_id = Uuid::new_v4().to_string();
        let primary_file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or(version.files.first());
        let game_versions_json = serde_json::to_string(&version.game_versions).ok();
        let release_channel = format!("{:?}", version.version_type).to_lowercase();

        sqlx::query!(
            r#"
            INSERT INTO shader_versions (
                id, shader_id, version, modrinth_version_id, download_url,
                file_hash, file_size, game_versions, release_channel,
                upstream_published_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (shader_id, version) DO NOTHING
            "#,
            version_id,
            shader_id,
            version.version_number,
            version.id,
            primary_file.map(|f| f.url.as_str()),
            primary_file.map(|f| f.hashes.sha1.as_str()),
            primary_file.map(|f| f.size as i64),
            game_versions_json,
            release_channel,
            version.date_published,
        )
        .execute(state.db())
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    }

    // Fetch and insert authors
    let members = state
        .modrinth()
        .get_team_members(&project.id)
        .await
        .unwrap_or_default();

    for member in &members {
        let author_id = Uuid::new_v4().to_string();
        ShaderAuthorRepo::upsert(
            state.db(),
            &author_id,
            &shader_id,
            &member.user.username,
            member.user.avatar_url.as_deref(),
            "modrinth",
        )
        .await?;
    }

    info!(
        shader_id = %shader_id,
        slug = %project.slug,
        versions = versions.len(),
        authors = members.len(),
        "Adopted shader from Modrinth"
    );

    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    Ok((StatusCode::CREATED, Json(shader)))
}

async fn adopt_from_curseforge(
    state: &AppState,
    slug: &str,
) -> AppResult<(StatusCode, Json<Shader>)> {
    let cf = state
        .curseforge()
        .ok_or_else(|| AppError::ServiceUnavailable("CurseForge API key not configured".into()))?;

    let cf_mod = cf
        .get_mod_by_slug(slug)
        .await
        .map_err(platform_error_to_app_error)?
        .ok_or_else(|| AppError::NotFound(format!("Shader '{}' not found on CurseForge", slug)))?;

    // Check for duplicate
    if let Some(_existing) = ShaderRepo::find_by_slug(state.db(), &cf_mod.slug).await? {
        return Err(AppError::Conflict(format!(
            "Shader with slug '{}' already exists",
            cf_mod.slug
        )));
    }

    let shader_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let source_url = cf_mod.links.as_ref().and_then(|l| l.source_url.clone());
    let website_url = cf_mod.links.as_ref().and_then(|l| l.website_url.clone());

    sqlx::query!(
        r#"
        INSERT INTO shaders (
            id, name, slug, description, curseforge_id, website_url,
            icon_url, source_url, upstream_downloads,
            last_synced_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
        "#,
        shader_id,
        cf_mod.name,
        cf_mod.slug,
        cf_mod.summary,
        cf_mod.id.to_string(),
        website_url,
        cf_mod.logo.as_ref().map(|l| l.url.as_str()),
        source_url,
        cf_mod.download_count as i64,
        now,
        now,
    )
    .execute(state.db())
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    // Fetch and insert files as versions
    let files = cf
        .list_files(cf_mod.id, None)
        .await
        .map_err(platform_error_to_app_error)?;

    for file in &files {
        let version_id = Uuid::new_v4().to_string();
        let game_versions_json = serde_json::to_string(&file.game_versions).ok();
        let sha1_hash = file
            .hashes
            .iter()
            .find(|h| matches!(h.algo, crate::platform::curseforge::CfHashAlgo::Sha1))
            .map(|h| h.value.as_str());
        let release_channel = format!("{:?}", file.release_type).to_lowercase();

        sqlx::query!(
            r#"
            INSERT INTO shader_versions (
                id, shader_id, version, curseforge_file_id, download_url,
                file_hash, file_size, game_versions, release_channel,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            ON CONFLICT (shader_id, version) DO NOTHING
            "#,
            version_id,
            shader_id,
            file.display_name,
            file.id,
            file.download_url,
            sha1_hash,
            file.file_length as i64,
            game_versions_json,
            release_channel,
        )
        .execute(state.db())
        .await
        .map_err(|e| AppError::Internal(e.into()))?;
    }

    // Insert authors
    for author in &cf_mod.authors {
        let author_id = Uuid::new_v4().to_string();
        ShaderAuthorRepo::upsert(
            state.db(),
            &author_id,
            &shader_id,
            &author.name,
            Some(&author.url),
            "curseforge",
        )
        .await?;
    }

    info!(
        shader_id = %shader_id,
        slug = %cf_mod.slug,
        versions = files.len(),
        authors = cf_mod.authors.len(),
        "Adopted shader from CurseForge"
    );

    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    Ok((StatusCode::CREATED, Json(shader)))
}

/// POST /api/shaders/{id}/link - Link another platform to an existing shader (admin)
async fn link_platform(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
    Json(request): Json<LinkShaderRequest>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;

    let platform_ref = platform::parse_platform_url(&request.url)
        .ok_or_else(|| AppError::BadRequest("Invalid or unsupported platform URL".into()))?;

    match platform_ref {
        platform::PlatformRef::Modrinth { id_or_slug } => {
            if shader.modrinth_id.is_some() {
                return Err(AppError::Conflict(
                    "Shader already linked to Modrinth".into(),
                ));
            }

            let project = state
                .modrinth()
                .get_project(&id_or_slug)
                .await
                .map_err(platform_error_to_app_error)?;

            sqlx::query!(
                "UPDATE shaders SET modrinth_id = $1, updated_at = now() WHERE id = $2",
                project.id,
                shader_id
            )
            .execute(state.db())
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

            // Also import versions and authors from the newly linked platform
            let versions = state
                .modrinth()
                .list_versions(&project.id, None, None)
                .await
                .map_err(platform_error_to_app_error)?;

            for version in &versions {
                let version_id = Uuid::new_v4().to_string();
                let primary_file = version
                    .files
                    .iter()
                    .find(|f| f.primary)
                    .or(version.files.first());
                let game_versions_json = serde_json::to_string(&version.game_versions).ok();
                let release_channel = format!("{:?}", version.version_type).to_lowercase();

                sqlx::query!(
                    r#"
                    INSERT INTO shader_versions (
                        id, shader_id, version, modrinth_version_id, download_url,
                        file_hash, file_size, game_versions, release_channel,
                        upstream_published_at, created_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
                    ON CONFLICT (shader_id, version) DO UPDATE SET
                        modrinth_version_id = EXCLUDED.modrinth_version_id,
                        download_url = COALESCE(EXCLUDED.download_url, shader_versions.download_url),
                        file_hash = COALESCE(EXCLUDED.file_hash, shader_versions.file_hash)
                    "#,
                    version_id,
                    shader_id,
                    version.version_number,
                    version.id,
                    primary_file.map(|f| f.url.as_str()),
                    primary_file.map(|f| f.hashes.sha1.as_str()),
                    primary_file.map(|f| f.size as i64),
                    game_versions_json,
                    release_channel,
                    version.date_published,
                )
                .execute(state.db())
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
            }

            let members = state
                .modrinth()
                .get_team_members(&project.id)
                .await
                .unwrap_or_default();

            for member in &members {
                let author_id = Uuid::new_v4().to_string();
                ShaderAuthorRepo::upsert(
                    state.db(),
                    &author_id,
                    &shader_id,
                    &member.user.username,
                    member.user.avatar_url.as_deref(),
                    "modrinth",
                )
                .await?;
            }

            info!(shader_id = %shader_id, modrinth_id = %project.id, "Linked Modrinth to shader");
        }
        platform::PlatformRef::CurseForge { slug } => {
            if shader.curseforge_id.is_some() {
                return Err(AppError::Conflict(
                    "Shader already linked to CurseForge".into(),
                ));
            }

            let cf = state.curseforge().ok_or_else(|| {
                AppError::ServiceUnavailable("CurseForge API key not configured".into())
            })?;

            let cf_mod = cf
                .get_mod_by_slug(&slug)
                .await
                .map_err(platform_error_to_app_error)?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Shader '{}' not found on CurseForge", slug))
                })?;

            sqlx::query!(
                "UPDATE shaders SET curseforge_id = $1, updated_at = now() WHERE id = $2",
                cf_mod.id.to_string(),
                shader_id
            )
            .execute(state.db())
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

            let files = cf
                .list_files(cf_mod.id, None)
                .await
                .map_err(platform_error_to_app_error)?;

            for file in &files {
                let version_id = Uuid::new_v4().to_string();
                let game_versions_json = serde_json::to_string(&file.game_versions).ok();
                let sha1_hash = file
                    .hashes
                    .iter()
                    .find(|h| matches!(h.algo, crate::platform::curseforge::CfHashAlgo::Sha1))
                    .map(|h| h.value.as_str());
                let release_channel = format!("{:?}", file.release_type).to_lowercase();

                sqlx::query!(
                    r#"
                    INSERT INTO shader_versions (
                        id, shader_id, version, curseforge_file_id, download_url,
                        file_hash, file_size, game_versions, release_channel, created_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
                    ON CONFLICT (shader_id, version) DO UPDATE SET
                        curseforge_file_id = EXCLUDED.curseforge_file_id,
                        download_url = COALESCE(EXCLUDED.download_url, shader_versions.download_url),
                        file_hash = COALESCE(EXCLUDED.file_hash, shader_versions.file_hash)
                    "#,
                    version_id,
                    shader_id,
                    file.display_name,
                    file.id,
                    file.download_url,
                    sha1_hash,
                    file.file_length as i64,
                    game_versions_json,
                    release_channel,
                )
                .execute(state.db())
                .await
                .map_err(|e| AppError::Internal(e.into()))?;
            }

            for author in &cf_mod.authors {
                let author_id = Uuid::new_v4().to_string();
                ShaderAuthorRepo::upsert(
                    state.db(),
                    &author_id,
                    &shader_id,
                    &author.name,
                    Some(&author.url),
                    "curseforge",
                )
                .await?;
            }

            info!(shader_id = %shader_id, curseforge_id = %cf_mod.id, "Linked CurseForge to shader");
        }
    }

    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    Ok(Json(shader))
}

/// POST /api/shaders/{id}/sync - Re-fetch metadata and versions from linked platforms (admin)
async fn sync_shader(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(shader_id): Path<String>,
) -> AppResult<Json<Shader>> {
    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    let now = Utc::now();

    let mut total_downloads: i64 = 0;
    let mut name = shader.name.clone();
    let mut description = shader.description.clone();
    let mut icon_url = shader.icon_url.clone();
    let mut source_url = shader.source_url.clone();
    let mut license_id = shader.license_id.clone();
    let mut upstream_updated_at = shader.upstream_updated_at;

    // Sync from Modrinth (primary source of metadata if linked)
    if let Some(ref modrinth_id) = shader.modrinth_id {
        match state.modrinth().get_project(modrinth_id).await {
            Ok(project) => {
                name = project.title;
                description = Some(project.description);
                icon_url = project.icon_url;
                source_url = project.source_url;
                license_id = project.license.map(|l| l.id);
                upstream_updated_at = Some(project.updated);
                total_downloads += project.downloads as i64;

                // Upsert versions
                if let Ok(versions) = state
                    .modrinth()
                    .list_versions(modrinth_id, None, None)
                    .await
                {
                    for version in &versions {
                        let version_id = Uuid::new_v4().to_string();
                        let primary_file = version
                            .files
                            .iter()
                            .find(|f| f.primary)
                            .or(version.files.first());
                        let game_versions_json = serde_json::to_string(&version.game_versions).ok();
                        let release_channel = format!("{:?}", version.version_type).to_lowercase();

                        sqlx::query!(
                            r#"
                            INSERT INTO shader_versions (
                                id, shader_id, version, modrinth_version_id, download_url,
                                file_hash, file_size, game_versions, release_channel,
                                upstream_published_at, created_at
                            )
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
                            ON CONFLICT (shader_id, version) DO UPDATE SET
                                modrinth_version_id = EXCLUDED.modrinth_version_id,
                                download_url = COALESCE(EXCLUDED.download_url, shader_versions.download_url),
                                file_hash = COALESCE(EXCLUDED.file_hash, shader_versions.file_hash),
                                file_size = COALESCE(EXCLUDED.file_size, shader_versions.file_size),
                                game_versions = COALESCE(EXCLUDED.game_versions, shader_versions.game_versions),
                                release_channel = COALESCE(EXCLUDED.release_channel, shader_versions.release_channel),
                                upstream_published_at = COALESCE(EXCLUDED.upstream_published_at, shader_versions.upstream_published_at)
                            "#,
                            version_id,
                            shader_id,
                            version.version_number,
                            version.id,
                            primary_file.map(|f| f.url.as_str()),
                            primary_file.map(|f| f.hashes.sha1.as_str()),
                            primary_file.map(|f| f.size as i64),
                            game_versions_json,
                            release_channel,
                            version.date_published,
                        )
                        .execute(state.db())
                        .await
                        .map_err(|e| AppError::Internal(e.into()))?;
                    }
                }

                // Refresh authors
                if let Ok(members) = state.modrinth().get_team_members(modrinth_id).await {
                    for member in &members {
                        let author_id = Uuid::new_v4().to_string();
                        ShaderAuthorRepo::upsert(
                            state.db(),
                            &author_id,
                            &shader_id,
                            &member.user.username,
                            member.user.avatar_url.as_deref(),
                            "modrinth",
                        )
                        .await?;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = ?e, modrinth_id = %modrinth_id, "Failed to sync from Modrinth");
            }
        }
    }

    // Sync from CurseForge
    if let Some(ref curseforge_id) = shader.curseforge_id
        && let Some(cf) = state.curseforge()
        && let Ok(cf_id) = curseforge_id.parse::<i32>()
    {
        match cf.get_mod(cf_id).await {
            Ok(cf_mod) => {
                total_downloads += cf_mod.download_count as i64;

                // CurseForge fills gaps if Modrinth isn't linked
                if shader.modrinth_id.is_none() {
                    name = cf_mod.name;
                    description = Some(cf_mod.summary);
                    icon_url = cf_mod.logo.map(|l| l.url);
                    source_url = cf_mod.links.and_then(|l| l.source_url);
                }

                // Upsert file versions
                if let Ok(files) = cf.list_files(cf_id, None).await {
                    for file in &files {
                        let version_id = Uuid::new_v4().to_string();
                        let game_versions_json = serde_json::to_string(&file.game_versions).ok();
                        let sha1_hash = file
                            .hashes
                            .iter()
                            .find(|h| {
                                matches!(h.algo, crate::platform::curseforge::CfHashAlgo::Sha1)
                            })
                            .map(|h| h.value.as_str());
                        let release_channel = format!("{:?}", file.release_type).to_lowercase();

                        sqlx::query!(
                                    r#"
                                    INSERT INTO shader_versions (
                                        id, shader_id, version, curseforge_file_id, download_url,
                                        file_hash, file_size, game_versions, release_channel, created_at
                                    )
                                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
                                    ON CONFLICT (shader_id, version) DO UPDATE SET
                                        curseforge_file_id = EXCLUDED.curseforge_file_id,
                                        download_url = COALESCE(EXCLUDED.download_url, shader_versions.download_url),
                                        file_hash = COALESCE(EXCLUDED.file_hash, shader_versions.file_hash),
                                        file_size = COALESCE(EXCLUDED.file_size, shader_versions.file_size),
                                        game_versions = COALESCE(EXCLUDED.game_versions, shader_versions.game_versions),
                                        release_channel = COALESCE(EXCLUDED.release_channel, shader_versions.release_channel)
                                    "#,
                                    version_id,
                                    shader_id,
                                    file.display_name,
                                    file.id,
                                    file.download_url,
                                    sha1_hash,
                                    file.file_length as i64,
                                    game_versions_json,
                                    release_channel,
                                )
                                .execute(state.db())
                                .await
                                .map_err(|e| AppError::Internal(e.into()))?;
                    }
                }

                // Refresh CF authors
                for author in &cf_mod.authors {
                    let author_id = Uuid::new_v4().to_string();
                    ShaderAuthorRepo::upsert(
                        state.db(),
                        &author_id,
                        &shader_id,
                        &author.name,
                        Some(&author.url),
                        "curseforge",
                    )
                    .await?;
                }
            }
            Err(e) => {
                tracing::warn!(error = ?e, curseforge_id = %curseforge_id, "Failed to sync from CurseForge");
            }
        }
    }

    // Update shader metadata
    sqlx::query!(
        r#"
        UPDATE shaders SET
            name = $1,
            description = $2,
            icon_url = $3,
            source_url = $4,
            license_id = $5,
            upstream_downloads = $6,
            upstream_updated_at = $7,
            last_synced_at = $8,
            updated_at = $8
        WHERE id = $9
        "#,
        name,
        description,
        icon_url,
        source_url,
        license_id,
        total_downloads,
        upstream_updated_at,
        now,
        shader_id,
    )
    .execute(state.db())
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    info!(shader_id = %shader_id, "Synced shader from upstream platforms");

    let shader = ShaderRepo::get_by_id(state.db(), &shader_id).await?;
    Ok(Json(shader))
}

fn platform_error_to_app_error(e: PlatformError) -> AppError {
    match e {
        PlatformError::NotFound => AppError::NotFound("Resource not found on platform".into()),
        PlatformError::RateLimited { retry_after_secs } => AppError::ServiceUnavailable(format!(
            "Platform rate limited, retry after {}s",
            retry_after_secs
        )),
        PlatformError::BadStatus { status, body } => {
            tracing::error!(status, body = %body, "Platform API error");
            AppError::ServiceUnavailable(format!("Platform API returned status {}", status))
        }
        PlatformError::Http(e) => {
            tracing::error!(error = ?e, "Platform HTTP error");
            AppError::ServiceUnavailable("Failed to connect to platform API".into())
        }
        PlatformError::Deserialize { url, source, .. } => {
            tracing::error!(url = %url, error = ?source, "Failed to parse platform response");
            AppError::Internal(source.into())
        }
    }
}
