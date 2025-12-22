//! Minecraft process launching and monitoring

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Minecraft process handle
pub struct MinecraftProcess {
    child: Child,
    /// Channel to receive log lines
    log_rx: mpsc::Receiver<String>,
}

/// Result of waiting for Minecraft to complete
#[derive(Debug)]
pub enum MinecraftResult {
    /// Completed successfully (exit code 0)
    Success,
    /// Exited with non-zero code
    Failed { code: i32 },
    /// Process was killed or crashed
    Crashed { message: String },
}

impl MinecraftProcess {
    /// Launch Minecraft in autonomous mode
    ///
    /// This is a simplified launcher - in production you'd use the actual
    /// Minecraft launcher or a wrapper script.
    pub async fn launch(minecraft_dir: &Path, java_path: &Path) -> Result<Self> {
        info!(minecraft_dir = %minecraft_dir.display(), "Launching Minecraft");

        // For now, we expect a launch script at minecraft_dir/launch.sh
        // In production, this would integrate with the actual launcher
        let launch_script = minecraft_dir.join("launch.sh");

        let mut cmd = if launch_script.exists() {
            let mut c = Command::new(&launch_script);
            c.current_dir(minecraft_dir);
            c
        } else {
            // Fallback: try to find and run fabric launcher
            // This is a placeholder - real implementation would use proper launcher
            warn!("No launch.sh found, attempting direct fabric launch");
            let mut c = Command::new(java_path);
            c.current_dir(minecraft_dir);
            // These args are placeholders - real launch requires proper classpath, main class, etc.
            c.args(["-version"]);
            c
        };

        // Set autonomous mode
        cmd.env("GLINT_AUTONOMOUS", "true");

        // Capture stdout/stderr
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn Minecraft process")?;

        // Set up log streaming
        let (log_tx, log_rx) = mpsc::channel(1000);

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let tx = log_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!(target: "minecraft", "{}", line);
                    let _ = tx.send(line).await;
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let tx = log_tx;
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!(target: "minecraft_err", "{}", line);
                    let _ = tx.send(line).await;
                }
            });
        }

        Ok(Self { child, log_rx })
    }

    /// Wait for Minecraft to exit
    pub async fn wait(mut self) -> Result<MinecraftResult> {
        // Drop log receiver - we've already set up streaming
        drop(self.log_rx);

        let status = self
            .child
            .wait()
            .await
            .context("Failed to wait for Minecraft process")?;

        if status.success() {
            info!("Minecraft exited successfully");
            Ok(MinecraftResult::Success)
        } else if let Some(code) = status.code() {
            error!(code, "Minecraft exited with error");
            Ok(MinecraftResult::Failed { code })
        } else {
            error!("Minecraft process was killed");
            Ok(MinecraftResult::Crashed {
                message: "Process was killed".to_string(),
            })
        }
    }

    /// Kill the Minecraft process
    #[allow(dead_code)]
    pub async fn kill(&mut self) -> Result<()> {
        self.child
            .kill()
            .await
            .context("Failed to kill Minecraft process")
    }
}

/// Write scene definitions for the mod to read
pub async fn write_scene_definitions(
    scenes_dir: &Path,
    scenes: &[glint_shared::SceneInfo],
    worlds: &[glint_shared::WorldInfo],
) -> Result<()> {
    use std::collections::HashMap;
    use tokio::fs;

    // Ensure scenes directory exists
    fs::create_dir_all(scenes_dir).await?;

    // Group scenes by world
    let mut scenes_by_world: HashMap<&str, Vec<&glint_shared::SceneInfo>> = HashMap::new();
    let world_map: HashMap<&str, &glint_shared::WorldInfo> =
        worlds.iter().map(|w| (w.id.as_str(), w)).collect();

    for scene in scenes {
        scenes_by_world
            .entry(&scene.world_id)
            .or_default()
            .push(scene);
    }

    // Write one file per world
    for (world_id, world_scenes) in scenes_by_world {
        let Some(world) = world_map.get(world_id) else {
            warn!(world_id, "World not found for scenes, skipping");
            continue;
        };

        // For now, we just write the raw scene definitions
        // The mod expects SceneCollection format, which includes world metadata
        // This is a simplified version - we'd need to construct the full format
        let scene_file = scenes_dir.join(format!("{}.json", world.slug));

        // Parse scene definitions
        let mut definitions: Vec<serde_json::Value> = Vec::new();
        for scene in &world_scenes {
            match serde_json::from_str(&scene.definition_json) {
                Ok(def) => definitions.push(def),
                Err(e) => {
                    warn!(
                        scene_id = %scene.id,
                        error = %e,
                        "Failed to parse scene definition JSON, skipping scene"
                    );
                }
            }
        }

        let collection = serde_json::json!({
            "world": world.slug,
            "version": "1.21.4",
            "scenes": definitions
        });

        let json = serde_json::to_string_pretty(&collection)?;
        fs::write(&scene_file, json).await?;

        info!(world_slug = %world.slug, scene_count = world_scenes.len(), "Wrote scene collection");
    }

    Ok(())
}
