use axum::{Json, Router, extract::State, routing::get};
use custom_debug_derive::Debug as CustomDebug;
use serde::Deserialize;
use tracing::{debug, instrument};

use crate::auth::AgentUser;
use crate::error::AppResult;
use crate::models::WorkItem;
use crate::repo::WorkRepo;
use crate::state::AppState;

#[derive(CustomDebug, Deserialize)]
pub struct WorkQuery {
    /// Maximum number of distinct shader versions to include per world,
    /// ordered by popularity. Guarantees complete shader units (all scenes
    /// and profiles for selected shaders are included).
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub shader_limit: Option<i64>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub force: Option<bool>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub shaders: Option<String>,
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub scenes: Option<String>,
    /// If true, returns work items without side effects.
    /// Currently the endpoint is stateless, but this parameter documents intent
    /// and will prevent future reservation/locking logic from triggering.
    #[debug(skip_if = Option::is_none, with = "crate::fmt::opt")]
    pub dry_run: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_work))
}

#[instrument(skip(state, _user), fields(user_id = _user.user.id))]
async fn get_work(
    _user: AgentUser,
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<WorkQuery>,
) -> AppResult<Json<Vec<WorkItem>>> {
    let shader_limit = query.shader_limit.unwrap_or(10).min(100);
    let force = query.force.unwrap_or(false);
    let dry_run = query.dry_run.unwrap_or(false);

    // "!" and "+" are wildcard sentinels meaning "all" — normalize to None
    let shaders_filter: Option<String> = query.shaders.filter(|s| !matches!(s.as_str(), "!" | "+"));
    let scenes_filter: Option<String> = query.scenes.filter(|s| !matches!(s.as_str(), "!" | "+"));

    let items = WorkRepo::get_work_items(
        state.db(),
        shader_limit,
        force,
        shaders_filter,
        scenes_filter,
    )
    .await?;

    debug!(count = items.len(), force, dry_run, "Computed work items");
    Ok(Json(items))
}
