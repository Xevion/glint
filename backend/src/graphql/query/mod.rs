use async_graphql::{Context, MergedObject, Object, Result};

use crate::graphql::types::connection::decode_cursor;
use crate::graphql::types::scene::{SceneConnection, SceneNode};
use crate::repo::SceneRepo;
use crate::state::AppState;

/// Root query type — merges all domain query types.
#[derive(MergedObject, Default)]
pub struct QueryRoot(SceneQuery);

#[derive(Default)]
pub struct SceneQuery;

#[Object]
impl SceneQuery {
    /// List active scenes with cursor-based pagination.
    async fn scenes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
    ) -> Result<SceneConnection> {
        let state = ctx.data_unchecked::<AppState>();

        let decoded_after = after
            .map(|c| decode_cursor(&c))
            .transpose()?
            .map(|(id, ts)| {
                let dt = chrono::DateTime::from_timestamp_millis(ts).ok_or_else(|| {
                    crate::error::AppError::BadRequest("Invalid cursor timestamp".into())
                })?;
                Ok::<_, crate::error::AppError>((dt, id))
            })
            .transpose()?;

        let page = SceneRepo::list_active_cursor(state.db(), first, decoded_after).await?;
        Ok(page.into())
    }

    /// Get a single scene by ID or slug.
    async fn scene(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Scene ID (nanoid) or slug")] id: String,
    ) -> Result<Option<SceneNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let db = state.db();

        let scene = SceneRepo::find_by_id(db, &id).await?;
        if let Some(s) = scene {
            return Ok(Some(s.into()));
        }

        let scenes = SceneRepo::find_active_by_slug(db, &id).await?;
        Ok(scenes.into_iter().next().map(Into::into))
    }
}
