use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    models::{Capture, CaptureDetail, PaginatedCaptures},
    repo::CaptureRepo,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CaptureListParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub shader: Option<String>,
    pub scene: Option<String>,
    pub status: Option<String>,
    pub run_id: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_captures_public))
        .route("/all", get(list_captures_all))
        .route("/{id}", get(get_capture_public).delete(delete_capture))
        .route("/{id}/details", get(get_capture_details))
}

/// GET /api/captures - List completed captures (public)
async fn list_captures_public(State(state): State<AppState>) -> AppResult<Json<Vec<Capture>>> {
    let captures = CaptureRepo::list_completed(state.db()).await?;
    Ok(Json(captures))
}

/// GET /api/captures/all - List all captures with context, paginated (admin)
async fn list_captures_all(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<CaptureListParams>,
) -> AppResult<Json<PaginatedCaptures>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(50).clamp(1, 250);
    let offset = (page - 1) * page_size;

    let (items, total) = CaptureRepo::list_all_with_context_paginated(
        state.db(),
        page_size as i64,
        offset as i64,
        params.shader.as_deref(),
        params.scene.as_deref(),
        params.status.as_deref(),
        params.run_id.as_deref(),
    )
    .await?;

    Ok(Json(PaginatedCaptures {
        items,
        total,
        page,
        page_size,
    }))
}

/// GET /api/captures/{id} - Get capture by ID (public)
async fn get_capture_public(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Capture>> {
    let capture = CaptureRepo::get_by_id(state.db(), &id).await?;
    Ok(Json(capture))
}

/// GET /api/captures/{id}/details - Get full capture detail with related captures (admin)
async fn get_capture_details(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<CaptureDetail>> {
    let detail = CaptureRepo::get_detail(state.db(), &id).await?;
    Ok(Json(detail))
}

/// DELETE /api/captures/{id} - Delete a capture (admin)
async fn delete_capture(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let deleted = CaptureRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("Capture not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}
