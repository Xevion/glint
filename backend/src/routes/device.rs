use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use tracing::{debug, warn};

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::device::{
        DeviceAuthResponse, DeviceCodeStatus, DeviceConfirmRequest, DeviceConfirmResponse,
        DeviceTokenError, DeviceTokenRequest, DeviceTokenResponse,
    },
    repo::{DeviceCodeRepo, SessionRepo, device_code::POLLING_INTERVAL_SECONDS},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Start device auth flow (no auth required - mod calls this)
        .route("/authorize", post(authorize))
        // Poll for token (no auth required - mod calls this)
        .route("/token", post(token))
        // User confirms device authorization (auth required)
        .route("/confirm", post(confirm))
        // Get device code status for frontend (auth required)
        .route("/code/{user_code}", get(get_code_status))
        // Health check endpoint for mod (no auth required)
        .route("/status", get(status))
}

/// POST /api/device/authorize - Start device authorization flow
/// Called by mod to get device_code and user_code
async fn authorize(State(state): State<AppState>) -> AppResult<Json<DeviceAuthResponse>> {
    let code = DeviceCodeRepo::create(state.db()).await?;

    // Build verification URLs
    let base_url = state
        .config()
        .cors_origins
        .first()
        .cloned()
        .unwrap_or_else(|| "http://localhost:5173".to_string());

    let verification_uri = format!("{}/auth/device", base_url);
    let verification_uri_complete = format!("{}?code={}", verification_uri, code.user_code);

    let expires_in = (code.expires_at - Utc::now()).num_seconds().max(0);

    debug!(user_code = %code.user_code, "Device authorization started");

    Ok(Json(DeviceAuthResponse {
        device_code: code.device_code,
        user_code: code.user_code,
        verification_uri,
        verification_uri_complete,
        expires_in,
        interval: POLLING_INTERVAL_SECONDS,
    }))
}

/// POST /api/device/token - Poll for access token
/// Called by mod to check if user has authorized and get token
async fn token(
    State(state): State<AppState>,
    Json(request): Json<DeviceTokenRequest>,
) -> Result<Json<DeviceTokenResponse>, DeviceTokenErrorResponse> {
    let code = DeviceCodeRepo::find_by_device_code(state.db(), &request.device_code)
        .await
        .map_err(|_| DeviceTokenErrorResponse::server_error())?;

    let code = match code {
        Some(c) => c,
        None => {
            warn!("Invalid device code attempted");
            return Err(DeviceTokenErrorResponse::invalid_grant());
        }
    };

    // Check if expired
    if code.expires_at < Utc::now() {
        return Err(DeviceTokenErrorResponse::expired_token());
    }

    // Check status
    match code.status.as_str() {
        "pending" => {
            // User hasn't authorized yet
            Err(DeviceTokenErrorResponse::authorization_pending())
        }
        "authorized" => {
            // User has authorized - create session and return token
            let user_id = code.user_id.ok_or_else(|| {
                warn!("Authorized device code missing user_id");
                DeviceTokenErrorResponse::server_error()
            })?;

            // Create session for the device
            let session = SessionRepo::create(state.db(), user_id, "device")
                .await
                .map_err(|e| {
                    warn!(error = %e, "Failed to create device session");
                    DeviceTokenErrorResponse::server_error()
                })?;

            // Mark device code as used
            DeviceCodeRepo::mark_used(state.db(), &code.device_code)
                .await
                .map_err(|e| {
                    warn!(error = %e, "Failed to mark device code as used");
                    DeviceTokenErrorResponse::server_error()
                })?;

            let expires_in = (session.expires_at - Utc::now()).num_seconds().max(0);

            debug!(user_id, "Device authorization completed, token issued");

            Ok(Json(DeviceTokenResponse {
                access_token: session.token,
                token_type: "Bearer".to_string(),
                expires_in,
            }))
        }
        "used" => {
            // Already used
            Err(DeviceTokenErrorResponse::invalid_grant())
        }
        _ => {
            warn!(status = %code.status, "Unknown device code status");
            Err(DeviceTokenErrorResponse::server_error())
        }
    }
}

/// POST /api/device/confirm - User confirms device authorization
/// Called by frontend when user clicks "Authorize" button
async fn confirm(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<DeviceConfirmRequest>,
) -> AppResult<Json<DeviceConfirmResponse>> {
    DeviceCodeRepo::authorize(state.db(), &request.user_code, auth.user.id).await?;

    debug!(
        user_id = auth.user.id,
        user_code = %request.user_code,
        "Device authorization confirmed by user"
    );

    Ok(Json(DeviceConfirmResponse { success: true }))
}

/// GET /api/device/code/{user_code} - Get device code status
/// Called by frontend to show code status and verify it exists
async fn get_code_status(
    _auth: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(user_code): axum::extract::Path<String>,
) -> AppResult<Json<DeviceCodeStatus>> {
    let code = DeviceCodeRepo::find_by_user_code(state.db(), &user_code)
        .await?
        .ok_or_else(|| AppError::NotFound("Device code not found".to_string()))?;

    // Check if expired
    if code.expires_at < Utc::now() {
        return Err(AppError::Gone("Device code has expired".to_string()));
    }

    Ok(Json(DeviceCodeStatus {
        user_code: code.user_code,
        status: code.status,
        expires_at: code.expires_at,
    }))
}

/// GET /api/device/status - Health check for device auth
/// Called by mod to verify backend is reachable and auth is working
async fn status() -> &'static str {
    "ok"
}

/// Custom error response for device token endpoint (RFC 8628 compliant)
struct DeviceTokenErrorResponse {
    status: StatusCode,
    body: DeviceTokenError,
}

impl DeviceTokenErrorResponse {
    fn authorization_pending() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: DeviceTokenError::authorization_pending(),
        }
    }

    fn expired_token() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: DeviceTokenError::expired_token(),
        }
    }

    fn invalid_grant() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: DeviceTokenError::invalid_grant(),
        }
    }

    fn server_error() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: DeviceTokenError {
                error: "server_error".to_string(),
                error_description: Some("Internal server error".to_string()),
            },
        }
    }
}

impl axum::response::IntoResponse for DeviceTokenErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.body)).into_response()
    }
}
