mod adopt;
mod auth;
mod capture_health;
mod captures;
mod device;
mod runs;
mod scenes;
mod sessions;
mod shaders;
mod storage;
mod user;
mod users;
mod work;
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
        .nest("/shaders", shaders::router().merge(adopt::router()))
        .nest("/scenes", scenes::router())
        .nest("/captures", captures::router())
        .nest("/worlds", worlds::router())
        .nest("/users", users::router())
        .nest("/sessions", sessions::router())
        .nest("/device", device::router())
        .nest("/runs", runs::router())
        .nest("/work", work::router())
        .nest("/admin/capture-health", capture_health::router())
        .nest("/admin/storage", storage::router())
        .merge(runs::failure_router())
        .merge(runs::upload_router())
}

async fn health() -> &'static str {
    "ok"
}
