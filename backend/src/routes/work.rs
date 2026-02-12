use axum::{Json, Router, extract::State, routing::get};
use serde::Deserialize;
use tracing::{debug, instrument};

use crate::auth::AgentUser;
use crate::error::AppResult;
use crate::models::WorkItem;
use crate::repo::WorkRepo;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WorkQuery {
    pub limit: Option<i64>,
    pub force: Option<bool>,
    pub shaders: Option<String>,
    pub scenes: Option<String>,
    /// If true, returns work items without side effects.
    /// Currently the endpoint is stateless, but this parameter documents intent
    /// and will prevent future reservation/locking logic from triggering.
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
    let limit = query.limit.unwrap_or(100).min(1000);
    let force = query.force.unwrap_or(false);
    let dry_run = query.dry_run.unwrap_or(false);

    // "!" and "+" are wildcard sentinels meaning "all" — normalize to None
    let shaders_filter: Option<String> = query.shaders.filter(|s| !matches!(s.as_str(), "!" | "+"));
    let scenes_filter: Option<String> = query.scenes.filter(|s| !matches!(s.as_str(), "!" | "+"));

    let items =
        WorkRepo::get_work_items(state.db(), limit, force, shaders_filter, scenes_filter).await?;

    debug!(count = items.len(), force, dry_run, "Computed work items");
    Ok(Json(items))
}
