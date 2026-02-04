use axum::{Json, Router, routing::get};

use crate::{auth::AuthUser, error::AppResult, models::User, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

/// GET /api/user/me - Returns the currently authenticated user
async fn me(auth: AuthUser) -> AppResult<Json<User>> {
    Ok(Json(auth.user))
}
