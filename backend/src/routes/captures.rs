use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use custom_debug_derive::Debug as CustomDebug;
use serde::Deserialize;
use tracing::{instrument, warn};

use crate::{
    auth::AdminUser,
    error::{AppError, AppResult, OptionNotFoundExt},
    id::{CaptureRunId, SceneId},
    models::{CaptureDetail, CaptureStatus, CaptureWithContext, PageQuery, Paginated},
    repo::{
        CaptureRepo, SceneRepo,
        capture::{CaptureDistinct, CaptureFilters},
    },
    state::AppState,
};

#[derive(CustomDebug, Deserialize)]
pub struct CaptureListParams {
    #[serde(flatten)]
    pub page: PageQuery,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub shader: Option<String>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub scene: Option<String>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub status: Option<CaptureStatus>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub run_id: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/all", get(list_captures_all))
        .route("/{id}", get(get_capture_public).delete(delete_capture))
        .route("/{id}/details", get(get_capture_details))
}

/// GET /api/captures/all - List all captures with context, paginated (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn list_captures_all(
    _admin: AdminUser,
    State(state): State<AppState>,
    Query(params): Query<CaptureListParams>,
) -> AppResult<Json<Paginated<CaptureWithContext>>> {
    let p = params.page.normalize();

    let filters = CaptureFilters {
        shader_slug: params.shader,
        scene_id: params.scene.map(SceneId::from),
        status: params.status,
        run_id: params.run_id.map(CaptureRunId::from),
        ..Default::default()
    };

    let (items, total) =
        CaptureRepo::list_with_context(state.db(), &filters, Some(&p), CaptureDistinct::None)
            .await?;

    Ok(Json(Paginated::new(items, total.unwrap_or(0), &p)))
}

/// Check `If-None-Match` against an ETag value.
fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|t| t.trim().trim_matches('"') == etag.trim_matches('"'))
        })
}

/// GET /api/captures/{id} - Get capture by ID (public, only from active scenes)
#[instrument(skip(state, headers))]
async fn get_capture_public(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let capture = CaptureRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("Capture", &id)?;

    // Verify the capture's scene is active (don't leak disabled-scene captures)
    let scene = SceneRepo::find_by_id(state.db(), capture.scene_id.as_ref()).await?;
    match scene {
        Some(s) if s.active => {}
        _ => return Err(AppError::NotFound(format!("Capture '{}' not found", id))),
    }

    let etag = format!("\"c:{}:{}\"", capture.id, capture.updated_at.timestamp());

    if etag_matches(&headers, &etag) {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        if let Ok(val) = HeaderValue::from_str(&etag) {
            response.headers_mut().insert(header::ETAG, val);
        }
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static(
                "public, max-age=120, s-maxage=120, stale-while-revalidate=600",
            ),
        );
        return Ok(response);
    }

    let mut response = Json(capture).into_response();
    if let Ok(val) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, val);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=120, s-maxage=120, stale-while-revalidate=600"),
    );
    Ok(response)
}

/// GET /api/captures/{id}/details - Get full capture detail with related captures (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn get_capture_details(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<CaptureDetail>> {
    let detail = CaptureRepo::get_detail(state.db(), &id).await?;
    Ok(Json(detail))
}

/// DELETE /api/captures/{id} - Delete a capture and its R2 image (admin)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn delete_capture(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let capture = CaptureRepo::find_by_id(state.db(), &id)
        .await?
        .or_not_found("Capture", &id)?;

    let deleted = CaptureRepo::delete(state.db(), &id).await?;
    if !deleted {
        return Err(AppError::NotFound("Capture not found".into()));
    }

    // Best-effort R2 cleanup
    if let Some(s3) = state.s3() {
        let bucket = state.config().r2.bucket.as_deref().unwrap_or("glint");
        if let Err(e) = s3
            .delete_object()
            .bucket(bucket)
            .key(&capture.image_path)
            .send()
            .await
        {
            warn!(image_path = capture.image_path, error = %e, "Failed to delete capture image from R2");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With DisplayFromStr on PageQuery, serde_urlencoded handles flatten correctly.
    #[test]
    fn serde_urlencoded_flatten_with_display_from_str() {
        let qs = "page=1&page_size=50";
        let params: CaptureListParams = serde_urlencoded::from_str(qs)
            .expect("DisplayFromStr should fix flatten with serde_urlencoded");
        assert_eq!(params.page.page, Some(1));
        assert_eq!(params.page.page_size, Some(50));
        assert!(params.shader.is_none());
        assert!(params.status.is_none());
    }

    #[test]
    fn serde_urlencoded_flatten_with_filters() {
        let qs = "page=2&page_size=25&shader=bsl&status=completed";
        let params: CaptureListParams =
            serde_urlencoded::from_str(qs).expect("DisplayFromStr should fix flatten with filters");
        assert_eq!(params.page.page, Some(2));
        assert_eq!(params.page.page_size, Some(25));
        assert_eq!(params.shader.as_deref(), Some("bsl"));
        assert_eq!(params.status, Some(CaptureStatus::Completed));
    }

    /// PageQuery also works when used directly (non-flattened).
    #[test]
    fn page_query_direct_deserialization() {
        let qs = "page=3&page_size=100";
        let params: PageQuery = serde_urlencoded::from_str(qs)
            .expect("PageQuery should deserialize directly with DisplayFromStr");
        assert_eq!(params.page, Some(3));
        assert_eq!(params.page_size, Some(100));
    }

    /// Empty query string yields None for both fields.
    #[test]
    fn page_query_empty() {
        let params: PageQuery =
            serde_urlencoded::from_str("").expect("empty query string should give defaults");
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);
    }
}
