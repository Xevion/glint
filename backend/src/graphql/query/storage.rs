use async_graphql::{Context, Object, Result};

use crate::graphql::guard::AdminGuard;
use crate::graphql::types::storage::{StorageBucketNode, StorageStatsNode};
use crate::repo::CaptureRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct StorageQuery;

#[Object]
impl StorageQuery {
    /// Storage statistics — total bytes, counts, averages (admin).
    #[graphql(guard = "AdminGuard")]
    async fn storage_stats(&self, ctx: &Context<'_>) -> Result<StorageStatsNode> {
        let state = ctx.data_unchecked::<AppState>();
        let stats = CaptureRepo::storage_stats(state.db()).await?;
        Ok(stats.into())
    }

    /// Storage growth time series (admin).
    #[graphql(guard = "AdminGuard")]
    async fn storage_growth(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 90, desc = "Number of days to look back")] days: i32,
        #[graphql(default = 1, desc = "Interval in hours for each bucket")] interval_hours: i32,
    ) -> Result<Vec<StorageBucketNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let buckets = CaptureRepo::storage_growth(state.db(), days, interval_hours).await?;
        Ok(buckets.into_iter().map(Into::into).collect())
    }
}
