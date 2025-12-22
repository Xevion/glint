//! Agent configuration

use std::path::PathBuf;
use std::time::Duration;

/// Agent configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Backend API URL
    pub api_url: String,

    /// API key for authentication
    pub api_key: String,

    /// Minecraft directory (contains saves/, shaderpacks/, glint/, etc.)
    pub minecraft_dir: PathBuf,

    /// Java executable path
    pub java_path: PathBuf,

    /// Minecraft launcher path (optional, uses java directly if not set)
    pub mc_launcher: Option<PathBuf>,

    /// Poll interval when no jobs available
    pub poll_interval: Duration,

    /// Heartbeat interval while processing
    pub heartbeat_interval: Duration,

    /// Agent identifier
    pub agent_id: String,
}

impl Config {
    /// Path to the saves directory
    pub fn saves_dir(&self) -> PathBuf {
        self.minecraft_dir.join("saves")
    }

    /// Path to the shaderpacks directory
    pub fn shaderpacks_dir(&self) -> PathBuf {
        self.minecraft_dir.join("shaderpacks")
    }

    /// Path to the glint directory (scenes, captures)
    pub fn glint_dir(&self) -> PathBuf {
        self.minecraft_dir.join("glint")
    }

    /// Path to scene definitions
    pub fn scenes_dir(&self) -> PathBuf {
        self.glint_dir().join("scenes")
    }

    /// Path to capture output
    pub fn captures_dir(&self) -> PathBuf {
        self.glint_dir().join("captures")
    }
}
