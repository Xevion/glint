use async_graphql::{Context, Object, Result};

use crate::error::AppError;
use crate::graphql::types::connection::decode_cursor;
use crate::graphql::types::shader::{ShaderConnection, ShaderNode, TrendingShaderNode};
use crate::models::Page;
use crate::repo::{ShaderRepo, SlugRedirectRepo};
use crate::services::shader::ShaderService;
use crate::state::AppState;

#[derive(Default)]
pub struct ShaderQuery;

#[Object]
impl ShaderQuery {
    /// Paginated list of shaders with cursor pagination.
    async fn shaders(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
        #[graphql(desc = "Search by name")] search: Option<String>,
        #[graphql(desc = "Sort: popular, name, or default (recent)")] sort: Option<String>,
    ) -> Result<ShaderConnection> {
        let state = ctx.data_unchecked::<AppState>();

        let decoded_after = after
            .map(|c| decode_cursor(&c))
            .transpose()?
            .map(|(id, ts)| {
                let dt = chrono::DateTime::from_timestamp_millis(ts)
                    .ok_or_else(|| AppError::BadRequest("Invalid cursor timestamp".into()))?;
                Ok::<_, AppError>((dt, id))
            })
            .transpose()?;

        let page = ShaderRepo::list_cursor(
            state.db(),
            first,
            decoded_after,
            search.as_deref(),
            sort.as_deref(),
        )
        .await?;

        Ok(page.into())
    }

    /// Get a shader by ID or slug with full detail.
    ///
    /// Returns a `ShaderNode` whose nested fields (versions, authors) are
    /// resolved on demand via `ComplexObject` resolvers.
    async fn shader(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Shader ID (nanoid) or slug")] id: String,
    ) -> Result<Option<ShaderNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();

        // Short inputs could be a nanoid or a short slug — check both
        let shader = if id.len() <= crate::id::ID_LENGTH {
            ShaderRepo::find_by_id(db, &id)
                .await?
                .or(ShaderRepo::find_by_slug(db, &id).await?)
        } else {
            ShaderRepo::find_by_slug(db, &id).await?
        };

        // Fall back to the slug redirect table for old/renamed slugs
        let shader = match shader {
            Some(s) => s,
            None => {
                if let Some(entity_id) = SlugRedirectRepo::find_entity_id(db, "shader", &id).await?
                {
                    match ShaderRepo::find_by_id(db, &entity_id).await? {
                        Some(s) => s,
                        None => return Ok(None),
                    }
                } else {
                    return Ok(None);
                }
            }
        };

        Ok(Some(shader.into()))
    }

    /// Trending shaders by recent view count.
    async fn trending_shaders(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 7, desc = "Number of days to consider (1-90)")] days: u32,
        #[graphql(default = 20, desc = "Number of items to return (max 100)")] first: i32,
    ) -> Result<Vec<TrendingShaderNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let days = days.clamp(1, 90);
        let page = Page {
            page: 1,
            page_size: (first as u32).clamp(1, 100),
            offset: 0,
        };
        let result = ShaderService::list_trending(state.db(), days, &page).await?;
        Ok(result.into_iter().map(Into::into).collect())
    }
}
