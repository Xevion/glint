use async_graphql::{Context, Object, Result};

use crate::graphql::types::background::BackgroundNode;
use crate::repo::BackgroundRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct BackgroundQuery;

#[Object]
impl BackgroundQuery {
    /// List enabled backgrounds (public, for frontend layout).
    async fn backgrounds(&self, ctx: &Context<'_>) -> Result<Vec<BackgroundNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let backgrounds = BackgroundRepo::list_enabled(state.db()).await?;
        Ok(backgrounds.into_iter().map(Into::into).collect())
    }

    /// List all backgrounds including disabled (admin only).
    async fn all_backgrounds(&self, ctx: &Context<'_>) -> Result<Vec<BackgroundNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let backgrounds = BackgroundRepo::list_all(state.db()).await?;
        Ok(backgrounds.into_iter().map(Into::into).collect())
    }
}
