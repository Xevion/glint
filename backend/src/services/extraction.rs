use std::time::Duration;

use sqlx::PgPool;
use tracing::{debug, error, info, warn};

use crate::extraction::limits::MAX_ARCHIVE_SIZE;
use crate::models::ShaderVersion;
use crate::repo::{ExtractionRepo, ShaderVersionRepo};
use crate::services::lifecycle::ServiceContext;

/// Number of versions to fetch per query (large batch, worked through steadily).
const BATCH_SIZE: i64 = 60;

/// Timeout for downloading a shader pack zip from CDN.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);

/// Delay between consecutive extractions while processing a batch (~3 items per 5s).
const ITEM_DELAY: Duration = Duration::from_millis(1_700);

/// Initial backoff when no pending work is found.
const IDLE_BACKOFF_INITIAL: Duration = Duration::from_secs(30);

/// Maximum backoff when the queue stays empty.
const IDLE_BACKOFF_MAX: Duration = Duration::from_secs(5 * 60);

/// Background worker that continuously extracts metadata from pending shader versions.
///
/// Fetches a batch of pending versions, processes them steadily with a short
/// inter-item delay, then re-queries. When the queue is empty, backs off
/// exponentially (30s → 60s → … → 5min) and resets as soon as work appears.
pub async fn run(ctx: ServiceContext, pool: PgPool, http: reqwest::Client) {
    let mut backoff = IDLE_BACKOFF_INITIAL;

    loop {
        let processed = process_batch(&ctx, &pool, &http).await;

        if processed > 0 {
            backoff = IDLE_BACKOFF_INITIAL;
        } else {
            debug!(
                backoff_secs = backoff.as_secs(),
                "No pending extractions, backing off"
            );
            if !ctx.sleep(backoff).await {
                break;
            }
            backoff = (backoff * 2).min(IDLE_BACKOFF_MAX);
        }

        if ctx.is_shutting_down() {
            break;
        }
    }
}

/// Fetch and process a batch of pending shader versions.
///
/// Returns the number of items successfully fetched (not necessarily all completed).
/// Checks for shutdown between items so long batches don't block exit.
async fn process_batch(ctx: &ServiceContext, pool: &PgPool, http: &reqwest::Client) -> usize {
    let versions = match ShaderVersionRepo::list_pending_extraction(pool, BATCH_SIZE).await {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "Failed to list pending extraction versions");
            return 0;
        }
    };

    if versions.is_empty() {
        return 0;
    }

    info!(count = versions.len(), "Processing pending extractions");

    let count = versions.len();
    for (i, version) in versions.iter().enumerate() {
        if i > 0 && !ctx.sleep(ITEM_DELAY).await {
            return count;
        }
        process_one(pool, http, version).await;
    }

    count
}

/// Download, extract, and persist metadata for a single shader version.
async fn process_one(pool: &PgPool, http: &reqwest::Client, version: &ShaderVersion) {
    let version_id = version.id.as_ref();
    let download_url = match &version.download_url {
        Some(url) => url,
        None => {
            // Shouldn't happen (query filters download_url IS NOT NULL), but
            // mark failed defensively so it doesn't re-poll indefinitely.
            warn!(version_id, "Version has no download URL");
            if let Err(e) =
                ShaderVersionRepo::mark_extraction_failed(pool, version_id, "no download URL").await
            {
                error!(version_id, error = %e, "Failed to mark extraction as failed");
            }
            return;
        }
    };

    debug!(
        version_id,
        version = version.version,
        url = download_url,
        "Starting extraction"
    );

    // Download
    let bytes = match download_zip(http, download_url).await {
        Ok(b) => b,
        Err(e) => {
            let msg = format!("download failed: {e}");
            error!(version_id, error = %e, "Download failed");
            if let Err(mark_err) =
                ShaderVersionRepo::mark_extraction_failed(pool, version_id, &msg).await
            {
                error!(version_id, error = %mark_err, "Failed to mark extraction as failed");
            }
            return;
        }
    };

    debug!(
        version_id,
        size_bytes = bytes.len(),
        "Downloaded shader pack"
    );

    // Extract (CPU-bound, run on blocking thread pool)
    let extract_result = {
        match tokio::task::spawn_blocking(move || crate::extraction::extract_shader_pack(&bytes))
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let msg = format!("extraction task panicked: {e}");
                error!(version_id, error = %e, "Extraction task panicked");
                if let Err(mark_err) =
                    ShaderVersionRepo::mark_extraction_failed(pool, version_id, &msg).await
                {
                    error!(version_id, error = %mark_err, "Failed to mark extraction as failed");
                }
                return;
            }
        }
    };

    let data = match extract_result {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("extraction failed: {e}");
            warn!(version_id, error = %e, "Extraction failed");
            if let Err(mark_err) =
                ShaderVersionRepo::mark_extraction_failed(pool, version_id, &msg).await
            {
                error!(version_id, error = %mark_err, "Failed to mark extraction as failed");
            }
            return;
        }
    };

    let profile_count = data.properties.as_ref().map_or(0, |p| p.profiles.len());

    // Persist (profiles + metadata + status update in one transaction)
    if let Err(e) = ExtractionRepo::persist_extraction(pool, version_id, &data).await {
        let msg = format!("persist failed: {e}");
        error!(version_id, error = %e, "Failed to persist extraction data");
        if let Err(mark_err) =
            ShaderVersionRepo::mark_extraction_failed(pool, version_id, &msg).await
        {
            error!(version_id, error = %mark_err, "Failed to mark extraction as failed");
        }
        return;
    }

    info!(
        version_id,
        version = version.version,
        profiles = profile_count,
        has_properties = data.properties.is_some(),
        has_lang = data.lang.is_some(),
        file_count = data.scan.file_paths.len(),
        "Extraction completed"
    );
}

/// Download a shader pack zip archive from the given URL.
///
/// Validates response status and enforces the archive size limit before
/// reading the full body.
async fn download_zip(http: &reqwest::Client, url: &str) -> Result<Vec<u8>, DownloadError> {
    let response = http
        .get(url)
        .timeout(DOWNLOAD_TIMEOUT)
        .send()
        .await
        .map_err(DownloadError::Http)?;

    let status = response.status();
    if !status.is_success() {
        return Err(DownloadError::BadStatus(status.as_u16()));
    }

    // Check Content-Length header if present for early rejection
    if let Some(content_length) = response.content_length()
        && content_length as usize > MAX_ARCHIVE_SIZE
    {
        return Err(DownloadError::TooLarge {
            size: content_length as usize,
            max: MAX_ARCHIVE_SIZE,
        });
    }

    let bytes = response.bytes().await.map_err(DownloadError::Http)?;

    if bytes.len() > MAX_ARCHIVE_SIZE {
        return Err(DownloadError::TooLarge {
            size: bytes.len(),
            max: MAX_ARCHIVE_SIZE,
        });
    }

    Ok(bytes.to_vec())
}

#[derive(Debug, thiserror::Error)]
enum DownloadError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("bad status: {0}")]
    BadStatus(u16),

    #[error("archive too large: {size} bytes (max {max})")]
    TooLarge { size: usize, max: usize },
}
