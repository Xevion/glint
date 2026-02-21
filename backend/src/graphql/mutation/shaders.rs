use async_graphql::{Context, ErrorExtensions, ID, Object, Result};

use crate::error::{AppError, OptionNotFoundExt};
use crate::graphql::guard::{AdminGuard, require_admin};
use crate::graphql::types::shader::{
    CreateShaderVersionInput, ShaderNode, ShaderVersionNode, UpdateShaderInput,
};
use crate::id;
use crate::platform;
use crate::repo::{ShaderRepo, ShaderVersionRepo};
use crate::services::platform::PlatformService;
use crate::state::AppState;

#[derive(Default)]
pub struct ShaderMutation;

#[Object]
impl ShaderMutation {
    /// Update shader metadata.
    #[graphql(guard = "AdminGuard")]
    async fn update_shader(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateShaderInput,
    ) -> Result<ShaderNode> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();

        let old = ShaderRepo::get(db, &id, true)
            .await
            .map_err(|e| e.extend())?;

        let request = input.into();

        let shader = ShaderRepo::update(db, &old.id.0, &request)
            .await
            .map_err(|e| e.extend())?;

        Ok(shader.into())
    }

    /// Delete a shader.
    #[graphql(guard = "AdminGuard")]
    async fn delete_shader(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();

        let deleted = ShaderRepo::delete(state.db(), &id)
            .await
            .map_err(|e| e.extend())?;

        if !deleted {
            return Err(AppError::NotFound("Shader not found or already deleted".into()).extend());
        }

        Ok(true)
    }

    /// Restore a soft-deleted shader.
    #[graphql(guard = "AdminGuard")]
    async fn restore_shader(&self, ctx: &Context<'_>, id: ID) -> Result<ShaderNode> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();

        let shader = ShaderRepo::restore(state.db(), &id)
            .await
            .map_err(|e| e.extend())?;

        match shader {
            Some(s) => Ok(s.into()),
            None => Err(AppError::NotFound("Shader not found or not deleted".into()).extend()),
        }
    }

    /// Permanently delete a soft-deleted shader (hard delete with CASCADE).
    #[graphql(guard = "AdminGuard")]
    async fn purge_shader(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();

        let purged = ShaderRepo::purge(state.db(), &id)
            .await
            .map_err(|e| e.extend())?;

        if !purged {
            return Err(AppError::NotFound("Shader not found or not deleted".into()).extend());
        }

        Ok(true)
    }

    /// Sync shader metadata from upstream platform.
    #[graphql(guard = "AdminGuard")]
    async fn sync_shader(&self, ctx: &Context<'_>, id: ID) -> Result<ShaderNode> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();

        let shader = ShaderRepo::find_by_id(state.db(), &id)
            .await
            .map_err(|e| e.extend())?
            .or_not_found("Shader", &*id)
            .map_err(|e| e.extend())?;

        let shader = PlatformService::sync_shader(state, &shader)
            .await
            .map_err(|e| e.extend())?;

        Ok(shader.into())
    }

    /// Link an additional platform (Modrinth or CurseForge) to a shader.
    #[graphql(guard = "AdminGuard")]
    async fn link_shader_platform(
        &self,
        ctx: &Context<'_>,
        id: ID,
        url: String,
    ) -> Result<ShaderNode> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();

        let shader = ShaderRepo::find_by_id(state.db(), &id)
            .await
            .map_err(|e| e.extend())?
            .or_not_found("Shader", &*id)
            .map_err(|e| e.extend())?;

        let platform_ref = platform::parse_platform_url(&url).ok_or_else(|| {
            AppError::BadRequest("Invalid or unsupported platform URL".into()).extend()
        })?;

        let shader = PlatformService::link_platform(state, &shader, &platform_ref)
            .await
            .map_err(|e| e.extend())?;

        Ok(shader.into())
    }

    /// Create a new version for a shader.
    #[graphql(guard = "AdminGuard")]
    async fn create_shader_version(
        &self,
        ctx: &Context<'_>,
        shader_id: ID,
        input: CreateShaderVersionInput,
    ) -> Result<ShaderVersionNode> {
        let _user = require_admin(ctx)?;
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();

        if !ShaderRepo::exists_by_id(db, &shader_id)
            .await
            .map_err(|e| e.extend())?
        {
            return Err(AppError::NotFound("Shader not found".into()).extend());
        }

        let version_id = id::generate_id();
        let request = input.into();

        let version = ShaderVersionRepo::create(db, &version_id, &shader_id, &request)
            .await
            .map_err(|e| e.extend())?;

        Ok(version.into())
    }
}
