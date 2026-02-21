use async_graphql::{Context, Object, Result};

use crate::graphql::types::author::AuthorNode;
use crate::graphql::types::connection::{Connection, decode_cursor_for_query};
use crate::repo::ShaderAuthorRepo;
use crate::state::AppState;

#[derive(Default)]
pub struct AuthorQuery;

#[Object]
impl AuthorQuery {
    /// Paginated list of authors aggregated across all shaders.
    async fn authors(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return (max 100)")] first: i32,
        #[graphql(desc = "Cursor from a previous page's endCursor")] after: Option<String>,
        #[graphql(desc = "Search by author name")] search: Option<String>,
        #[graphql(desc = "Sort: popular (default), shaders, or name")] sort: Option<String>,
    ) -> Result<Connection<AuthorNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let decoded_after = after.map(|c| decode_cursor_for_query(&c)).transpose()?;

        let page = ShaderAuthorRepo::list_authors_cursor(
            state.db(),
            first,
            decoded_after,
            search.as_deref(),
            sort.as_deref(),
        )
        .await?;

        Ok(page.into_connection())
    }

    /// Get a single author by slugified name.
    async fn author(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Author slug (slugified name)")] slug: String,
    ) -> Result<Option<AuthorNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let author = ShaderAuthorRepo::find_by_slug(state.db(), &slug).await?;
        Ok(author.map(Into::into))
    }
}
