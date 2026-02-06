use anyhow::Context;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{
    CaptureWithContext, CreateShaderRequest, CreateShaderVersionRequest, Shader, ShaderVersion,
    UpdateShaderRequest,
};

pub struct ShaderRepo;

impl ShaderRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn list(db: &DbPool) -> AppResult<Vec<Shader>> {
        let shaders = sqlx::query_as!(Shader, "SELECT * FROM shaders ORDER BY name")
            .fetch_all(db)
            .await
            .context("failed to list shaders")?;
        debug!(count = shaders.len(), "Listed shaders");
        Ok(shaders)
    }

    /// Resolve a shader by UUID or slug. If the input parses as a UUID, looks up
    /// by ID; otherwise falls back to slug.
    #[instrument(skip(db), level = "debug")]
    pub async fn get(db: &DbPool, id_or_slug: &str) -> AppResult<Shader> {
        if Uuid::try_parse(id_or_slug).is_ok() {
            Self::get_by_id(db, id_or_slug).await
        } else {
            Self::get_by_slug(db, id_or_slug).await
        }
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<Shader>> {
        sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to find shader '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_slug(db: &DbPool, slug: &str) -> AppResult<Shader> {
        Self::find_by_slug(db, slug)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shader '{}' not found", slug)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: &str) -> AppResult<Option<Shader>> {
        sqlx::query_as!(Shader, "SELECT * FROM shaders WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find shader by id '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<Shader> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shader with id '{}' not found", id)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn exists_by_slug(db: &DbPool, slug: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM shaders WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to check shader existence '{}'", slug))?;
        Ok(result.is_some())
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn exists_by_id(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM shaders WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to check shader existence by id '{}'", id))?;
        Ok(result.is_some())
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn create(db: &DbPool, id: &str, req: &CreateShaderRequest) -> AppResult<Shader> {
        let result = sqlx::query!(
            r#"
            INSERT INTO shaders (id, name, slug, description, modrinth_id, curseforge_id, website_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
            "#,
            id,
            req.name,
            req.slug,
            req.description,
            req.modrinth_id,
            req.curseforge_id,
            req.website_url
        )
        .execute(db)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result
            && db_err.code().as_deref() == Some("23505")
        {
            return Err(AppError::Conflict(format!(
                "Shader with slug '{}' already exists",
                req.slug
            )));
        }
        result.context(format!("failed to create shader '{}'", req.slug))?;

        Self::get_by_id(db, id).await
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM shaders WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete shader '{}'", id))?;
        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn update(db: &DbPool, id: &str, req: &UpdateShaderRequest) -> AppResult<Shader> {
        sqlx::query!(
            r#"
            UPDATE shaders SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                modrinth_id = COALESCE($3, modrinth_id),
                curseforge_id = COALESCE($4, curseforge_id),
                website_url = COALESCE($5, website_url),
                updated_at = now()
            WHERE id = $6
            "#,
            req.name,
            req.description,
            req.modrinth_id,
            req.curseforge_id,
            req.website_url,
            id
        )
        .execute(db)
        .await
        .context(format!("failed to update shader '{}'", id))?;

        Self::get_by_id(db, id).await
    }

    /// Fetch captures with shader/version context for a given shader
    #[instrument(skip(db), level = "debug")]
    pub async fn get_captures_with_context(
        db: &DbPool,
        shader_id: &str,
    ) -> AppResult<Vec<CaptureWithContext>> {
        let captures = sqlx::query_as!(
            CaptureWithContext,
            r#"
            SELECT
                c.id,
                c.scene_id,
                s.slug as shader_slug,
                s.name as shader_name,
                sv.version as shader_version,
                c.profile,
                c.screenshot_path,
                c.screenshot_url,
                c.captured_at,
                c.resolution_width,
                c.resolution_height
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders s ON sv.shader_id = s.id
            WHERE s.id = $1 AND c.status = 'completed'
            ORDER BY sv.created_at DESC
            "#,
            shader_id
        )
        .fetch_all(db)
        .await
        .context(format!("failed to get captures for shader '{}'", shader_id))?;

        debug!(count = captures.len(), "Fetched captures for shader");
        Ok(captures)
    }
}

pub struct ShaderVersionRepo;

impl ShaderVersionRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn list_by_shader(db: &DbPool, shader_id: &str) -> AppResult<Vec<ShaderVersion>> {
        let versions = sqlx::query_as!(
            ShaderVersion,
            "SELECT * FROM shader_versions WHERE shader_id = $1",
            shader_id
        )
        .fetch_all(db)
        .await
        .context(format!(
            "failed to list versions for shader '{}'",
            shader_id
        ))?;
        debug!(count = versions.len(), "Listed shader versions");
        Ok(versions)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: &str) -> AppResult<Option<ShaderVersion>> {
        sqlx::query_as!(
            ShaderVersion,
            "SELECT * FROM shader_versions WHERE id = $1",
            id
        )
        .fetch_optional(db)
        .await
        .context(format!("failed to find shader version '{}'", id))
        .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<ShaderVersion> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Shader version '{}' not found", id)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn exists_by_id(db: &DbPool, id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar!("SELECT 1 as one FROM shader_versions WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to check shader version existence '{}'", id))?;
        Ok(result.is_some())
    }

    #[instrument(skip(db, req), level = "debug")]
    pub async fn create(
        db: &DbPool,
        id: &str,
        shader_id: &str,
        req: &CreateShaderVersionRequest,
    ) -> AppResult<ShaderVersion> {
        sqlx::query!(
            r#"
            INSERT INTO shader_versions (id, shader_id, version, modrinth_version_id, download_url, file_hash, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            "#,
            id,
            shader_id,
            req.version,
            req.modrinth_version_id,
            req.download_url,
            req.file_hash
        )
        .execute(db)
        .await
        .context(format!("failed to create shader version '{}'", req.version))?;

        Self::get_by_id(db, id).await
    }

    #[instrument(skip(db, profiles_json), level = "debug")]
    pub async fn update_supported_profiles(
        db: &DbPool,
        id: &str,
        profiles_json: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            "UPDATE shader_versions SET supported_profiles = $1 WHERE id = $2",
            profiles_json,
            id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to update supported profiles for version '{}'",
            id
        ))?;
        Ok(())
    }

    /// Get the parent shader's slug for a given shader version ID.
    #[instrument(skip(db), level = "debug")]
    pub async fn get_shader_slug(db: &DbPool, version_id: &str) -> AppResult<Option<String>> {
        sqlx::query_scalar!(
            r#"
            SELECT s.slug FROM shaders s
            JOIN shader_versions sv ON sv.shader_id = s.id
            WHERE sv.id = $1
            "#,
            version_id
        )
        .fetch_optional(db)
        .await
        .context(format!(
            "failed to get shader slug for version '{}'",
            version_id
        ))
        .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_latest_for_shader(
        db: &DbPool,
        shader_id: &str,
    ) -> AppResult<Option<ShaderVersion>> {
        sqlx::query_as!(
            ShaderVersion,
            "SELECT * FROM shader_versions WHERE shader_id = $1 ORDER BY created_at DESC LIMIT 1",
            shader_id
        )
        .fetch_optional(db)
        .await
        .context(format!(
            "failed to get latest version for shader '{}'",
            shader_id
        ))
        .map_err(Into::into)
    }
}
