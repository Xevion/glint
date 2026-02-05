use serde::Deserialize;

/// Generic CurseForge API response wrapper
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfResponse<T> {
    pub data: T,
    pub pagination: Option<CfPagination>,
}

/// Pagination metadata for search responses
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfPagination {
    pub index: u32,
    pub page_size: u32,
    pub result_count: u32,
    pub total_count: u32,
}

/// Type alias for search responses
pub type CfSearchResponse = CfResponse<Vec<CfMod>>;

/// CurseForge mod metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfMod {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub class_id: Option<i32>,
    pub authors: Vec<CfModAuthor>,
    pub logo: Option<CfModAsset>,
    pub screenshots: Vec<CfModAsset>,
    pub main_file_id: i32,
    pub latest_files: Vec<CfFile>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub download_count: u64,
    pub is_available: bool,
    pub links: Option<CfModLinks>,
    pub categories: Vec<CfCategory>,
}

/// External links for a mod
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfModLinks {
    pub website_url: Option<String>,
    pub wiki_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

/// Mod author information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfModAuthor {
    pub id: i32,
    pub name: String,
    pub url: String,
}

/// Mod asset (logo, screenshot, etc.)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfModAsset {
    pub id: i32,
    pub mod_id: i32,
    pub title: String,
    pub description: String,
    pub url: String,
}

/// File metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfFile {
    pub id: i32,
    pub mod_id: i32,
    pub display_name: String,
    pub file_name: String,
    pub file_length: u64,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
    pub release_type: CfReleaseType,
    pub file_date: String,
    pub hashes: Vec<CfFileHash>,
}

/// Release type for a file
#[derive(Debug, Clone, Copy, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum CfReleaseType {
    Release = 1,
    Beta = 2,
    Alpha = 3,
}

/// File hash with algorithm identifier
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfFileHash {
    pub value: String,
    pub algo: CfHashAlgo,
}

/// Hash algorithm identifier
#[derive(Debug, Clone, Copy, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum CfHashAlgo {
    Sha1 = 1,
    Md5 = 2,
}

/// Category metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfCategory {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub class_id: Option<i32>,
}
