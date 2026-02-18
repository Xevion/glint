use async_graphql::{Context, Object, Result};

use crate::error::AppError;
use crate::graphql::types::capture::CaptureConnection;
use crate::graphql::types::connection::decode_cursor;
use crate::repo::CaptureRepo;
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
    ) -> Result<CaptureConnection> {
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

        let page = CaptureRepo::list_items_cursor(state.db(), first, decoded_after).await?;
        Ok(page.into())
    }
}
