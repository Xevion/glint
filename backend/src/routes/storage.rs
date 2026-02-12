use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use custom_debug_derive::Debug as CustomDebug;
use serde::Deserialize;
use tracing::instrument;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::models::{StorageBucket, StorageStats};
use crate::repo::CaptureRepo;
use crate::state::AppState;

#[derive(CustomDebug, Deserialize)]
struct GrowthParams {
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    days: Option<i32>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    interval_hours: Option<i32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stats", get(storage_stats))
        .route("/growth", get(storage_growth))
}

/// GET /api/admin/storage/stats — Aggregate storage statistics
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn storage_stats(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<StorageStats>> {
    let stats = CaptureRepo::storage_stats(state.db()).await?;
    Ok(Json(stats))
}

/// GET /api/admin/storage/growth — Cumulative storage growth with gap-filled time series
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn storage_growth(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<GrowthParams>,
) -> AppResult<Json<Vec<StorageBucket>>> {
    let days = params.days.unwrap_or(90).clamp(1, 365);
    let interval_hours = params.interval_hours.unwrap_or(1).clamp(1, 24);
    let buckets = CaptureRepo::storage_growth(state.db(), days, interval_hours).await?;
    Ok(Json(buckets))
}
