use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::{Deserialize, Deserializer};

use crate::middleware::rate_limit::RateLimitConfig;

/// Accept a string or an integer, coercing to `Option<String>`.
///
/// Figment infers types from env var values — pure-digit strings like Discord
/// snowflake IDs (`1468509036625531012`) become u64 instead of String.
fn deserialize_optional_string_or_int<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Str(String),
        Int(u64),
    }
    Ok(Option::<StringOrInt>::deserialize(de)?.map(|v| match v {
        StringOrInt::Str(s) => s,
        StringOrInt::Int(n) => n.to_string(),
    }))
}

/// Accept either a sequence `["a", "b"]` or a comma-separated string `"a,b"`.
fn deserialize_string_or_seq<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrSeq {
        Seq(Vec<String>),
        Str(String),
    }
    match StringOrSeq::deserialize(de)? {
        StringOrSeq::Seq(v) => Ok(v),
        StringOrSeq::Str(s) => Ok(s.split(',').map(|s| s.trim().to_string()).collect()),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    pub database_url: String,

    #[serde(
        default = "default_cors_origins",
        deserialize_with = "deserialize_string_or_seq"
    )]
    pub cors_origins: Vec<String>,

    /// R2/S3 configuration for capture image storage
    #[serde(default)]
    pub r2: R2Config,

    /// Discord OAuth configuration
    #[serde(default)]
    pub discord: DiscordConfig,

    /// Platform integration configuration (Modrinth, CurseForge)
    #[serde(default)]
    pub platform: PlatformConfig,

    /// PostHog analytics configuration
    #[serde(default)]
    pub posthog: PostHogConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct R2Config {
    /// R2 account ID (used to construct endpoint)
    pub account_id: Option<String>,

    /// R2 bucket name
    pub bucket: Option<String>,

    /// R2 access key ID
    pub access_key_id: Option<String>,

    /// R2 secret access key
    pub secret_access_key: Option<String>,

    /// Public URL prefix for serving files (e.g., https://cdn.glint.example.com)
    pub public_url: Option<String>,
}

impl R2Config {
    /// Returns true if R2 is fully configured
    pub fn is_configured(&self) -> bool {
        self.account_id.is_some()
            && self.bucket.is_some()
            && self.access_key_id.is_some()
            && self.secret_access_key.is_some()
    }

    /// R2 endpoint URL
    pub fn endpoint(&self) -> Option<String> {
        self.account_id
            .as_ref()
            .map(|id| format!("https://{}.r2.cloudflarestorage.com", id))
    }

    /// Build a public URL for a given R2 object key
    pub fn public_url_for_key(&self, key: &str) -> String {
        if let Some(ref public_url_prefix) = self.public_url {
            format!("{}/{}", public_url_prefix, key)
        } else {
            let bucket = self.bucket.as_deref().unwrap_or("glint");
            format!("https://{}.r2.cloudflarestorage.com/{}", bucket, key)
        }
    }

    /// Extract the R2 object key from a public URL (inverse of `public_url_for_key`).
    ///
    /// Given `https://pub-xxx.r2.dev/captures/shader/scene/id.webp`,
    /// returns `captures/shader/scene/id.webp`.
    pub fn key_from_url(&self, url: &str) -> String {
        if let Some(ref prefix) = self.public_url {
            let trimmed = prefix.trim_end_matches('/');
            if let Some(key) = url.strip_prefix(trimmed) {
                return key.trim_start_matches('/').to_string();
            }
        }
        // Fallback: take everything after the host (third slash)
        url.find("://")
            .and_then(|i| url[i + 3..].find('/'))
            .map(|i| {
                let start = url.find("://").unwrap() + 3 + i + 1;
                url[start..].to_string()
            })
            .unwrap_or_else(|| url.to_string())
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_cors_origins() -> Vec<String> {
    vec![
        "http://localhost:5173".to_string(),
        "http://localhost:8080".to_string(),
    ]
}

/// Discord OAuth configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DiscordConfig {
    /// Discord application client ID (snowflake — always numeric)
    #[serde(default, deserialize_with = "deserialize_optional_string_or_int")]
    pub client_id: Option<String>,

    /// Discord application client secret
    pub client_secret: Option<String>,

    /// OAuth callback URL (e.g., "http://localhost:8080/api/auth/discord/callback")
    pub redirect_uri: Option<String>,
}

impl DiscordConfig {
    pub fn is_configured(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some() && self.redirect_uri.is_some()
    }
}

/// Platform API configuration for Modrinth and CurseForge
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PlatformConfig {
    /// CurseForge API key (obtain from https://console.curseforge.com/)
    pub curseforge_api_key: Option<String>,

    /// User-Agent for Modrinth API requests
    #[serde(default = "default_modrinth_user_agent")]
    pub modrinth_user_agent: String,
}

impl PlatformConfig {
    pub fn curseforge_configured(&self) -> bool {
        self.curseforge_api_key.is_some()
    }
}

fn default_modrinth_user_agent() -> String {
    "glint/0.1.0 (https://github.com/Xevion/glint)".to_string()
}

/// PostHog analytics configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PostHogConfig {
    /// PostHog project API key
    pub api_key: Option<String>,

    /// PostHog API host (defaults to https://observe.xevion.dev)
    #[serde(default = "default_posthog_host")]
    pub host: String,
}

fn default_posthog_host() -> String {
    "https://observe.xevion.dev".to_string()
}

impl PostHogConfig {
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }
}

/// Known nested config prefixes. When a GLINT_ env var starts with one of
/// these, the prefix is replaced with a dotted path so Figment maps it to
/// the correct nested struct field (e.g. `r2_account_id` → `r2.account_id`).
const NESTED_PREFIXES: &[&str] = &["r2_", "discord_", "platform_", "posthog_", "rate_limit_"];

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("GLINT_").map(|key| {
                let key_lower = key.as_str().to_ascii_lowercase();
                for prefix in NESTED_PREFIXES {
                    if let Some(rest) = key_lower.strip_prefix(prefix) {
                        let section = prefix.trim_end_matches('_');
                        return format!("{section}.{rest}").into();
                    }
                }
                key_lower.into()
            }))
            // DATABASE_URL is commonly set by tools like docker-compose, so read it directly
            .merge(
                Env::raw()
                    .only(&["DATABASE_URL"])
                    .map(|_| "database_url".into()),
            )
            .extract()?;

        // If ORIGIN is set (shared with the SvelteKit frontend), ensure it's
        // included in the allowed CORS origins so SSR fetch requests succeed.
        if let Ok(origin) = std::env::var("ORIGIN")
            && !config.cors_origins.contains(&origin)
        {
            config.cors_origins.push(origin);
        }

        Ok(config)
    }
}
