use axum::{
    Json, Router,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tracing::instrument;

use crate::{auth::AuthUser, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

/// GET /api/user/me - Returns the currently authenticated user
///
/// Uses `private, no-cache` with ETag so browsers always revalidate but
/// get 304 when user data hasn't changed, saving response body transfer.
#[instrument(skip(auth, headers), fields(user_id = auth.user.id))]
async fn me(auth: AuthUser, headers: HeaderMap) -> Result<Response, crate::error::AppError> {
    let etag = format!(
        "\"u:{}:{}\"",
        auth.user.id,
        auth.user.updated_at.timestamp()
    );

    let etag_match = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|t| t.trim().trim_matches('"') == etag.trim_matches('"'))
        });

    if etag_match {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        if let Ok(val) = HeaderValue::from_str(&etag) {
            response.headers_mut().insert(header::ETAG, val);
        }
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
        return Ok(response);
    }

    let mut response = Json(auth.user).into_response();
    if let Ok(val) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, val);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-cache"),
    );
    Ok(response)
}
