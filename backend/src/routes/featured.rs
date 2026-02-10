use axum::{Json, Router, extract::State, routing::get};

use crate::{error::AppResult, models::FeaturedPair, repo::FeaturedRepo, state::AppState};

const DEFAULT_PAIR_COUNT: usize = 6;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_featured_pairs))
}

/// GET /api/featured — Random popular shader pairs for the homepage hero
async fn list_featured_pairs(State(state): State<AppState>) -> AppResult<Json<Vec<FeaturedPair>>> {
    let pairs = FeaturedRepo::random_pairs(state.db(), DEFAULT_PAIR_COUNT).await?;
    Ok(Json(pairs))
}
