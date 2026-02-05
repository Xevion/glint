mod facets;
mod types;

pub use facets::{Facet, ProjectType};
pub use types::*;

use crate::platform::{PlatformResult, RequestBuilderExt};

/// Modrinth API v2 client. No authentication needed for read operations.
pub struct ModrinthClient {
    client: reqwest::Client,
}

impl ModrinthClient {
    const BASE_URL: &str = "https://api.modrinth.com/v2";

    pub fn new(user_agent: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("failed to create HTTP client"),
        }
    }

    /// Search for shader projects
    pub async fn search_shaders(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
    ) -> PlatformResult<SearchResponse> {
        let facets = serde_json::to_string(&vec![vec![Facet::ProjectType(ProjectType::Shader)]])
            .expect("facet serialization cannot fail");

        self.client
            .get(format!("{}/search", Self::BASE_URL))
            .query(&[
                ("query", query),
                ("facets", &facets),
                ("offset", &offset.to_string()),
                ("limit", &limit.to_string()),
            ])
            .send_and_parse()
            .await
    }

    /// Get a single project by ID or slug
    pub async fn get_project(&self, id_or_slug: &str) -> PlatformResult<Project> {
        self.client
            .get(format!("{}/project/{}", Self::BASE_URL, id_or_slug))
            .send_and_parse()
            .await
    }

    /// Get multiple projects by ID
    pub async fn get_projects(&self, ids: &[&str]) -> PlatformResult<Vec<Project>> {
        let ids_json = serde_json::to_string(ids).expect("ids serialization cannot fail");
        self.client
            .get(format!("{}/projects", Self::BASE_URL))
            .query(&[("ids", &ids_json)])
            .send_and_parse()
            .await
    }

    /// List all versions for a project, optionally filtered by game version or loader
    pub async fn list_versions(
        &self,
        project_id: &str,
        game_versions: Option<&[&str]>,
        loaders: Option<&[&str]>,
    ) -> PlatformResult<Vec<Version>> {
        let mut req = self
            .client
            .get(format!("{}/project/{}/version", Self::BASE_URL, project_id));

        if let Some(gv) = game_versions {
            let json = serde_json::to_string(gv).expect("game_versions serialization cannot fail");
            req = req.query(&[("game_versions", &json)]);
        }
        if let Some(l) = loaders {
            let json = serde_json::to_string(l).expect("loaders serialization cannot fail");
            req = req.query(&[("loaders", &json)]);
        }

        req.send_and_parse().await
    }

    /// Get a single version by ID
    pub async fn get_version(&self, version_id: &str) -> PlatformResult<Version> {
        self.client
            .get(format!("{}/version/{}", Self::BASE_URL, version_id))
            .send_and_parse()
            .await
    }

    /// Get team members (authors) for a project
    pub async fn get_team_members(&self, project_id: &str) -> PlatformResult<Vec<TeamMember>> {
        self.client
            .get(format!("{}/project/{}/members", Self::BASE_URL, project_id))
            .send_and_parse()
            .await
    }
}
