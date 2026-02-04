use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, put},
};

use crate::{
    auth::AdminUser,
    error::AppResult,
    models::{UpdateUserRoleRequest, User, UserWithSessions},
    repo::{SessionRepo, UserRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", get(get_user))
        .route("/{id}/role", put(update_user_role))
        .route("/{id}/sessions", delete(delete_user_sessions))
}

/// GET /api/users - List all users (admin)
async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<User>>> {
    let users = UserRepo::list(state.db()).await?;
    Ok(Json(users))
}

/// GET /api/users/{id} - Get user by ID with sessions (admin)
async fn get_user(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<UserWithSessions>> {
    let user = UserRepo::get_by_id(state.db(), id).await?;
    let sessions = SessionRepo::list_for_user(state.db(), id).await?;
    Ok(Json(UserWithSessions { user, sessions }))
}

/// PUT /api/users/{id}/role - Update user role (admin)
async fn update_user_role(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<User>> {
    let user = UserRepo::update_role(state.db(), id, &request.role).await?;
    Ok(Json(user))
}

/// DELETE /api/users/{id}/sessions - Delete all sessions for a user (admin)
async fn delete_user_sessions(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<StatusCode> {
    SessionRepo::delete_for_user(state.db(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
