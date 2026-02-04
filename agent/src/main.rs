//! Glint Agent - Captures shader screenshots via Minecraft orchestration
//!
//! This agent:
//! 1. Polls the backend for available jobs
//! 2. Downloads required resources (worlds, shaders)
//! 3. Writes scene definitions for the mod
//! 4. Launches Minecraft with autonomous mode
//! 5. Uploads resulting screenshots to R2
//! 6. Reports completion to the backend

mod client;
mod config;
mod download;
mod minecraft;
mod worker;

use anyhow::Result;
use clap::Parser;
use glint_shared::cli::VerboseArgs;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "glint-agent")]
#[command(about = "Glint capture agent - runs Minecraft to capture shader screenshots")]
struct Args {
    #[command(flatten)]
    verbose: VerboseArgs,

    /// Backend API URL
    #[arg(long, env = "GLINT_API_URL", default_value = "http://localhost:8080")]
    api_url: String,

    /// Agent API key for authentication
    #[arg(long, env = "GLINT_API_KEY", default_value = "dev-agent-key")]
    api_key: String,

    /// Minecraft directory (contains saves/, shaderpacks/, etc.)
    #[arg(long, env = "GLINT_MC_DIR", default_value = ".minecraft")]
    minecraft_dir: String,

    /// Java executable path
    #[arg(long, env = "GLINT_JAVA_PATH", default_value = "java")]
    java_path: String,

    /// Path to Minecraft launcher JAR or script
    #[arg(long, env = "GLINT_MC_LAUNCHER")]
    mc_launcher: Option<String>,

    /// Poll interval in seconds when no jobs available
    #[arg(long, env = "GLINT_POLL_INTERVAL", default_value = "30")]
    poll_interval: u64,

    /// Heartbeat interval in seconds while processing
    #[arg(long, env = "GLINT_HEARTBEAT_INTERVAL", default_value = "30")]
    heartbeat_interval: u64,

    /// Run once and exit (don't loop)
    #[arg(long)]
    once: bool,

    /// Agent identifier (for logging/tracking)
    #[arg(long, env = "GLINT_AGENT_ID", default_value = "agent-001")]
    agent_id: String,

    /// Development mode: shader slug to capture (bypasses job queue)
    #[arg(long, env = "GLINT_DEV_SHADER")]
    dev_shader: Option<String>,

    /// Development mode: scene slugs to capture (comma-separated)
    #[arg(long, env = "GLINT_DEV_SCENES")]
    dev_scenes: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing with compact formatter
    glint_shared::logging::init_with_verbosity(args.verbose.verbose);

    info!(
        agent_id = %args.agent_id,
        api_url = %args.api_url,
        minecraft_dir = %args.minecraft_dir,
        "Starting Glint Agent"
    );

    let config = config::Config {
        api_url: args.api_url,
        api_key: args.api_key,
        agent_id: args.agent_id,
        minecraft_dir: args.minecraft_dir.into(),
        java_path: args.java_path.into(),
        mc_launcher: args.mc_launcher.map(Into::into),
        poll_interval: std::time::Duration::from_secs(args.poll_interval),
        heartbeat_interval: std::time::Duration::from_secs(args.heartbeat_interval),
    };

    // Check for dev mode (direct shader/scene invocation)
    if let (Some(shader_slug), Some(scenes_csv)) = (args.dev_shader, args.dev_scenes) {
        let scene_slugs: Vec<String> = scenes_csv
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        info!(
            shader = %shader_slug,
            scenes = ?scene_slugs,
            "Running in development mode (bypassing job queue)"
        );
        worker::run_dev_direct(&config, &shader_slug, &scene_slugs).await
    } else if args.once {
        worker::run_once(&config).await
    } else {
        worker::run_loop(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["glint-agent"]);
        assert_eq!(args.verbose.verbose, 0);
        assert_eq!(args.api_url, "http://localhost:8080");
        assert_eq!(args.api_key, "dev-agent-key");
        assert_eq!(args.minecraft_dir, ".minecraft");
        assert_eq!(args.java_path, "java");
        assert_eq!(args.poll_interval, 30);
        assert_eq!(args.heartbeat_interval, 30);
        assert_eq!(args.agent_id, "agent-001");
        assert!(!args.once);
        assert!(args.dev_shader.is_none());
        assert!(args.dev_scenes.is_none());
    }

    #[test]
    fn test_args_custom_values() {
        let args = Args::parse_from([
            "glint-agent",
            "--api-url",
            "http://example.com",
            "--api-key",
            "test-key",
            "--once",
        ]);
        assert_eq!(args.api_url, "http://example.com");
        assert_eq!(args.api_key, "test-key");
        assert!(args.once);
    }

    #[test]
    fn test_verbose_flags() {
        let args = Args::parse_from(["glint-agent", "-v"]);
        assert_eq!(args.verbose.verbose, 1);

        let args = Args::parse_from(["glint-agent", "-vv"]);
        assert_eq!(args.verbose.verbose, 2);

        let args = Args::parse_from(["glint-agent", "--verbose", "--verbose"]);
        assert_eq!(args.verbose.verbose, 2);
    }
}
