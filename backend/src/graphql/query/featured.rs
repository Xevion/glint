use async_graphql::{Context, Object, Result};

use crate::graphql::types::featured::FeaturedPairNode;
use crate::repo::FeaturedRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct FeaturedQuery;

#[Object]
impl FeaturedQuery {
    /// Random popular shader pairs for homepage hero.
    async fn featured(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 6, desc = "Number of pairs to return")] count: i32,
    ) -> Result<Vec<FeaturedPairNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let count = (count as usize).clamp(1, 20);
        let pairs = FeaturedRepo::random_pairs(state.db(), count).await?;
        Ok(pairs.into_iter().map(Into::into).collect())
    }
}
