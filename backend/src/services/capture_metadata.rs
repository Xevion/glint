use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use image::imageops::FilterType;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::repo::CaptureRepo;

/// Run the capture metadata background worker.
///
/// On startup, backfills any completed captures missing thumbhash or file metadata.
/// Then loops on the channel, processing new capture IDs as they arrive.
pub async fn run(mut rx: mpsc::UnboundedReceiver<String>, pool: PgPool) {
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

    // Process new captures as they arrive
    while let Some(capture_id) = rx.recv().await {
        process_capture(&pool, &capture_id).await;
    }

    warn!("Metadata worker channel closed, shutting down");
}

async fn process_capture(pool: &PgPool, capture_id: &str) {
    let capture = match CaptureRepo::find_by_id(pool, capture_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(capture_id, "Metadata: capture not found, skipping");
            return;
        }
        Err(e) => {
            error!(capture_id, error = %e, "Metadata: failed to fetch capture");
            return;
        }
    };

    let image_url = match capture.image_url {
        Some(ref url) if !url.is_empty() => url.clone(),
        _ => {
            debug!(capture_id, "Metadata: no image_url, skipping");
            return;
        }
    };

    let needs_thumbhash = capture.thumbhash.is_none();
    let needs_file_metadata = capture.file_size_bytes.is_none();

    if !needs_thumbhash && !needs_file_metadata {
        debug!(capture_id, "Metadata: already fully processed, skipping");
        return;
    }

    if needs_thumbhash {
        // Full GET: generate thumbhash + extract file metadata
        match fetch_and_process(&image_url).await {
            Ok((hash_b64, file_size, content_type)) => {
                if let Err(e) = CaptureRepo::set_thumbhash(pool, capture_id, &hash_b64).await {
                    error!(capture_id, error = %e, "Metadata: failed to save thumbhash");
                    return;
                }
                if let Err(e) =
                    CaptureRepo::set_file_metadata(pool, capture_id, file_size, &content_type).await
                {
                    error!(capture_id, error = %e, "Metadata: failed to save file metadata");
                    return;
                }
                debug!(
                    capture_id,
                    file_size, "Metadata: generated thumbhash + file metadata"
                );
            }
            Err(e) => {
                warn!(capture_id, error = %e, "Metadata: failed to fetch and process image");
            }
        }
    } else if needs_file_metadata {
        // HEAD only: just extract file metadata (thumbhash already exists)
        match fetch_file_metadata(&image_url).await {
            Ok((file_size, content_type)) => {
                if let Err(e) =
                    CaptureRepo::set_file_metadata(pool, capture_id, file_size, &content_type).await
                {
                    error!(capture_id, error = %e, "Metadata: failed to save file metadata");
                    return;
                }
                debug!(
                    capture_id,
                    file_size, "Metadata: saved file metadata via HEAD"
                );
            }
            Err(e) => {
                warn!(capture_id, error = %e, "Metadata: failed to HEAD image for metadata");
            }
        }
    }
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
async fn fetch_file_metadata(image_url: &str) -> anyhow::Result<(i64, String)> {
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
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    Ok((file_size, content_type))
}
