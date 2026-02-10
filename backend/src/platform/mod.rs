pub mod curseforge;
pub mod modrinth;

use chrono::{DateTime, Utc};
use reqwest::Response;
use serde::de::DeserializeOwned;
use tracing::debug;

/// Which platform a shader was sourced from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Modrinth,
    CurseForge,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Modrinth => write!(f, "modrinth"),
            Platform::CurseForge => write!(f, "curseforge"),
        }
    }
}

/// Result of parsing a pasted URL or raw identifier
#[derive(Debug)]
pub enum PlatformRef {
    Modrinth { id_or_slug: String },
    CurseForge { slug: String },
}

/// Parse a Modrinth or CurseForge URL into a PlatformRef.
///
/// Supported formats:
///   https://modrinth.com/shader/bsl-shaders
///   https://modrinth.com/shader/bsl-shaders/versions
///   https://www.curseforge.com/minecraft/shaders/bsl-shaders
///   https://www.curseforge.com/minecraft/customization/bsl-shaders
pub fn parse_platform_url(input: &str) -> Option<PlatformRef> {
    let input = input.trim();

    // Try URL parsing first
    if let Ok(url) = url::Url::parse(input) {
        let host = url.host_str()?;
        let segments: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();

        match host {
            "modrinth.com" | "www.modrinth.com" => {
                // /shader/<slug>[/versions]
                if segments.len() >= 2 && segments[0] == "shader" {
                    return Some(PlatformRef::Modrinth {
                        id_or_slug: segments[1].to_string(),
                    });
                }
            }
            "curseforge.com" | "www.curseforge.com" => {
                // /minecraft/<category>/<slug>
                if segments.len() >= 3 && segments[0] == "minecraft" {
                    return Some(PlatformRef::CurseForge {
                        slug: segments[2].to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    None
}

/// Errors from platform API calls
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),

    #[error("failed to deserialize response from {url}")]
    Deserialize {
        url: String,
        body: Vec<u8>,
        #[source]
        source: serde_json::Error,
    },

    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("resource not found")]
    NotFound,

    #[error("API returned {status}")]
    BadStatus { status: u16, body: String },
}

pub type PlatformResult<T> = Result<T, PlatformError>;

/// Platform-agnostic shader project data (metadata + authors)
pub struct ProjectData {
    pub metadata: PlatformMetadata,
    pub authors: Vec<PlatformAuthor>,
}

/// Platform-agnostic shader metadata
pub struct PlatformMetadata {
    pub platform_id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub source_url: Option<String>,
    pub website_url: Option<String>,
    pub license_id: Option<String>,
    pub downloads: u64,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A version/file from a platform, normalized to a common format
pub struct PlatformVersion {
    pub version_number: String,
    pub modrinth_version_id: Option<String>,
    pub curseforge_file_id: Option<i32>,
    pub download_url: Option<String>,
    pub file_hash: Option<String>,
    pub file_size: Option<i64>,
    pub game_versions: Option<Vec<String>>,
    pub release_channel: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

/// An author from a platform
pub struct PlatformAuthor {
    pub name: String,
    pub url: Option<String>,
}

/// Extension trait on reqwest::RequestBuilder for shared response handling
pub(crate) trait RequestBuilderExt {
    /// Send request, check status, deserialize JSON response body
    async fn send_and_parse<T: DeserializeOwned>(self) -> PlatformResult<T>;

    /// Send request, check status, return raw response
    async fn send_and_check(self) -> PlatformResult<Response>;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    async fn send_and_check(self) -> PlatformResult<Response> {
        let response = self.send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("x-ratelimit-reset")
                .or_else(|| response.headers().get("retry-after"))
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(PlatformError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(PlatformError::NotFound);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PlatformError::BadStatus {
                status: status.as_u16(),
                body,
            });
        }

        Ok(response)
    }

    async fn send_and_parse<T: DeserializeOwned>(self) -> PlatformResult<T> {
        let response = self.send_and_check().await?;
        let url = response.url().to_string();
        let bytes = response.bytes().await?;

        debug!(url = %url, bytes = bytes.len(), "Parsing platform response");

        serde_json::from_slice(&bytes).map_err(|source| PlatformError::Deserialize {
            url,
            body: bytes.to_vec(),
            source,
        })
    }
}
