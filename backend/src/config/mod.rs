use std::path::PathBuf;

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_database_url")]
    pub database_url: String,

    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
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

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("GLINT_"))
            .extract()?;

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
