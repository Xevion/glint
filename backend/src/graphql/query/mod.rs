pub mod authors;
pub mod backgrounds;
pub mod captures;
pub mod featured;
pub mod runs;
pub mod shaders;
pub mod stats;
pub mod storage;

use async_graphql::{Context, MergedObject, Object, Result};

use crate::graphql::guard::require_admin_for_visibility;
use crate::graphql::types::common::Visibility;
use crate::graphql::types::connection::{Connection, decode_cursor_as_datetime};
use crate::graphql::types::scene::SceneNode;
use crate::repo::SceneRepo;
use crate::state::AppState;

use authors::AuthorQuery;
use backgrounds::BackgroundQuery;
use captures::CaptureQuery;
use featured::FeaturedQuery;
use runs::RunQuery;
use shaders::ShaderQuery;
use stats::StatsQuery;
use storage::StorageQuery;

/// Root query type — merges all domain query types.
#[derive(MergedObject, Default)]
pub struct QueryRoot(
    SceneQuery,
    ShaderQuery,
    CaptureQuery,
    StatsQuery,
    FeaturedQuery,
    BackgroundQuery,
    RunQuery,
    StorageQuery,
    AuthorQuery,
);

#[derive(Default)]
pub struct SceneQuery;

#[Object]
impl SceneQuery {
    /// List scenes with cursor-based pagination.
    async fn scenes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
        #[graphql(
            default_with = "Visibility::Exclude",
            desc = "Filter inactive scenes: EXCLUDE (default, active only), INCLUDE (all, admin), ONLY (inactive only, admin)"
        )]
        visibility: Visibility,
    ) -> Result<Connection<SceneNode>> {
        require_admin_for_visibility(ctx, visibility)?;
        let state = ctx.data_unchecked::<AppState>();

        let decoded_after = after.map(|c| decode_cursor_as_datetime(&c)).transpose()?;

        let page = SceneRepo::list_cursor(state.db(), first, decoded_after, visibility).await?;
        Ok(page.into_connection())
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
