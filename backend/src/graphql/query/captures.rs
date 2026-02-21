use async_graphql::{Context, Object, Result};

use crate::graphql::guard::AdminGuard;
use crate::graphql::types::capture::{
    AdminCaptureFiltersInput, CaptureNode, CaptureWithContextNode,
};
use crate::graphql::types::capture_health::CaptureHealthNode;
use crate::graphql::types::connection::{Connection, decode_cursor_for_query};
use crate::models::{CaptureStatus, PageQuery};
use crate::repo::capture::{CaptureDistinct, CaptureFilters};
use crate::repo::{CaptureHealthRepo, CaptureRepo};
use crate::state::AppState;

#[derive(Default)]
pub struct CaptureQuery;

#[Object]
impl CaptureQuery {
    /// Paginated list of completed captures (public).
    async fn captures(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
    ) -> Result<Connection<CaptureNode>> {
        let state = ctx.data_unchecked::<AppState>();

        let decoded_after = after.map(|c| decode_cursor_for_query(&c)).transpose()?;

        let page = CaptureRepo::list_items_cursor(state.db(), first, decoded_after).await?;
        Ok(page.into_connection())
    }

    /// Capture health summary — target matrix completion status (admin).
    #[graphql(guard = "AdminGuard")]
    async fn capture_health(&self, ctx: &Context<'_>) -> Result<CaptureHealthNode> {
        let state = ctx.data_unchecked::<AppState>();
        let health = CaptureHealthRepo::get_capture_health(state.db()).await?;
        Ok(health.summary.into())
    }

    /// Recent completed captures with full context (admin).
    #[graphql(guard = "AdminGuard")]
    async fn recent_captures(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 5, desc = "Number of recent captures to return")] count: i32,
    ) -> Result<Vec<CaptureWithContextNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let filters = CaptureFilters {
            statuses: Some(vec![CaptureStatus::Completed]),
            ..Default::default()
        };
        let page = PageQuery {
            page: Some(1),
            page_size: Some(count as u32),
        }
        .normalize();
        let (captures, _) = CaptureRepo::list_with_context(
            state.db(),
            &filters,
            Some(&page),
            CaptureDistinct::None,
        )
        .await?;
        Ok(captures.into_iter().map(Into::into).collect())
    }

    /// Cursor-paginated list of captures with rich filtering (admin).
    #[graphql(guard = "AdminGuard")]
    async fn admin_captures(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 25, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
        #[graphql(desc = "Sort: capturedAt, fileSizeBytes, shaderName (prefix - for desc)")]
        sort: Option<String>,
        filters: Option<AdminCaptureFiltersInput>,
    ) -> Result<Connection<CaptureWithContextNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let decoded_after = after.map(|c| decode_cursor_for_query(&c)).transpose()?;

        let capture_filters: CaptureFilters = filters.unwrap_or_default().into();

        let page = CaptureRepo::list_with_context_cursor(
            state.db(),
            &capture_filters,
            first,
            decoded_after,
            CaptureDistinct::None,
            sort.as_deref(),
        )
        .await?;

        Ok(page.into_connection())
    }
}
