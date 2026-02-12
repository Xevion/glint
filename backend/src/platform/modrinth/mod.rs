mod facets;
mod types;

pub use facets::{Facet, ProjectType};
pub use types::*;

use crate::platform::{
    PlatformAuthor, PlatformMetadata, PlatformResult, PlatformVersion, ProjectData,
    RequestBuilderExt,
};

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
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to create HTTP client"),
        }
    }

    /// Search for shader projects.
    /// `index` controls sort order: "relevance" (default for search), "downloads", "newest"
    pub async fn search_shaders(
        &self,
        query: &str,
        offset: u32,
        limit: u32,
        index: Option<&str>,
    ) -> PlatformResult<SearchResponse> {
        let facets = serde_json::to_string(&vec![vec![Facet::ProjectType(ProjectType::Shader)]])
            .expect("facet serialization cannot fail");

        let mut req = self
            .client
            .get(format!("{}/search", Self::BASE_URL))
            .query(&[
                ("query", query),
                ("facets", &facets),
                ("offset", &offset.to_string()),
                ("limit", &limit.to_string()),
            ]);

        if let Some(idx) = index {
            req = req.query(&[("index", idx)]);
        }

        req.send_and_parse().await
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

    /// Fetch project metadata and authors as platform-agnostic types
    pub async fn fetch_shader_data(&self, id_or_slug: &str) -> PlatformResult<ProjectData> {
        let project = self.get_project(id_or_slug).await?;
        let members = match self.get_team_members(&project.id).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(project_id = %project.id, error = %e, "Failed to fetch team members, proceeding without authors");
                Vec::new()
            }
        };

        Ok(ProjectData {
            metadata: PlatformMetadata {
                platform_id: project.id,
                name: project.title,
                slug: project.slug,
                description: Some(project.description),
                icon_url: project.icon_url,
                source_url: project.source_url.clone(),
                website_url: project.source_url,
                license_id: project.license.map(|l| l.id),
                downloads: project.downloads,
                updated_at: Some(project.updated),
            },
            authors: members
                .into_iter()
                .map(|m| PlatformAuthor {
                    url: Some(format!("https://modrinth.com/user/{}", m.user.username)),
                    name: m.user.username,
                })
                .collect(),
        })
    }

    /// Fetch versions as platform-agnostic types
    pub async fn fetch_shader_versions(
        &self,
        platform_id: &str,
    ) -> PlatformResult<Vec<PlatformVersion>> {
        let versions = self.list_versions(platform_id, None, None).await?;
        Ok(versions
            .into_iter()
            .map(|v| {
                let primary_file = v.files.iter().find(|f| f.primary).or(v.files.first());
                PlatformVersion {
                    version_number: v.version_number,
                    modrinth_version_id: Some(v.id),
                    curseforge_file_id: None,
                    download_url: primary_file.map(|f| f.url.clone()),
                    file_hash: primary_file.map(|f| f.hashes.sha1.clone()),
                    file_size: primary_file.map(|f| f.size as i64),
                    game_versions: if v.game_versions.is_empty() {
                        None
                    } else {
                        Some(v.game_versions)
                    },
                    release_channel: Some(format!("{:?}", v.version_type).to_lowercase()),
                    published_at: Some(v.date_published),
                }
            })
            .collect())
    }
}
