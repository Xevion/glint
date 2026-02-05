use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Debug, Deserialize)]
pub struct SearchHit {
    pub slug: Option<String>,
    pub title: String,
    pub description: String,
    pub project_id: String,
    pub author: String,
    pub categories: Vec<String>,
    pub downloads: u64,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub icon_url: Option<String>,
    pub versions: Vec<String>,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub license: String,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u64,
    pub followers: u64,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub icon_url: Option<String>,
    pub license: Option<License>,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub source_url: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub issues_url: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub wiki_url: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub discord_url: Option<String>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub gallery: Vec<GalleryImage>,
}

#[derive(Debug, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub version_type: VersionType,
    pub featured: bool,
    pub downloads: u64,
    pub files: Vec<VersionFile>,
    pub date_published: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Deserialize)]
pub struct VersionFile {
    pub hashes: FileHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct FileHashes {
    pub sha512: String,
    pub sha1: String,
    #[serde(flatten)]
    pub others: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamMember {
    pub user: TeamUser,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamUser {
    pub id: String,
    pub username: String,
    #[serde(deserialize_with = "deserialize_optional_url")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GalleryImage {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub ordering: i32,
}

pub(crate) fn deserialize_optional_url<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.is_empty()))
}
