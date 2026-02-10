use serde::Serialize;
use ts_rs::TS;

/// A before/after pair for the homepage hero slider.
/// Dynamically selected from popular shaders.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct FeaturedPair {
    pub left_image_url: String,
    pub left_thumbhash: Option<String>,
    pub left_shader_name: String,
    pub left_scene_name: String,
    pub right_image_url: String,
    pub right_thumbhash: Option<String>,
    pub right_shader_name: String,
    pub right_scene_name: String,
}
