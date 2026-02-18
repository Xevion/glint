use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use chrono::{DateTime, Utc};

use crate::id::SceneId;
use crate::models::Scene;
use crate::repo::{CaptureRepo, ScenePresetRepo, SceneVersionRepo};
use crate::state::AppState;

use super::connection::{CursorEncodable, CursorPage, PageInfo, encode_cursor};
use super::preset::PresetNode;
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
        let state = ctx.data_unchecked::<AppState>();
        let presets = ScenePresetRepo::list_by_scene(state.db(), self.id.as_ref()).await?;
        Ok(presets.into_iter().map(Into::into).collect())
    }

    /// Representative thumbnail image path for this scene.
    async fn image_path(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let state = ctx.data_unchecked::<AppState>();
        let thumb = CaptureRepo::thumbnail_for_scene(state.db(), self.id.as_ref()).await?;
        Ok(thumb.map(|t| t.image_path))
    }

    /// Thumbhash placeholder for the representative thumbnail.
    async fn thumbhash(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let state = ctx.data_unchecked::<AppState>();
        let thumb = CaptureRepo::thumbnail_for_scene(state.db(), self.id.as_ref()).await?;
        Ok(thumb.and_then(|t| t.thumbhash))
    }

    /// Number of completed captures for this scene.
    async fn capture_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let state = ctx.data_unchecked::<AppState>();
        Ok(CaptureRepo::count_for_scene(state.db(), self.id.as_ref()).await?)
    }

    /// Latest version (camera position, world settings) for this scene.
    async fn version(&self, ctx: &Context<'_>) -> Result<Option<SceneVersionNode>> {
        let state = ctx.data_unchecked::<AppState>();
        let version = SceneVersionRepo::get_latest(state.db(), self.id.as_ref()).await?;
        Ok(version.map(Into::into))
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
