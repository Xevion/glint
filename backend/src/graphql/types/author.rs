use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use chrono::{DateTime, Utc};

use crate::graphql::types::connection::{
    Connection, CursorPayload, CursorSource, decode_cursor_for_query,
};
use crate::graphql::types::shader::ShaderNode;
use crate::models::shader::AuthorAggregate;
use crate::repo::ShaderAuthorRepo;
use crate::slug::slugify;
use crate::state::AppState;

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct AuthorNode {
    pub name: String,
    pub slug: String,
    pub shader_count: i64,
    pub total_views: i64,
    pub total_captures: i64,
    pub last_modified: DateTime<Utc>,
    pub image_path: Option<String>,
    pub thumbhash: Option<String>,
    pub top_shader_name: Option<String>,
    pub top_shader_slug: Option<String>,
}

#[ComplexObject]
impl AuthorNode {
    /// Paginated shaders by this author.
    async fn shaders(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 24, desc = "Number of items to return.")] first: i32,
        #[graphql(desc = "Cursor to paginate after.")] after: Option<String>,
    ) -> Result<Connection<ShaderNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let decoded_after = after.map(|c| decode_cursor_for_query(&c)).transpose()?;
        let page = ShaderAuthorRepo::list_shaders_by_author_cursor(
            state.db(),
            &self.name,
            first,
            decoded_after,
        )
        .await?;
        Ok(page.into_connection())
    }

    /// Platform links (Modrinth/CurseForge profile URLs) for this author.
    async fn platforms(&self, ctx: &Context<'_>) -> Result<Vec<AuthorPlatformLink>> {
        let state = ctx.data_unchecked::<AppState>();
        let links = ShaderAuthorRepo::list_platforms_by_author(state.db(), &self.name).await?;
        Ok(links
            .into_iter()
            .map(|(platform, url)| AuthorPlatformLink { platform, url })
            .collect())
    }
}

#[derive(SimpleObject, Debug, Clone)]
pub struct AuthorPlatformLink {
    pub platform: String,
    pub url: Option<String>,
}

impl From<AuthorAggregate> for AuthorNode {
    fn from(a: AuthorAggregate) -> Self {
        let slug = slugify(&a.name);
        Self {
            name: a.name,
            slug,
            shader_count: a.shader_count,
            total_views: a.total_views,
            total_captures: a.total_captures,
            last_modified: a.last_modified,
            image_path: a.image_path,
            thumbhash: a.thumbhash,
            top_shader_name: a.top_shader_name,
            top_shader_slug: a.top_shader_slug,
        }
    }
}

impl CursorSource for AuthorAggregate {
    fn to_cursor(&self) -> CursorPayload {
        // Encode total_views in the timestamp field for keyset pagination
        CursorPayload::new(&self.name, self.total_views)
    }
}
