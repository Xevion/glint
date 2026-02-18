use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::extraction::normalize::normalize_display_name;
use crate::services::lifecycle::ServiceContext;

/// One-shot background worker that normalizes `display_name` for all shader profiles.
///
/// Queries every profile in the database, recomputes the normalized display name
/// from `name` and `label`, and updates any rows whose stored value differs.
/// Reports statistics and any anomalies, then exits permanently.
pub async fn run(ctx: ServiceContext, pool: PgPool) {
    if ctx.is_shutting_down() {
        return;
    }

    info!("Starting profile display name normalization (one-shot)");

    let rows = match sqlx::query!(
        r#"
        SELECT id, name, label, display_name
        FROM shader_version_profiles
        ORDER BY id
        "#,
    )
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "Failed to query profiles for normalization backfill — aborting");
            return;
        }
    };

    let total = rows.len();
    if total == 0 {
        info!("No profiles in database — nothing to normalize");
        return;
    }

    let mut updated = 0u64;
    let mut empty_results = 0u64;
    let mut short_results = 0u64;
    let mut errors = 0u64;

    for row in &rows {
        let normalized = normalize_display_name(&row.name, row.label.as_deref());

        // Validate the normalization result
        if normalized.is_empty() {
            error!(
                profile_id = row.id,
                name = row.name,
                label = ?row.label,
                stored = row.display_name,
                "Normalization produced empty string — this should never happen"
            );
            empty_results += 1;
            continue;
        }

        if normalized.len() < 3 {
            warn!(
                profile_id = row.id,
                name = row.name,
                label = ?row.label,
                display_name = normalized,
                "Normalization produced suspiciously short display name (<3 chars)"
            );
            short_results += 1;
        }

        // Only update if the stored value differs
        if row.display_name == normalized {
            continue;
        }

        if let Err(e) = sqlx::query!(
            "UPDATE shader_version_profiles SET display_name = $1 WHERE id = $2",
            normalized,
            row.id,
        )
        .execute(&pool)
        .await
        {
            error!(
                error = %e,
                profile_id = row.id,
                name = row.name,
                old_display_name = row.display_name,
                new_display_name = normalized,
                "Failed to update display_name for profile"
            );
            errors += 1;
            continue;
        }

        updated += 1;
    }

    // Summary report
    if errors > 0 || empty_results > 0 {
        error!(
            total,
            updated,
            errors,
            empty_results,
            short_results,
            "Profile normalization completed with errors"
        );
    } else if short_results > 0 || updated > 0 {
        warn!(
            total,
            updated, short_results, "Profile normalization completed with warnings"
        );
    } else {
        info!(
            total,
            "Profile normalization completed — all display names are current"
        );
    }
}
