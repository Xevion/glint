mod agent;
mod auth;
mod captures;
mod device;
mod jobs;
mod scenes;
mod sessions;
mod shaders;
mod user;
mod users;
mod worlds;

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
        .nest("/auth", auth::router())
        .nest("/user", user::router())
        .nest("/shaders", shaders::router())
        .nest("/scenes", scenes::router())
        .nest("/captures", captures::router())
        .nest("/worlds", worlds::router())
        .nest("/jobs", jobs::router())
        .nest("/users", users::router())
        .nest("/sessions", sessions::router())
        .nest("/agent", agent::router())
        .nest("/device", device::router())
}

async fn health() -> &'static str {
    "ok"
}
