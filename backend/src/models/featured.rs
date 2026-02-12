use serde::Serialize;
use serde_with::skip_serializing_none;
use ts_rs::TS;

/// A before/after pair for the homepage hero slider.
/// Dynamically selected from popular shaders.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, optional_fields)]
pub struct FeaturedPair {
    pub left_image_url: String,
    pub left_thumbhash: Option<String>,
    pub left_shader_name: String,
    pub left_shader_slug: String,
    pub left_shader_author: Option<String>,
    pub left_shader_version: String,
    pub left_scene_name: String,
    pub right_image_url: String,
    pub right_thumbhash: Option<String>,
    pub right_shader_name: String,
    pub right_shader_slug: String,
    pub right_shader_author: Option<String>,
    pub right_shader_version: String,
    pub right_scene_name: String,
}
