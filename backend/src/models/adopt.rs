use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Request to preview or adopt a shader from a platform URL
#[derive(Debug, Deserialize)]
pub struct AdoptShaderRequest {
    pub url: String,
}

/// Request to link an additional platform to an existing shader
#[derive(Debug, Deserialize)]
pub struct LinkShaderRequest {
    pub url: String,
}

/// Preview response before confirming adoption
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AdoptPreviewResponse {
    pub platform: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub version_count: usize,
    pub authors: Vec<AdoptPreviewAuthor>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AdoptPreviewAuthor {
    pub name: String,
    pub url: Option<String>,
}
