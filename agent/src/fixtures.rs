//! Local fixtures for offline agent testing
//!
//! Loads shader, world, and scene data from the `.fixtures` directory,
//! enabling agent testing without a running backend.

use anyhow::{Context, Result};
use glint_shared::{SceneInfo, ShaderInfo, WorldInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Provider for local fixture data
pub struct FixtureProvider {
    manifest: FixtureManifest,
    fixtures_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct FixtureManifest {
    shaders: HashMap<String, ManifestShader>,
    worlds: HashMap<String, ManifestWorld>,
    scenes: HashMap<String, ManifestScene>,
}

#[derive(Debug, Deserialize)]
struct ManifestShader {
    id: String,
    name: String,
    version_id: String,
    version: String,
    file: String,
    file_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestWorld {
    id: String,
    name: String,
    file: String,
    file_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestScene {
    id: String,
    name: String,
    world_slug: String,
    definition_file: String,
}

/// Scene definition format (matches mod's SceneCollection)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SceneCollection {
    world: String,
    #[serde(default)]
    scenes: Vec<SceneDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneDefinition {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    dimension: String,
    position: Position,
    camera: Camera,
    time_of_day: i32,
    weather: String,
    #[serde(default)]
    weather_intensity: f32,
    #[serde(default)]
    moon_phase: i32,
    #[serde(default)]
    biome: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Camera {
    yaw: f32,
    pitch: f32,
}

impl FixtureProvider {
    /// Load fixtures from the agent/.fixtures directory
    pub fn load(agent_dir: &Path) -> Result<Self> {
        let fixtures_dir = agent_dir.join(".fixtures");
        let manifest_path = fixtures_dir.join("manifest.json");

        if !manifest_path.exists() {
            anyhow::bail!(
                "Fixtures not found at {:?}. Run `bun scripts/setup-fixtures.ts` first.",
                manifest_path
            );
        }

        let manifest_json = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest: {:?}", manifest_path))?;

        let manifest: FixtureManifest = serde_json::from_str(&manifest_json)
            .with_context(|| "Failed to parse fixture manifest")?;

        info!(
            shaders = manifest.shaders.len(),
            worlds = manifest.worlds.len(),
            scenes = manifest.scenes.len(),
            "Loaded fixtures manifest"
        );

        Ok(Self {
            manifest,
            fixtures_dir,
        })
    }

    /// Get shader info by slug
    pub fn get_shader(&self, slug: &str) -> Result<ShaderInfo> {
        let shader = self
            .manifest
            .shaders
            .get(slug)
            .ok_or_else(|| anyhow::anyhow!("Shader '{}' not found in fixtures", slug))?;

        Ok(ShaderInfo {
            id: shader.id.clone(),
            slug: slug.to_string(),
            name: shader.name.clone(),
            version_id: shader.version_id.clone(),
            version: shader.version.clone(),
            // For fixtures, download_url points to local file
            download_url: Some(format!("file://{}", self.shader_path(slug).display())),
            file_hash: shader.file_hash.clone(),
        })
    }

    /// Get scene and world info by slug
    pub fn get_scene(&self, slug: &str) -> Result<(SceneInfo, WorldInfo)> {
        let scene = self
            .manifest
            .scenes
            .get(slug)
            .ok_or_else(|| anyhow::anyhow!("Scene '{}' not found in fixtures", slug))?;

        let world = self.manifest.worlds.get(&scene.world_slug).ok_or_else(|| {
            anyhow::anyhow!(
                "World '{}' referenced by scene '{}' not found in fixtures",
                scene.world_slug,
                slug
            )
        })?;

        // Load scene definition from file to get the JSON
        let definition_path = self.fixtures_dir.join(&scene.definition_file);
        let collection_json = std::fs::read_to_string(&definition_path)
            .with_context(|| format!("Failed to read scene definition: {:?}", definition_path))?;

        let collection: SceneCollection = serde_json::from_str(&collection_json)
            .with_context(|| "Failed to parse scene collection")?;

        // Find the specific scene in the collection
        let scene_def = collection
            .scenes
            .iter()
            .find(|s| s.id == slug)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Scene '{}' not found in collection file {:?}",
                    slug,
                    definition_path
                )
            })?;

        // Build the definition JSON for the mod
        let definition_json = serde_json::to_string(&scene_def)
            .with_context(|| "Failed to serialize scene definition")?;

        let scene_info = SceneInfo {
            id: scene.id.clone(),
            slug: slug.to_string(),
            name: scene.name.clone(),
            world_id: world.id.clone(),
            definition_json,
        };

        let world_info = WorldInfo {
            id: world.id.clone(),
            slug: scene.world_slug.clone(),
            name: world.name.clone(),
            // For fixtures, file_url points to local file
            file_url: Some(format!(
                "file://{}",
                self.world_path(&scene.world_slug).display()
            )),
            file_hash: world.file_hash.clone(),
            size_bytes: None,
        };

        Ok((scene_info, world_info))
    }

    /// Get full path to shader zip
    pub fn shader_path(&self, slug: &str) -> PathBuf {
        if let Some(shader) = self.manifest.shaders.get(slug) {
            self.fixtures_dir.join(&shader.file)
        } else {
            self.fixtures_dir
                .join("shaders")
                .join(format!("{}.zip", slug))
        }
    }

    /// Get full path to world zip
    pub fn world_path(&self, slug: &str) -> PathBuf {
        if let Some(world) = self.manifest.worlds.get(slug) {
            self.fixtures_dir.join(&world.file)
        } else {
            self.fixtures_dir
                .join("worlds")
                .join(format!("{}.zip", slug))
        }
    }

    /// List available shader slugs
    pub fn available_shaders(&self) -> Vec<&str> {
        self.manifest.shaders.keys().map(|s| s.as_str()).collect()
    }

    /// List available scene slugs
    #[allow(dead_code)]
    pub fn available_scenes(&self) -> Vec<&str> {
        self.manifest.scenes.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a shader file exists
    pub fn shader_exists(&self, slug: &str) -> bool {
        self.shader_path(slug).exists()
    }

    /// Check if a world file exists
    pub fn world_exists(&self, slug: &str) -> bool {
        self.world_path(slug).exists()
    }
}

/// Resolve shader and scenes from fixtures for offline mode
pub fn resolve_from_fixtures(
    fixtures: &FixtureProvider,
    shader_slug: &str,
    scene_slugs: &[String],
) -> Result<(ShaderInfo, Vec<SceneInfo>, Vec<WorldInfo>)> {
    debug!(shader = %shader_slug, scenes = ?scene_slugs, "Resolving from fixtures");

    // Validate shader exists
    if !fixtures.shader_exists(shader_slug) {
        anyhow::bail!(
            "Shader '{}' not downloaded. Available: {:?}",
            shader_slug,
            fixtures.available_shaders()
        );
    }

    let shader = fixtures.get_shader(shader_slug)?;

    let mut scenes = Vec::new();
    let mut worlds = Vec::new();
    let mut seen_world_ids = std::collections::HashSet::new();

    for slug in scene_slugs {
        let (scene, world) = fixtures.get_scene(slug)?;

        // Check world file exists
        let _world_slug = &scene.world_id;
        if !fixtures.world_exists(&fixtures.manifest.scenes[slug].world_slug) {
            let world_slug_actual = &fixtures.manifest.scenes[slug].world_slug;
            anyhow::bail!(
                "World '{}' for scene '{}' not found. Place world zip at: {:?}",
                world_slug_actual,
                slug,
                fixtures.world_path(world_slug_actual)
            );
        }

        scenes.push(scene);

        // Deduplicate worlds
        if seen_world_ids.insert(world.id.clone()) {
            worlds.push(world);
        }
    }

    info!(
        shader = %shader.slug,
        shader_version = %shader.version,
        scene_count = scenes.len(),
        world_count = worlds.len(),
        "Resolved from fixtures"
    );

    Ok((shader, scenes, worlds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_fixtures(dir: &Path) {
        let fixtures_dir = dir.join(".fixtures");
        fs::create_dir_all(fixtures_dir.join("shaders")).unwrap();
        fs::create_dir_all(fixtures_dir.join("worlds")).unwrap();
        fs::create_dir_all(fixtures_dir.join("scenes")).unwrap();

        // Create manifest
        let manifest = r#"{
            "shaders": {
                "test-shader": {
                    "id": "fixture-test",
                    "name": "Test Shader",
                    "version_id": "fixture-test-v1",
                    "version": "v1.0.0",
                    "file": "shaders/test-shader-v1.0.0.zip",
                    "file_hash": "sha256:abc123"
                }
            },
            "worlds": {
                "test-world": {
                    "id": "fixture-world-1",
                    "name": "Test World",
                    "file": "worlds/test-world.zip"
                }
            },
            "scenes": {
                "test-scene": {
                    "id": "fixture-scene-1",
                    "name": "Test Scene",
                    "world_slug": "test-world",
                    "definition_file": "scenes/test-world.json"
                }
            }
        }"#;
        fs::write(fixtures_dir.join("manifest.json"), manifest).unwrap();

        // Create scene definition
        let scene_def = r#"{
            "world": "test-world",
            "scenes": [{
                "id": "test-scene",
                "name": "Test Scene",
                "dimension": "minecraft:overworld",
                "position": {"x": 0, "y": 100, "z": 0},
                "camera": {"yaw": 0, "pitch": 0},
                "timeOfDay": 6000,
                "weather": "CLEAR",
                "weatherIntensity": 0
            }]
        }"#;
        fs::write(fixtures_dir.join("scenes/test-world.json"), scene_def).unwrap();

        // Create dummy shader zip
        fs::write(
            fixtures_dir.join("shaders/test-shader-v1.0.0.zip"),
            b"fake zip",
        )
        .unwrap();

        // Create dummy world zip
        fs::write(fixtures_dir.join("worlds/test-world.zip"), b"fake zip").unwrap();
    }

    #[test]
    fn test_load_fixtures() {
        let temp = TempDir::new().unwrap();
        create_test_fixtures(temp.path());

        let provider = FixtureProvider::load(temp.path()).unwrap();
        assert!(provider.available_shaders().contains(&"test-shader"));
        assert!(provider.available_scenes().contains(&"test-scene"));
    }

    #[test]
    fn test_get_shader() {
        let temp = TempDir::new().unwrap();
        create_test_fixtures(temp.path());

        let provider = FixtureProvider::load(temp.path()).unwrap();
        let shader = provider.get_shader("test-shader").unwrap();

        assert_eq!(shader.slug, "test-shader");
        assert_eq!(shader.version, "v1.0.0");
    }

    #[test]
    fn test_get_scene() {
        let temp = TempDir::new().unwrap();
        create_test_fixtures(temp.path());

        let provider = FixtureProvider::load(temp.path()).unwrap();
        let (scene, world) = provider.get_scene("test-scene").unwrap();

        assert_eq!(scene.slug, "test-scene");
        assert_eq!(world.slug, "test-world");
    }
}
