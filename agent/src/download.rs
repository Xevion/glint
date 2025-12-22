//! Resource downloading (worlds, shaders)

use anyhow::{Context, Result};
use glint_shared::{ShaderInfo, WorldInfo};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};

/// Download manager for worlds and shaders
pub struct Downloader {
    client: reqwest::Client,
    /// SHA256 hashes of worlds we've already downloaded this session
    world_cache: HashSet<String>,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            world_cache: HashSet::new(),
        }
    }

    /// Download a world if not already cached
    pub async fn download_world(&mut self, world: &WorldInfo, saves_dir: &Path) -> Result<()> {
        // Check if already downloaded this session
        if self.world_cache.contains(&world.file_hash) {
            debug!(world_slug = %world.slug, "World already cached");
            return Ok(());
        }

        let world_path = saves_dir.join(&world.slug);

        // Check if world exists and hash matches
        if world_path.exists() {
            debug!(world_slug = %world.slug, "World directory exists, assuming valid");
            self.world_cache.insert(world.file_hash.clone());
            return Ok(());
        }

        info!(
            world_slug = %world.slug,
            size_bytes = world.size_bytes,
            "Downloading world"
        );

        // Download to temp file
        let temp_path = saves_dir.join(format!(".{}.zip.tmp", world.slug));
        self.download_file(&world.file_url, &temp_path).await?;

        // Verify hash
        let hash = hash_file(&temp_path).await?;
        if hash != world.file_hash {
            fs::remove_file(&temp_path).await.ok();
            anyhow::bail!(
                "World hash mismatch: expected {}, got {}",
                world.file_hash,
                hash
            );
        }

        // Extract zip
        extract_zip(&temp_path, &world_path).await?;

        // Clean up temp file
        fs::remove_file(&temp_path).await.ok();

        self.world_cache.insert(world.file_hash.clone());
        info!(world_slug = %world.slug, "World downloaded and extracted");

        Ok(())
    }

    /// Download a shader pack
    pub async fn download_shader(&self, shader: &ShaderInfo, shaderpacks_dir: &Path) -> Result<()> {
        let Some(ref download_url) = shader.download_url else {
            // Vanilla shader - nothing to download
            debug!(shader_slug = %shader.slug, "Vanilla shader, skipping download");
            return Ok(());
        };

        // Determine filename from URL or use slug fallback
        let default_filename = format!("{}.zip", shader.slug);
        let filename = download_url
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                warn!(
                    shader_slug = %shader.slug,
                    download_url = %download_url,
                    "Failed to extract filename from URL, using slug-based name"
                );
                &default_filename
            });

        let shader_path = shaderpacks_dir.join(filename);

        // Check if already exists with correct hash
        if shader_path.exists() {
            if let Some(ref expected_hash) = shader.file_hash {
                let actual_hash = hash_file(&shader_path).await?;
                if &actual_hash == expected_hash {
                    debug!(shader_slug = %shader.slug, "Shader already downloaded");
                    return Ok(());
                }
                warn!(shader_slug = %shader.slug, "Shader hash mismatch, re-downloading");
            } else {
                debug!(shader_slug = %shader.slug, "Shader exists, no hash to verify");
                return Ok(());
            }
        }

        info!(shader_slug = %shader.slug, "Downloading shader");

        // Download directly (shaders are single zip files, no extraction needed)
        self.download_file(download_url, &shader_path).await?;

        // Verify hash if provided
        if let Some(ref expected_hash) = shader.file_hash {
            let actual_hash = hash_file(&shader_path).await?;
            if &actual_hash != expected_hash {
                fs::remove_file(&shader_path).await.ok();
                anyhow::bail!(
                    "Shader hash mismatch: expected {}, got {}",
                    expected_hash,
                    actual_hash
                );
            }
        }

        info!(shader_slug = %shader.slug, "Shader downloaded");
        Ok(())
    }

    /// Download a file to a path
    async fn download_file(&self, url: &str, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to start download")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        let bytes = response.bytes().await.context("Failed to read response")?;

        let mut file = fs::File::create(path).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;

        Ok(())
    }
}

/// Compute SHA256 hash of a file
async fn hash_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).await?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{:x}", hash))
}

/// Extract a zip file to a directory
async fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let zip_path = zip_path.to_path_buf();
    let dest_dir = dest_dir.to_path_buf();

    // Run extraction in blocking task
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest_dir.join(file.mangled_name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok::<_, anyhow::Error>(())
    })
    .await??;

    Ok(())
}
