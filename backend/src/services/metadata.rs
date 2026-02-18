use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use image::imageops::FilterType;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::R2Config;
use crate::repo::CaptureRepo;
use crate::services::lifecycle::ServiceContext;

/// Run the capture metadata background worker.
///
/// On startup, backfills any completed captures missing thumbhash or file metadata.
/// After backfill, loops on the channel processing new capture IDs as they arrive.
///
/// On shutdown, drains any remaining items from the channel so queued work
/// is not silently lost.
pub async fn run(
    ctx: ServiceContext,
    mut rx: mpsc::UnboundedReceiver<String>,
    pool: PgPool,
    http: reqwest::Client,
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
                    process_capture(&pool, &http, &r2_config, &id).await;
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

    // Process new captures as they arrive, exiting on shutdown signal
    loop {
        tokio::select! {
            item = rx.recv() => {
                match item {
                    Some(capture_id) => process_capture(&pool, &http, &r2_config, &capture_id).await,
                    None => break, // channel closed
                }
            }
            () = ctx.cancelled() => break,
        }
    }

    // Drain any items buffered in the channel so queued work isn't lost
    let mut drained = 0u32;
    while let Ok(capture_id) = rx.try_recv() {
        process_capture(&pool, &http, &r2_config, &capture_id).await;
        drained += 1;
    }
    if drained > 0 {
        info!(count = drained, "Metadata worker drained remaining items");
    }
}

/// Max retry attempts for transient fetch failures (CDN propagation delays, etc.)
const MAX_RETRIES: u32 = 3;

/// Initial retry delay (doubles each attempt: 10s, 20s, 40s)
const INITIAL_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(10);

async fn process_capture(
    pool: &PgPool,
    http: &reqwest::Client,
    r2_config: &R2Config,
    capture_id: &str,
) {
    for attempt in 0..MAX_RETRIES {
        match try_process_capture(pool, http, r2_config, capture_id).await {
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

async fn try_process_capture(
    pool: &PgPool,
    http: &reqwest::Client,
    r2_config: &R2Config,
    capture_id: &str,
) -> Result<(), ProcessError> {
    let capture = match CaptureRepo::find_by_id(pool, capture_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(ProcessError::Permanent("capture not found".into())),
        Err(e) => return Err(ProcessError::Transient(e.into())),
    };

    let image_url = r2_config.public_url_for_key(&capture.image_path);

    let needs_thumbhash = capture.thumbhash.is_none();
    let needs_file_metadata = capture.file_size_bytes.is_none();

    if !needs_thumbhash && !needs_file_metadata {
        return Err(ProcessError::Permanent("already fully processed".into()));
    }

    if needs_thumbhash {
        // Full GET: generate thumbhash + extract file metadata
        let (hash_b64, file_size, content_type) = fetch_and_process(http, &image_url)
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
        let (file_size, content_type) = fetch_file_metadata(http, &image_url)
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

/// Full GET: download image, generate thumbhash, extract file size and content type.
async fn fetch_and_process(
    http: &reqwest::Client,
    image_url: &str,
) -> anyhow::Result<(String, i64, String)> {
    let response = http.get(image_url).send().await?;
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
async fn fetch_file_metadata(
    http: &reqwest::Client,
    image_url: &str,
) -> anyhow::Result<(Option<i64>, String)> {
    let response = http.head(image_url).send().await?;

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
