use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use image::imageops::FilterType;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::R2Config;
use crate::repo::CaptureRepo;

/// Run the capture metadata background worker.
///
/// On startup, backfills any completed captures missing thumbhash or file metadata,
/// then transcodes any remaining PNG captures to AVIF. After backfill, loops on the
/// channel processing new capture IDs as they arrive.
pub async fn run(
    mut rx: mpsc::UnboundedReceiver<String>,
    pool: PgPool,
    s3: Option<S3Client>,
    r2_config: R2Config,
) {
    // Backfill existing captures missing thumbhash or file metadata
    match CaptureRepo::list_unprocessed_ids(&pool).await {
        Ok(ids) => {
            let count = ids.len();
            if count > 0 {
                info!(
                    count,
                    "Metadata worker started, backfilling unprocessed captures"
                );
                for id in ids {
                    process_capture(&pool, &id).await;
                }
                info!(count, "Metadata backfill complete");
            } else {
                info!("Metadata worker started, no backfill needed");
            }
        }
        Err(e) => {
            error!(error = %e, "Metadata worker failed to query unprocessed captures");
        }
    }

    // Backfill PNG → AVIF transcoding
    if let Some(ref s3_client) = s3 {
        match CaptureRepo::list_png_capture_ids(&pool).await {
            Ok(ids) if !ids.is_empty() => {
                info!(count = ids.len(), "Transcoding PNG captures to AVIF");
                for id in &ids {
                    transcode_capture(&pool, s3_client, &r2_config, id).await;
                }
                info!(count = ids.len(), "PNG→AVIF transcoding backfill complete");
            }
            Ok(_) => {
                info!("No PNG captures to transcode");
            }
            Err(e) => {
                error!(error = %e, "Failed to query PNG captures for transcoding");
            }
        }
    } else {
        debug!("S3 not configured, skipping PNG→AVIF transcoding");
    }

    // Process new captures as they arrive
    while let Some(capture_id) = rx.recv().await {
        process_capture(&pool, &capture_id).await;
    }

    warn!("Metadata worker channel closed, shutting down");
}

/// Max retry attempts for transient fetch failures (CDN propagation delays, etc.)
const MAX_RETRIES: u32 = 3;

/// Initial retry delay (doubles each attempt: 10s, 20s, 40s)
const INITIAL_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(10);

async fn process_capture(pool: &PgPool, capture_id: &str) {
    for attempt in 0..MAX_RETRIES {
        match try_process_capture(pool, capture_id).await {
            Ok(()) => return,
            Err(ProcessError::Permanent(msg)) => {
                debug!(capture_id, reason = %msg, "Metadata: skipping (permanent)");
                return;
            }
            Err(ProcessError::Transient(e)) => {
                if attempt + 1 < MAX_RETRIES {
                    let delay = INITIAL_RETRY_DELAY * 2u32.pow(attempt);
                    warn!(
                        capture_id,
                        attempt = attempt + 1,
                        max = MAX_RETRIES,
                        delay_secs = delay.as_secs(),
                        error = %e,
                        "Metadata: transient failure, retrying"
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    error!(
                        capture_id,
                        attempts = MAX_RETRIES,
                        error = %e,
                        "Metadata: exhausted retries, giving up"
                    );
                }
            }
        }
    }
}

enum ProcessError {
    /// Not retryable (capture not found, no URL, already processed)
    Permanent(String),
    /// Retryable (network error, CDN propagation, transient failure)
    Transient(anyhow::Error),
}

async fn try_process_capture(pool: &PgPool, capture_id: &str) -> Result<(), ProcessError> {
    let capture = match CaptureRepo::find_by_id(pool, capture_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(ProcessError::Permanent("capture not found".into())),
        Err(e) => return Err(ProcessError::Transient(e.into())),
    };

    let image_url = match capture.image_url {
        Some(ref url) if !url.is_empty() => url.clone(),
        _ => return Err(ProcessError::Permanent("no image_url".into())),
    };

    let needs_thumbhash = capture.thumbhash.is_none();
    let needs_file_metadata = capture.file_size_bytes.is_none();

    if !needs_thumbhash && !needs_file_metadata {
        return Err(ProcessError::Permanent("already fully processed".into()));
    }

    if needs_thumbhash {
        // Full GET: generate thumbhash + extract file metadata
        let (hash_b64, file_size, content_type) = fetch_and_process(&image_url)
            .await
            .map_err(ProcessError::Transient)?;

        CaptureRepo::set_thumbhash(pool, capture_id, &hash_b64)
            .await
            .map_err(|e| ProcessError::Transient(e.into()))?;
        CaptureRepo::set_file_metadata(pool, capture_id, Some(file_size), &content_type)
            .await
            .map_err(|e| ProcessError::Transient(e.into()))?;

        debug!(
            capture_id,
            file_size, "Metadata: generated thumbhash + file metadata"
        );
    } else if needs_file_metadata {
        // HEAD only: just extract file metadata (thumbhash already exists)
        let (file_size, content_type) = fetch_file_metadata(&image_url)
            .await
            .map_err(ProcessError::Transient)?;

        CaptureRepo::set_file_metadata(pool, capture_id, file_size, &content_type)
            .await
            .map_err(|e| ProcessError::Transient(e.into()))?;

        debug!(
            capture_id,
            ?file_size,
            "Metadata: saved file metadata via HEAD"
        );
    }

    Ok(())
}

/// Transcode a single PNG capture to AVIF and update R2 + DB.
async fn transcode_capture(pool: &PgPool, s3: &S3Client, r2_config: &R2Config, capture_id: &str) {
    let capture = match CaptureRepo::find_by_id(pool, capture_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(capture_id, "Transcode: capture not found");
            return;
        }
        Err(e) => {
            error!(capture_id, error = %e, "Transcode: failed to fetch capture");
            return;
        }
    };

    let image_url = match capture.image_url {
        Some(ref url) if !url.is_empty() => url.clone(),
        _ => {
            debug!(capture_id, "Transcode: no image_url, skipping");
            return;
        }
    };

    // Download the PNG
    let png_bytes = match reqwest::get(&image_url).await {
        Ok(resp) => match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                warn!(capture_id, error = %e, "Transcode: failed to download image body");
                return;
            }
        },
        Err(e) => {
            warn!(capture_id, error = %e, "Transcode: failed to download image");
            return;
        }
    };

    // Encode to WebP in a blocking task (CPU-intensive)
    let png_bytes_clone = png_bytes.clone();
    let webp_bytes =
        match tokio::task::spawn_blocking(move || encode_to_webp(&png_bytes_clone)).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => {
                warn!(capture_id, error = %e, "Transcode: WebP encoding failed");
                return;
            }
            Err(e) => {
                error!(capture_id, error = %e, "Transcode: blocking task panicked");
                return;
            }
        };

    let webp_size = webp_bytes.len() as i64;
    let png_size = png_bytes.len() as i64;

    // Derive the new R2 key by replacing .png with .webp in the old URL
    let old_r2_key = r2_config.key_from_url(&image_url);
    let new_r2_key = old_r2_key.replace(".png", ".webp");
    let new_image_url = r2_config.public_url_for_key(&new_r2_key);
    let bucket = r2_config.bucket.as_deref().unwrap_or("glint");

    // Upload WebP to R2
    if let Err(e) = s3
        .put_object()
        .bucket(bucket)
        .key(&new_r2_key)
        .content_type("image/webp")
        .body(ByteStream::from(webp_bytes))
        .send()
        .await
    {
        error!(capture_id, error = %e, "Transcode: failed to upload WebP to R2");
        return;
    }

    // Update DB with new image URL, content type, and file size
    let new_image_path = capture
        .image_path
        .as_deref()
        .map(|p| p.replace(".png", ".webp"));
    if let Err(e) = CaptureRepo::update_image_after_transcode(
        pool,
        capture_id,
        &new_image_url,
        new_image_path.as_deref(),
        webp_size,
        "image/webp",
    )
    .await
    {
        error!(capture_id, error = %e, "Transcode: failed to update DB");
        return;
    }

    // Delete old PNG from R2 (best-effort)
    if old_r2_key != new_r2_key
        && let Err(e) = s3
            .delete_object()
            .bucket(bucket)
            .key(&old_r2_key)
            .send()
            .await
    {
        warn!(capture_id, error = %e, "Transcode: failed to delete old PNG from R2");
    }

    info!(
        capture_id,
        png_size,
        webp_size,
        ratio = format_args!("{:.1}x", png_size as f64 / webp_size as f64),
        "Transcoded PNG→WebP"
    );
}

/// Encode raw image bytes (any supported format) to WebP.
fn encode_to_webp(input: &[u8]) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::WebP)?;
    Ok(buf.into_inner())
}

/// Full GET: download image, generate thumbhash, extract file size and content type.
async fn fetch_and_process(image_url: &str) -> anyhow::Result<(String, i64, String)> {
    let response = reqwest::get(image_url).await?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = response.bytes().await?;
    let file_size = bytes.len() as i64;

    // Decode + resize in a blocking task to avoid blocking the async runtime
    let hash_b64 = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        let img = image::load_from_memory(&bytes)?;
        let small = img.resize(100, 100, FilterType::Triangle);
        let rgba = small.to_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        let hash = thumbhash::rgba_to_thumb_hash(w, h, rgba.as_raw());
        Ok(BASE64.encode(hash))
    })
    .await??;

    Ok((hash_b64, file_size, content_type))
}

/// HEAD request: extract file size and content type without downloading the image.
/// Returns `None` for file_size when Content-Length header is absent, avoiding
/// permanently marking captures as 0 bytes.
async fn fetch_file_metadata(image_url: &str) -> anyhow::Result<(Option<i64>, String)> {
    let client = reqwest::Client::new();
    let response = client.head(image_url).send().await?;

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let file_size = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok());

    Ok((file_size, content_type))
}
