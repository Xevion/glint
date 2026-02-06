use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    pub database_url: String,

    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,

    /// R2/S3 configuration for screenshot storage
    #[serde(default)]
    pub r2: R2Config,

    /// Discord OAuth configuration
    #[serde(default)]
    pub discord: DiscordConfig,

    /// Platform integration configuration (Modrinth, CurseForge)
    #[serde(default)]
    pub platform: PlatformConfig,
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
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:5173".to_string()]
}

/// Discord OAuth configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DiscordConfig {
    /// Discord application client ID
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

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("GLINT_"))
            // DATABASE_URL is commonly set by tools like docker-compose, so read it directly
            .merge(
                Env::raw()
                    .only(&["DATABASE_URL"])
                    .map(|_| "database_url".into()),
            )
            .extract()?;

        // Load R2 config from env vars directly (secrets - avoid logging)
        config.r2 = R2Config {
            account_id: std::env::var("GLINT_R2_ACCOUNT_ID").ok(),
            bucket: std::env::var("GLINT_R2_BUCKET").ok(),
            access_key_id: std::env::var("GLINT_R2_ACCESS_KEY_ID").ok(),
            secret_access_key: std::env::var("GLINT_R2_SECRET_ACCESS_KEY").ok(),
            public_url: std::env::var("GLINT_R2_PUBLIC_URL").ok(),
        };

        // Load Discord config from env vars directly (secrets - avoid logging)
        config.discord = DiscordConfig {
            client_id: std::env::var("GLINT_DISCORD_CLIENT_ID").ok(),
            client_secret: std::env::var("GLINT_DISCORD_CLIENT_SECRET").ok(),
            redirect_uri: std::env::var("GLINT_DISCORD_REDIRECT_URI").ok(),
        };

        // Load platform config from env vars directly (secrets - avoid logging)
        if let Ok(key) = std::env::var("GLINT_CURSEFORGE_API_KEY") {
            config.platform.curseforge_api_key = Some(key);
        }
        if let Ok(ua) = std::env::var("GLINT_MODRINTH_USER_AGENT") {
            config.platform.modrinth_user_agent = ua;
        }

        Ok(config)
    }
}
