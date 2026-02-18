use serde::Serialize;
use serde_with::skip_serializing_none;
use ts_rs::TS;

/// One side of a featured comparison pair (shader + capture context).
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct FeaturedSide {
    pub image_path: String,
    pub thumbhash: Option<String>,
    pub shader_name: String,
    pub shader_slug: String,
    pub shader_author: Option<String>,
    pub shader_version: String,
    pub scene_name: String,
}

/// A before/after pair for the homepage hero slider.
/// Dynamically selected from popular shaders.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct FeaturedPair {
    pub left: FeaturedSide,
    pub right: FeaturedSide,
}
