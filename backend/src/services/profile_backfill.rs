use std::time::Duration;

use sqlx::PgPool;
use tracing::{debug, info, warn};

use crate::extraction::normalize::normalize_display_name;
use crate::services::lifecycle::ServiceContext;

/// Number of profiles to process per batch.
const BATCH_SIZE: i64 = 200;

/// Initial backoff when no pending rows found.
const IDLE_BACKOFF_INITIAL: Duration = Duration::from_secs(60);

/// Maximum backoff when the queue stays empty.
const IDLE_BACKOFF_MAX: Duration = Duration::from_secs(10 * 60);

/// Background worker that backfills empty `display_name` values on shader profiles.
///
/// Queries for profiles where `display_name = ''` (the sentinel value), computes
/// the normalized display name from `name` and `label`, and writes it back.
/// Backs off exponentially when no work is found.
pub async fn run(ctx: ServiceContext, pool: PgPool) {
    let mut backoff = IDLE_BACKOFF_INITIAL;

    loop {
        let processed = process_batch(&pool).await;

        if processed > 0 {
            info!(count = processed, "Backfilled profile display names");
            backoff = IDLE_BACKOFF_INITIAL;
        } else {
            debug!(
                backoff_secs = backoff.as_secs(),
                "No profiles need backfill, backing off"
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

/// Fetch profiles with empty display_name and compute their normalized names.
async fn process_batch(pool: &PgPool) -> usize {
    let rows = match sqlx::query!(
        r#"
        SELECT id, name, label
        FROM shader_version_profiles
        WHERE display_name = ''
        LIMIT $1
        "#,
        BATCH_SIZE,
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!(error = %e, "Failed to query profiles for backfill");
            return 0;
        }
    };

    if rows.is_empty() {
        return 0;
    }

    let mut count = 0;
    for row in &rows {
        let display_name = normalize_display_name(&row.name, row.label.as_deref());

        if let Err(e) = sqlx::query!(
            "UPDATE shader_version_profiles SET display_name = $1 WHERE id = $2",
            display_name,
            row.id,
        )
        .execute(pool)
        .await
        {
            warn!(
                error = %e,
                profile_id = row.id,
                "Failed to update display_name for profile"
            );
            continue;
        }

        count += 1;
    }

    count
}
