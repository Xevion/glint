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
    /// Completed successfully (exit code 0, no fatal errors in logs)
    Success,
    /// Exited with non-zero code
    Failed { code: i32 },
    /// Process was killed or crashed (includes exit-0 crashes detected via log scanning)
    Crashed { message: String },
}

/// Patterns in Minecraft/JVM output that indicate a fatal crash, even if the
/// process exits with code 0. Derived from the smoke test fatal-error detection.
const FATAL_PATTERNS: &[&str] = &[
    "ExceptionInInitializerError",
    "ClassNotFoundException",
    "NoClassDefFoundError",
    "duplicate ASM classes",
    "mixin.transformer.throwables",
    "MixinApplyError",
    "java.lang.VerifyError",
    "FATAL ERROR in native method",
    "Failed to start the minecraft server",
];

impl MinecraftProcess {
    /// Launch Minecraft in autonomous mode using portablemc
    pub async fn launch(minecraft_dir: &Path, java_path: &Path) -> Result<Self> {
        info!(minecraft_dir = %minecraft_dir.display(), "Launching Minecraft");

        // Use portablemc to launch Minecraft with Fabric
        // --main-dir: libraries, assets, versions
        // --work-dir: saves, screenshots, mods, shaderpacks (game directory)
        let mut cmd = Command::new("python3");
        cmd.args([
            "-m",
            "portablemc",
            "--main-dir",
            minecraft_dir.to_str().unwrap(),
            "--work-dir",
            minecraft_dir.to_str().unwrap(),
            "start",
            "fabric:1.21.4",
            "--jvm",
            java_path.to_str().unwrap(),
            // Exclude older ASM versions that conflict with Fabric's bundled ASM.
            // Fabric Loader 0.16+ bundles ASM 9.7+; vanilla 1.21.4 ships ASM 9.6.
            // Having both on the classpath causes "duplicate ASM classes" errors.
            // Format: <artifact>[:[<version>][:<classifier>]]
            "--exclude-lib",
            "asm:9.6",
            "--exclude-lib",
            "asm-tree:9.6",
            "--exclude-lib",
            "asm-util:9.6",
            "--exclude-lib",
            "asm-commons:9.6",
            "--exclude-lib",
            "asm-analysis:9.6",
        ]);

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
                    info!(target: "minecraft", "{}", line);
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
                    warn!(target: "minecraft_err", "{}", line);
                    let _ = tx.send(line).await;
                }
            });
        }

        Ok(Self { child, log_rx })
    }

    /// Wait for Minecraft to exit.
    ///
    /// Drains log lines while waiting and scans for fatal error patterns.
    /// Even if the process exits with code 0, a fatal pattern in the output
    /// causes `MinecraftResult::Crashed` to be returned.
    pub async fn wait(mut self) -> Result<MinecraftResult> {
        // Drain the log channel while waiting for the process to exit.
        // The spawned stdout/stderr tasks write lines into this channel;
        // we collect them so we can scan for fatal errors afterwards.
        let mut fatal_lines: Vec<String> = Vec::new();

        let status = loop {
            tokio::select! {
                // Prefer draining logs so the channel doesn't fill up and
                // block the writer tasks (which would stall the child).
                biased;

                line = self.log_rx.recv() => {
                    // Channel closed — writers finished, child is likely
                    // about to (or already has) exited.
                    if let Some(line) = line
                        && FATAL_PATTERNS.iter().any(|p| line.contains(p))
                    {
                        fatal_lines.push(line);
                    }
                }

                result = self.child.wait() => {
                    break result.context("Failed to wait for Minecraft process")?;
                }
            }
        };

        // Drain any remaining lines after the process exits.
        while let Ok(line) = self.log_rx.try_recv() {
            if FATAL_PATTERNS.iter().any(|p| line.contains(p)) {
                fatal_lines.push(line);
            }
        }

        // Fatal log patterns override exit code 0.
        if !fatal_lines.is_empty() {
            let summary = fatal_lines.first().unwrap().clone();
            error!(
                fatal_line_count = fatal_lines.len(),
                first_line = %summary,
                "Minecraft crashed (detected via log output)"
            );
            return Ok(MinecraftResult::Crashed { message: summary });
        }

        if status.success() {
            debug!("Minecraft exited successfully");
            Ok(MinecraftResult::Success)
        } else if let Some(code) = status.code() {
            error!(exit_code = code, "Minecraft exited with error");
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

/// Remove duplicate library versions from the Maven-style `libraries/` tree.
///
/// portablemc sometimes downloads a newer version of a library without
/// removing the old one (e.g. ASM 9.9 alongside ASM 9.6). Fabric Loader's
/// classpath verification then fails with "duplicate ASM classes".
///
/// This walks the tree looking for artifact directories that contain multiple
/// version subdirectories (each with at least one `.jar`). For every such
/// artifact, all versions except the most recently modified are deleted.
pub async fn deduplicate_libraries(libraries_dir: &Path) -> Result<()> {
    if !libraries_dir.exists() {
        return Ok(());
    }

    let mut removed = 0u32;
    deduplicate_walk(libraries_dir, &mut removed).await?;

    if removed > 0 {
        debug!(count = removed, "Removed stale library versions");
    }

    Ok(())
}

/// Recursively walk the Maven tree. An "artifact directory" is one whose
/// immediate children are version directories (dirs containing ≥1 `.jar`).
/// If we find more than one such child, we keep only the newest (by mtime)
/// and delete the rest.
async fn deduplicate_walk(dir: &Path, removed: &mut u32) -> Result<()> {
    use tokio::fs;

    let mut entries = fs::read_dir(dir).await?;

    // Collect child directories
    let mut children: Vec<(String, std::path::PathBuf, std::time::SystemTime)> = Vec::new();
    let mut has_non_version_subdirs = false;

    while let Some(entry) = entries.next_entry().await? {
        let meta = entry.metadata().await?;
        if !meta.is_dir() {
            continue;
        }

        let path = entry.path();

        // Check if this child looks like a version directory (contains ≥1 .jar)
        if dir_contains_jar(&path).await {
            let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
            children.push((
                entry.file_name().to_string_lossy().to_string(),
                path,
                modified,
            ));
        } else {
            has_non_version_subdirs = true;
        }
    }

    // If there are multiple version directories, deduplicate
    if children.len() > 1 {
        // Sort by mtime descending — keep the newest
        children.sort_by(|a, b| b.2.cmp(&a.2));

        let (keep_version, _, _) = &children[0];
        for (version, path, _) in &children[1..] {
            warn!(
                artifact_dir = %dir.display(),
                removing = %version,
                keeping = %keep_version,
                "Removing duplicate library version"
            );
            fs::remove_dir_all(path).await?;
            *removed += 1;
        }
    }

    // Recurse into non-version subdirectories (deeper group nesting)
    if has_non_version_subdirs {
        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.metadata().await?.is_dir() && !dir_contains_jar(&entry.path()).await {
                Box::pin(deduplicate_walk(&entry.path(), removed)).await?;
            }
        }
    }

    Ok(())
}

/// Returns true if the directory directly contains at least one `.jar` file.
async fn dir_contains_jar(dir: &Path) -> bool {
    use tokio::fs;

    let Ok(mut entries) = fs::read_dir(dir).await else {
        return false;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(ext) = entry.path().extension()
            && ext == "jar"
        {
            return true;
        }
    }

    false
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

        debug!(world_slug = %world.slug, scene_count = world_scenes.len(), "Wrote scene collection");
    }

    Ok(())
}
