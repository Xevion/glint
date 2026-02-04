use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;

/// Device authorization code for mod authentication (OAuth 2.0 Device Authorization Grant)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceCode {
    /// Random 32-char hex secret used by mod for polling
    pub device_code: String,
    /// User-facing code in GLINT-XXXXXX format
    pub user_code: String,
    /// User ID once authorized (null while pending)
    pub user_id: Option<i32>,
    /// Status: pending, authorized, used
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Response from POST /api/device/authorize
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct DeviceAuthResponse {
    /// Secret code for mod to poll with
    pub device_code: String,
    /// User-facing code to enter in browser (GLINT-XXXXXX)
    pub user_code: String,
    /// Base URL for user to visit
    pub verification_uri: String,
    /// Complete URL with code pre-filled
    pub verification_uri_complete: String,
    /// Seconds until this code expires
    pub expires_in: i64,
    /// Recommended polling interval in seconds
    pub interval: i64,
}

/// Request body for POST /api/device/token
#[derive(Debug, Deserialize)]
pub struct DeviceTokenRequest {
    pub device_code: String,
}

/// Successful response from POST /api/device/token
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct DeviceTokenResponse {
    pub access_token: String,
    pub token_type: String,
    /// Seconds until token expires
    pub expires_in: i64,
}

/// Error response from POST /api/device/token (RFC 8628 error codes)
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct DeviceTokenError {
    pub error: String,
    pub error_description: Option<String>,
}

impl DeviceTokenError {
    pub fn authorization_pending() -> Self {
        Self {
            error: "authorization_pending".to_string(),
            error_description: Some("The user has not yet authorized this device".to_string()),
        }
    }

    pub fn expired_token() -> Self {
        Self {
            error: "expired_token".to_string(),
            error_description: Some("The device code has expired".to_string()),
        }
    }

    pub fn invalid_grant() -> Self {
        Self {
            error: "invalid_grant".to_string(),
            error_description: Some(
                "The device code is invalid or has already been used".to_string(),
            ),
        }
    }
}

/// Request body for POST /api/device/confirm
#[derive(Debug, Deserialize)]
pub struct DeviceConfirmRequest {
    pub user_code: String,
}

/// Response from POST /api/device/confirm
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct DeviceConfirmResponse {
    pub success: bool,
}

/// Response from GET /api/device/code/:user_code (for frontend to check code status)
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct DeviceCodeStatus {
    pub user_code: String,
    pub status: String,
    #[ts(type = "string")]
    pub expires_at: DateTime<Utc>,
}
