use async_graphql::{Context, Object, Result};

use crate::graphql::types::stats::StatsNode;
use crate::repo::StatsRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct StatsQuery;

#[Object]
impl StatsQuery {
    /// Aggregate counts for the index page.
    async fn stats(&self, ctx: &Context<'_>) -> Result<StatsNode> {
        let state = ctx.data_unchecked::<AppState>();
        let stats = StatsRepo::get(state.db()).await?;
        Ok(stats.into())
    }
}
