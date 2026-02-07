use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use image::imageops::FilterType;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::repo::CaptureRepo;

/// Run the thumbhash background worker.
///
/// On startup, backfills any completed captures missing a thumbhash.
/// Then loops on the channel, processing new capture IDs as they arrive.
pub async fn run(mut rx: mpsc::UnboundedReceiver<String>, pool: PgPool) {
    // Backfill existing captures
    match CaptureRepo::list_unhashed_ids(&pool).await {
        Ok(ids) => {
            let count = ids.len();
            if count > 0 {
                info!(
                    count,
                    "ThumbHash worker started, backfilling unhashed captures"
                );
                for id in ids {
                    process_capture(&pool, &id).await;
                }
                info!(count, "ThumbHash backfill complete");
            } else {
                info!("ThumbHash worker started, no backfill needed");
            }
        }
        Err(e) => {
            error!(error = %e, "ThumbHash worker failed to query unhashed captures");
        }
    }

    // Process new captures as they arrive
    while let Some(capture_id) = rx.recv().await {
        process_capture(&pool, &capture_id).await;
    }

    warn!("ThumbHash worker channel closed, shutting down");
}

async fn process_capture(pool: &PgPool, capture_id: &str) {
    let capture = match CaptureRepo::find_by_id(pool, capture_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            warn!(capture_id, "ThumbHash: capture not found, skipping");
            return;
        }
        Err(e) => {
            error!(capture_id, error = %e, "ThumbHash: failed to fetch capture");
            return;
        }
    };

    let image_url = match capture.image_url {
        Some(ref url) if !url.is_empty() => url.clone(),
        _ => {
            debug!(capture_id, "ThumbHash: no image_url, skipping");
            return;
        }
    };

    // Already has a thumbhash
    if capture.thumbhash.is_some() {
        debug!(capture_id, "ThumbHash: already has thumbhash, skipping");
        return;
    }

    let hash_b64 = match generate_thumbhash(&image_url).await {
        Ok(h) => h,
        Err(e) => {
            warn!(capture_id, error = %e, "ThumbHash: failed to generate hash");
            return;
        }
    };

    if let Err(e) = CaptureRepo::set_thumbhash(pool, capture_id, &hash_b64).await {
        error!(capture_id, error = %e, "ThumbHash: failed to save hash");
        return;
    }

    debug!(capture_id, "ThumbHash: generated and saved");
}

async fn generate_thumbhash(image_url: &str) -> anyhow::Result<String> {
    let bytes = reqwest::get(image_url).await?.bytes().await?;

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

    Ok(hash_b64)
}
