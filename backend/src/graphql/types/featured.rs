use async_graphql::SimpleObject;

use crate::models::{FeaturedPair, FeaturedSide};

/// One side of a featured comparison pair.
#[derive(SimpleObject, Debug, Clone)]
pub struct FeaturedSideNode {
    pub image_path: String,
    pub thumbhash: Option<String>,
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_author: Option<String>,
    pub shader_version: String,
    pub scene_name: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct FeaturedPairNode {
    pub left: FeaturedSideNode,
    pub right: FeaturedSideNode,
}

impl From<FeaturedSide> for FeaturedSideNode {
    fn from(s: FeaturedSide) -> Self {
        Self {
            image_path: s.image_path,
            thumbhash: s.thumbhash,
            shader_name: s.shader_name,
            shader_slug: s.shader_slug,
            shader_author: s.shader_author,
            shader_version: s.shader_version,
            scene_name: s.scene_name,
        }
    }
}

impl From<FeaturedPair> for FeaturedPairNode {
    fn from(p: FeaturedPair) -> Self {
        Self {
            left: p.left.into(),
            right: p.right.into(),
        }
    }
}
