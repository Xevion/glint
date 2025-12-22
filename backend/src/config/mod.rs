use std::path::PathBuf;

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

    #[serde(default = "default_database_url")]
    pub database_url: String,

    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,

    /// R2/S3 configuration for screenshot storage
    #[serde(default)]
    pub r2: R2Config,

    /// Job heartbeat monitoring configuration
    #[serde(default)]
    pub heartbeat: HeartbeatConfig,
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

#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatConfig {
    /// Timeout duration in seconds before considering a job stale
    #[serde(default = "default_heartbeat_timeout")]
    pub timeout_seconds: u64,

    /// Polling interval in seconds when there are active jobs
    #[serde(default = "default_active_poll_interval")]
    pub active_poll_seconds: u64,

    /// Polling interval in seconds when there are no active jobs
    #[serde(default = "default_idle_poll_interval")]
    pub idle_poll_seconds: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_heartbeat_timeout(),
            active_poll_seconds: default_active_poll_interval(),
            idle_poll_seconds: default_idle_poll_interval(),
        }
    }
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
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_database_url() -> String {
    "sqlite:glint.db?mode=rwc".to_string()
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:5173".to_string()]
}

fn default_heartbeat_timeout() -> u64 {
    300 // 5 minutes
}

fn default_active_poll_interval() -> u64 {
    10 // 10 seconds when jobs are active
}

fn default_idle_poll_interval() -> u64 {
    60 // 1 minute when no jobs are active
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("GLINT_"))
            .extract()?;

        // Load R2 config from env vars directly
        config.r2 = R2Config {
            account_id: std::env::var("GLINT_R2_ACCOUNT_ID").ok(),
            bucket: std::env::var("GLINT_R2_BUCKET").ok(),
            access_key_id: std::env::var("GLINT_R2_ACCESS_KEY_ID").ok(),
            secret_access_key: std::env::var("GLINT_R2_SECRET_ACCESS_KEY").ok(),
            public_url: std::env::var("GLINT_R2_PUBLIC_URL").ok(),
        };

        // Resolve database URL path relative to backend directory (Cargo manifest location)
        if config.database_url.starts_with("sqlite:") && !config.database_url.contains("://") {
            let path = config
                .database_url
                .strip_prefix("sqlite:")
                .unwrap_or(&config.database_url);
            if !path.starts_with('/') && !path.starts_with("file:") {
                // Relative path - resolve relative to backend directory
                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let db_path = manifest_dir.join(path.split('?').next().unwrap_or(path));
                let query = path
                    .split_once('?')
                    .map(|(_, q)| format!("?{}", q))
                    .unwrap_or_default();
                config.database_url = format!("sqlite:{}{}", db_path.display(), query);
            }
        }

        Ok(config)
    }
}
