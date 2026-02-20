use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use chrono::{DateTime, Utc};

use crate::graphql::loaders::RequestLoaders;
use crate::id::SceneId;
use crate::models::Scene;
use crate::models::capture::CaptureStatus;
use crate::repo::CaptureRepo;
use crate::repo::capture::{CaptureDistinct, CaptureFilters};
use crate::state::AppState;

use super::capture::CaptureWithContextConnection;
use super::connection::{
    CursorEncodable, CursorPage, PageInfo, decode_cursor_for_query, encode_cursor,
};
use super::preset::PresetNode;
use super::taxonomy::TagNode;
use super::version::SceneVersionNode;

/// GraphQL representation of a Scene.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct SceneNode {
    pub id: SceneId,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub dimension: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[ComplexObject]
impl SceneNode {
    /// Presets for this scene (time/weather variations)
    async fn presets(&self, ctx: &Context<'_>) -> Result<Vec<PresetNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let presets = loaders
            .scene_presets
            .load_one(self.id.to_string())
            .await?
            .unwrap_or_default();
        Ok(presets.into_iter().map(Into::into).collect())
    }

    /// Representative thumbnail image path for this scene.
    async fn image_path(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let thumb = loaders
            .scene_thumbnails
            .load_one(self.id.to_string())
            .await?;
        Ok(thumb.map(|t| t.image_path))
    }

    /// Thumbhash placeholder for the representative thumbnail.
    async fn thumbhash(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let thumb = loaders
            .scene_thumbnails
            .load_one(self.id.to_string())
            .await?;
        Ok(thumb.and_then(|t| t.thumbhash))
    }

    /// Number of completed captures for this scene.
    async fn capture_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let count = loaders
            .scene_capture_counts
            .load_one(self.id.to_string())
            .await?;
        Ok(count.unwrap_or(0))
    }

    /// Latest version (camera position, world settings) for this scene.
    async fn version(&self, ctx: &Context<'_>) -> Result<Option<SceneVersionNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let version = loaders
            .scene_latest_versions
            .load_one(self.id.to_string())
            .await?;
        Ok(version.map(Into::into))
    }

    /// Completed captures for this scene, one per shader (deduplicated).
    async fn captures(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20, desc = "Number of items to return.")] first: i32,
        #[graphql(desc = "Cursor to paginate after.")] after: Option<String>,
    ) -> Result<CaptureWithContextConnection> {
        let state = ctx.data_unchecked::<AppState>();
        let filters = CaptureFilters {
            scene_id: Some(self.id.clone()),
            status: Some(CaptureStatus::Completed),
            ..Default::default()
        };
        let decoded_after = after.map(|c| decode_cursor_for_query(&c)).transpose()?;
        let page = CaptureRepo::list_with_context_cursor(
            state.db(),
            &filters,
            first,
            decoded_after,
            CaptureDistinct::PerShader,
        )
        .await?;
        Ok(page.into())
    }

    /// Tags associated with this scene.
    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<TagNode>> {
        let loaders = ctx.data_unchecked::<RequestLoaders>();
        let tags = loaders
            .scene_tags
            .load_one(self.id.to_string())
            .await?
            .unwrap_or_default();
        Ok(tags.into_iter().map(Into::into).collect())
    }
}

impl From<Scene> for SceneNode {
    fn from(s: Scene) -> Self {
        Self {
            id: s.id,
            name: s.name,
            slug: s.slug,
            description: s.description,
            dimension: s.dimension,
            active: s.active,
            created_at: s.created_at,
        }
    }
}

impl CursorEncodable for Scene {
    fn encode_cursor(&self) -> String {
        encode_cursor(self.id.as_ref(), self.created_at.timestamp_millis())
    }
}

/// Concrete Relay-style edge for scenes.
#[derive(SimpleObject, Debug, Clone)]
pub struct SceneEdge {
    pub cursor: String,
    pub node: SceneNode,
}

/// Concrete Relay-style connection for scenes.
#[derive(SimpleObject, Debug, Clone)]
pub struct SceneConnection {
    pub edges: Vec<SceneEdge>,
    pub page_info: PageInfo,
    pub total_count: i64,
}

impl From<CursorPage<Scene>> for SceneConnection {
    fn from(page: CursorPage<Scene>) -> Self {
        let (edges, page_info, total_count) = page.into_connection(SceneNode::from);
        SceneConnection {
            edges: edges
                .into_iter()
                .map(|(cursor, node)| SceneEdge { cursor, node })
                .collect(),
            page_info,
            total_count,
        }
    }
}
