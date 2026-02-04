use anyhow::Context;
use chrono::{Duration, Utc};
use rand::Rng;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::device::DeviceCode;

/// Device code expiry duration
pub const DEVICE_CODE_EXPIRY_MINUTES: i64 = 15;

/// Polling interval in seconds (RFC 8628 recommends 5 seconds minimum)
pub const POLLING_INTERVAL_SECONDS: i64 = 5;

/// User-facing code charset (no ambiguous characters: 0/O, 1/I/L)
const USER_CODE_CHARSET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
const USER_CODE_LENGTH: usize = 6;

pub struct DeviceCodeRepo;

/// Generate a cryptographically secure device code (32-char hex)
fn generate_device_code() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill(&mut bytes);
    hex::encode(bytes)
}

/// Generate a user-friendly code (GLINT-XXXXXX format)
fn generate_user_code() -> String {
    let mut rng = rand::rng();
    let code: String = (0..USER_CODE_LENGTH)
        .map(|_| {
            let idx = rng.random_range(0..USER_CODE_CHARSET.len());
            USER_CODE_CHARSET[idx] as char
        })
        .collect();
    format!("GLINT-{}", code)
}

impl DeviceCodeRepo {
    /// Create a new device authorization code pair
    #[instrument(skip(db), level = "debug")]
    pub async fn create(db: &DbPool) -> AppResult<DeviceCode> {
        let device_code = generate_device_code();
        let user_code = generate_user_code();
        let expires_at = Utc::now() + Duration::minutes(DEVICE_CODE_EXPIRY_MINUTES);

        let code = sqlx::query_as!(
            DeviceCode,
            r#"
            INSERT INTO device_codes (device_code, user_code, expires_at)
            VALUES ($1, $2, $3)
            RETURNING device_code, user_code, user_id, status, expires_at, created_at
            "#,
            device_code,
            user_code,
            expires_at
        )
        .fetch_one(db)
        .await
        .context("failed to create device code")?;

        debug!(user_code = %code.user_code, "Device code created");
        Ok(code)
    }

    /// Find a device code by its secret device_code (for mod polling)
    #[instrument(skip(db, device_code), level = "debug")]
    pub async fn find_by_device_code(
        db: &DbPool,
        device_code: &str,
    ) -> AppResult<Option<DeviceCode>> {
        let code = sqlx::query_as!(
            DeviceCode,
            r#"
            SELECT device_code, user_code, user_id, status, expires_at, created_at
            FROM device_codes
            WHERE device_code = $1
            "#,
            device_code
        )
        .fetch_optional(db)
        .await
        .context("failed to find device code")?;

        Ok(code)
    }

    /// Find a device code by its user-facing code (for user confirmation)
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_user_code(db: &DbPool, user_code: &str) -> AppResult<Option<DeviceCode>> {
        // Normalize user code (uppercase, handle with or without GLINT- prefix)
        let normalized = user_code.to_uppercase();
        let normalized = if normalized.starts_with("GLINT-") {
            normalized
        } else {
            format!("GLINT-{}", normalized)
        };

        let code = sqlx::query_as!(
            DeviceCode,
            r#"
            SELECT device_code, user_code, user_id, status, expires_at, created_at
            FROM device_codes
            WHERE user_code = $1
            "#,
            normalized
        )
        .fetch_optional(db)
        .await
        .context("failed to find device code by user code")?;

        Ok(code)
    }

    /// Authorize a device code (user confirms in browser)
    #[instrument(skip(db), level = "debug")]
    pub async fn authorize(db: &DbPool, user_code: &str, user_id: i32) -> AppResult<DeviceCode> {
        // Normalize user code
        let normalized = user_code.to_uppercase();
        let normalized = if normalized.starts_with("GLINT-") {
            normalized
        } else {
            format!("GLINT-{}", normalized)
        };

        let code = sqlx::query_as!(
            DeviceCode,
            r#"
            UPDATE device_codes
            SET user_id = $1, status = 'authorized'
            WHERE user_code = $2
              AND status = 'pending'
              AND expires_at > now()
            RETURNING device_code, user_code, user_id, status, expires_at, created_at
            "#,
            user_id,
            normalized
        )
        .fetch_optional(db)
        .await
        .context("failed to authorize device code")?;

        match code {
            Some(c) => {
                debug!(user_code = %c.user_code, user_id, "Device code authorized");
                Ok(c)
            }
            None => Err(AppError::NotFound(
                "Device code not found, already used, or expired".to_string(),
            )),
        }
    }

    /// Mark a device code as used (after token is issued)
    #[instrument(skip(db, device_code), level = "debug")]
    pub async fn mark_used(db: &DbPool, device_code: &str) -> AppResult<()> {
        let result = sqlx::query!(
            r#"
            UPDATE device_codes
            SET status = 'used'
            WHERE device_code = $1
            "#,
            device_code
        )
        .execute(db)
        .await
        .context("failed to mark device code as used")?;

        if result.rows_affected() > 0 {
            debug!("Device code marked as used");
        }
        Ok(())
    }

    /// Delete expired device codes (cleanup task)
    #[instrument(skip(db), level = "debug")]
    pub async fn delete_expired(db: &DbPool) -> AppResult<u64> {
        let result = sqlx::query!("DELETE FROM device_codes WHERE expires_at < now()")
            .execute(db)
            .await
            .context("failed to delete expired device codes")?;

        let count = result.rows_affected();
        if count > 0 {
            debug!(count, "Deleted expired device codes");
        }
        Ok(count)
    }
}
