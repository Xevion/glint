use async_graphql::{Context, Object, Result};

use crate::graphql::guard::AdminGuard;
use crate::graphql::types::connection::{Connection, decode_cursor_for_query};
use crate::graphql::types::run::{
    AdminRunFiltersInput, CaptureRunItemNode, CaptureRunListNode, CaptureRunNode,
};
use crate::repo::CaptureRunRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct RunQuery;

#[Object]
impl RunQuery {
    /// Cursor-paginated list of capture runs with optional filtering (admin only).
    #[graphql(guard = "AdminGuard")]
    async fn admin_capture_runs(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 25, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
        #[graphql(desc = "Sort: startedAt, durationSecs, status, totalItems (prefix - for desc)")]
        sort: Option<String>,
        filters: Option<AdminRunFiltersInput>,
    ) -> Result<Connection<CaptureRunListNode>> {
        let state = ctx.data_unchecked::<AppState>();

        let decoded_after = after.map(|c| decode_cursor_for_query(&c)).transpose()?;

        let run_filters = filters.unwrap_or_default().into();

        let page = CaptureRunRepo::list_cursor(
            state.db(),
            first,
            decoded_after,
            &run_filters,
            sort.as_deref(),
        )
        .await?;

        Ok(page.into_connection())
    }

    /// Get a capture run by ID with all items (admin only).
    #[graphql(guard = "AdminGuard")]
    async fn admin_capture_run(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Capture run ID")] id: String,
    ) -> Result<Option<CaptureRunNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();

        let run = match CaptureRunRepo::get_by_id(db, &id).await {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        let items: Vec<CaptureRunItemNode> = CaptureRunRepo::list_items_with_context(db, &id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(Some((run, items).into()))
    }
}
