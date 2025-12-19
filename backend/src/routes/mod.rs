mod captures;
mod scenes;
mod shaders;

use axum::{Router, routing::get};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api", api_router())
        .with_state(state)
}

fn api_router() -> Router<AppState> {
    Router::new()
        .nest("/shaders", shaders::router())
        .nest("/scenes", scenes::router())
        .nest("/captures", captures::router())
}

async fn health() -> &'static str {
    "ok"
}
