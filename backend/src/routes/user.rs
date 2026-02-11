use axum::{Json, Router, routing::get};
use tracing::instrument;

use crate::{auth::AuthUser, error::AppResult, models::User, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

/// GET /api/user/me - Returns the currently authenticated user
#[instrument(skip(auth), fields(user_id = auth.user.id))]
async fn me(auth: AuthUser) -> AppResult<Json<User>> {
    Ok(Json(auth.user))
}
