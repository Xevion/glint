mod class_id;
mod types;

pub use class_id::ClassId;
pub use types::*;

use std::marker::PhantomData;

use crate::platform::{PlatformResult, RequestBuilderExt};

pub struct Authenticated;

/// CurseForge API client. Requires an API key for all operations.
pub struct CurseForgeClient<Auth = Authenticated> {
    client: reqwest::Client,
    _auth: PhantomData<Auth>,
}

impl CurseForgeClient<Authenticated> {
    const BASE_URL: &str = "https://api.curseforge.com/v1";
    const MINECRAFT_GAME_ID: i32 = 432;

    pub fn new(api_key: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            api_key.parse().expect("invalid CurseForge API key"),
        );

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .expect("failed to create HTTP client"),
            _auth: PhantomData,
        }
    }

    /// Search for shader packs
    pub async fn search_shaders(
        &self,
        query: &str,
        page_size: u32,
        index: u32,
    ) -> PlatformResult<CfSearchResponse> {
        self.client
            .get(format!("{}/mods/search", Self::BASE_URL))
            .query(&[
                ("gameId", &Self::MINECRAFT_GAME_ID.to_string()),
                ("classId", &(ClassId::Shaders as u16).to_string()),
                ("searchFilter", &query.to_string()),
                ("pageSize", &page_size.to_string()),
                ("index", &index.to_string()),
            ])
            .send_and_parse()
            .await
    }

    /// Get a single mod by ID
    pub async fn get_mod(&self, mod_id: i32) -> PlatformResult<CfMod> {
        let resp: CfResponse<CfMod> = self
            .client
            .get(format!("{}/mods/{}", Self::BASE_URL, mod_id))
            .send_and_parse()
            .await?;
        Ok(resp.data)
    }

    /// Get multiple mods by IDs
    pub async fn get_mods(&self, mod_ids: &[i32]) -> PlatformResult<Vec<CfMod>> {
        let resp: CfResponse<Vec<CfMod>> = self
            .client
            .post(format!("{}/mods", Self::BASE_URL))
            .json(&serde_json::json!({ "modIds": mod_ids }))
            .send_and_parse()
            .await?;
        Ok(resp.data)
    }

    /// List all files for a mod
    pub async fn list_files(
        &self,
        mod_id: i32,
        game_version: Option<&str>,
    ) -> PlatformResult<Vec<CfFile>> {
        let mut req = self
            .client
            .get(format!("{}/mods/{}/files", Self::BASE_URL, mod_id))
            .query(&[("pageSize", "10000")]);

        if let Some(gv) = game_version {
            req = req.query(&[("gameVersion", gv)]);
        }

        let resp: CfResponse<Vec<CfFile>> = req.send_and_parse().await?;
        Ok(resp.data)
    }

    /// Get a single file
    pub async fn get_file(&self, mod_id: i32, file_id: i32) -> PlatformResult<CfFile> {
        let resp: CfResponse<CfFile> = self
            .client
            .get(format!(
                "{}/mods/{}/files/{}",
                Self::BASE_URL,
                mod_id,
                file_id
            ))
            .send_and_parse()
            .await?;
        Ok(resp.data)
    }

    /// Get download URL for a file
    pub async fn get_download_url(&self, mod_id: i32, file_id: i32) -> PlatformResult<String> {
        let resp: CfResponse<String> = self
            .client
            .get(format!(
                "{}/mods/{}/files/{}/download-url",
                Self::BASE_URL,
                mod_id,
                file_id
            ))
            .send_and_parse()
            .await?;
        Ok(resp.data)
    }

    /// Get mod HTML description
    pub async fn get_description(&self, mod_id: i32) -> PlatformResult<String> {
        let resp: CfResponse<String> = self
            .client
            .get(format!("{}/mods/{}/description", Self::BASE_URL, mod_id))
            .send_and_parse()
            .await?;
        Ok(resp.data)
    }

    /// Search by slug (convenience: searches and finds exact match)
    pub async fn get_mod_by_slug(&self, slug: &str) -> PlatformResult<Option<CfMod>> {
        let results = self.search_shaders(slug, 50, 0).await?;
        Ok(results.data.into_iter().find(|m| m.slug == slug))
    }
}
