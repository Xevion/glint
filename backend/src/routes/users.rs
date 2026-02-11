use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use tracing::instrument;

use crate::{
    auth::{AdminUser, AuthUser},
    error::{AppError, AppResult, OptionNotFoundExt},
    models::{SessionInfo, UpdateUserRoleRequest, User, UserWithSessions},
    repo::{SessionRepo, UserRepo},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", get(get_user))
        .route("/{id}/role", put(update_user_role))
        .route("/{id}/sessions", get(list_sessions))
        .route("/{id}/sessions", delete(delete_all_sessions))
        .route("/{id}/sessions/{prefix}", delete(delete_session_by_prefix))
        .route("/{id}/sessions/revoke-others", post(revoke_other_sessions))
}

/// Parse the `{id}` path segment: "me" resolves to the authenticated user,
/// numeric IDs require admin privileges when targeting another user.
fn resolve_user_id(auth: &AuthUser, id: &str) -> AppResult<(i32, bool)> {
    if id == "me" {
        return Ok((auth.user.id, true));
    }

    let target_id: i32 = id
        .parse()
        .map_err(|_| AppError::BadRequest(format!("Invalid user ID: '{id}'")))?;

    if target_id == auth.user.id {
        return Ok((target_id, true));
    }

    if !auth.user.role.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    Ok((target_id, false))
}

/// Build `Vec<SessionInfo>` from raw rows, enriching with `is_current`.
fn build_session_infos(
    rows: Vec<crate::repo::SessionRow>,
    current_token: &str,
) -> Vec<SessionInfo> {
    rows.into_iter()
        .map(|row| {
            let is_current = row.token == current_token;
            SessionInfo {
                token_prefix: row.token.chars().take(8).collect(),
                source: row.source,
                is_current,
                created_at: row.created_at,
                expires_at: row.expires_at,
            }
        })
        .collect()
}

/// GET /api/users - List all users (admin only)
#[instrument(skip(state, _admin), fields(user_id = _admin.user.id))]
async fn list_users(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<User>>> {
    let users = UserRepo::list(state.db()).await?;
    Ok(Json(users))
}

/// GET /api/users/{id} - Get user profile with sessions (id=me or admin)
#[instrument(skip(state, auth), fields(user_id = auth.user.id))]
async fn get_user(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<UserWithSessions>> {
    let (user_id, _is_self) = resolve_user_id(&auth, &id)?;
    let user = UserRepo::find_by_id(state.db(), user_id)
        .await?
        .or_not_found("User", user_id)?;
    let rows = SessionRepo::list_for_user(state.db(), user_id).await?;
    let sessions = build_session_infos(rows, &auth.session.token);
    Ok(Json(UserWithSessions { user, sessions }))
}

/// PUT /api/users/{id}/role - Update user role (admin only)
#[instrument(skip(state, admin, request), fields(user_id = admin.user.id))]
async fn update_user_role(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<User>> {
    let target_id: i32 = if id == "me" {
        admin.user.id
    } else {
        id.parse()
            .map_err(|_| AppError::BadRequest(format!("Invalid user ID: '{id}'")))?
    };

    let user = UserRepo::update_role(state.db(), target_id, request.role).await?;

    // Invalidate cached sessions so the new role takes effect immediately
    state.session_cache().invalidate_user(target_id);

    Ok(Json(user))
}

/// GET /api/users/{id}/sessions - List sessions (id=me or admin)
#[instrument(skip(state, auth), fields(user_id = auth.user.id))]
async fn list_sessions(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<SessionInfo>>> {
    let (user_id, _is_self) = resolve_user_id(&auth, &id)?;
    let rows = SessionRepo::list_for_user(state.db(), user_id).await?;
    let sessions = build_session_infos(rows, &auth.session.token);
    Ok(Json(sessions))
}

/// DELETE /api/users/{id}/sessions - Revoke ALL sessions for a user (id=me or admin)
#[instrument(skip(state, auth), fields(user_id = auth.user.id))]
async fn delete_all_sessions(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let (user_id, _is_self) = resolve_user_id(&auth, &id)?;
    SessionRepo::delete_for_user(state.db(), user_id).await?;
    state.session_cache().invalidate_user(user_id);
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/users/{id}/sessions/{prefix} - Revoke a single session by token prefix
#[instrument(skip(state, auth), fields(user_id = auth.user.id))]
async fn delete_session_by_prefix(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, prefix)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    let (user_id, _is_self) = resolve_user_id(&auth, &id)?;

    let deleted_token = SessionRepo::delete_by_prefix(state.db(), user_id, &prefix)
        .await?
        .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

    state.session_cache().invalidate_token(&deleted_token);
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/users/{id}/sessions/revoke-others - Revoke all sessions except current
#[instrument(skip(state, auth), fields(user_id = auth.user.id))]
async fn revoke_other_sessions(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let (user_id, is_self) = resolve_user_id(&auth, &id)?;

    if !is_self {
        SessionRepo::delete_for_user(state.db(), user_id).await?;
    } else {
        SessionRepo::delete_others_for_user(state.db(), user_id, &auth.session.token).await?;
    }

    // Invalidate all cached sessions for user; the current session re-populates on next request
    state.session_cache().invalidate_user(user_id);
    Ok(StatusCode::NO_CONTENT)
}
