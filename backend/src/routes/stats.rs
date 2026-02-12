use axum::{Json, Router, extract::State, routing::get};
use tracing::instrument;

use crate::{error::AppResult, models::stats::Stats, repo::StatsRepo, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_stats))
}

/// GET /api/stats - Aggregate counts for the public index page.
#[instrument(skip(state))]
async fn get_stats(State(state): State<AppState>) -> AppResult<Json<Stats>> {
    let stats = StatsRepo::get(state.db()).await?;
    Ok(Json(stats))
}
