mod adopt;
mod auth;
mod backgrounds;
mod capture_health;
mod captures;
mod device;
mod featured;
mod runs;
mod scenes;
mod shaders;
mod storage;
mod user;
mod users;
mod work;
mod worlds;

use axum::{Router, routing::get};

use crate::{
    analytics::Analytics,
    middleware::rate_limit::{RateLimitConfig, RateLimitLayer},
    state::AppState,
};

pub fn router(
    state: AppState,
    rate_limit: &RateLimitConfig,
    analytics: Option<&Analytics>,
) -> Router {
    Router::new()
        // Health endpoint lives outside api_router so it's not subject to any
        // rate limiting — load balancers and monitoring probes must never get 429'd.
        .route("/api/health", get(health))
        .nest("/api", api_router(rate_limit, analytics))
        .with_state(state)
}

fn api_router(rl: &RateLimitConfig, analytics: Option<&Analytics>) -> Router<AppState> {
    let make_layer = |tier: &crate::middleware::rate_limit::TierConfig, name: &'static str| {
        RateLimitLayer::new(
            tier,
            name,
            rl.trusted_proxy_hops,
            rl.enabled,
            analytics.cloned(),
        )
    };

    Router::new()
        .nest("/auth", auth::router().layer(make_layer(&rl.auth, "auth")))
        .nest(
            "/device",
            device::router().layer(make_layer(&rl.device, "device")),
        )
        .nest(
            "/worlds",
            worlds::router().layer(make_layer(&rl.upload, "upload")),
        )
        .nest(
            "/runs",
            runs::router().layer(make_layer(&rl.upload, "upload")),
        )
        .merge(runs::failure_router().layer(make_layer(&rl.upload, "upload")))
        .merge(runs::upload_router().layer(make_layer(&rl.upload, "upload")))
        .nest("/backgrounds", backgrounds::router())
        .nest("/user", user::router())
        .nest("/shaders", shaders::router().merge(adopt::router()))
        .nest("/scenes", scenes::router())
        .nest("/captures", captures::router())
        .nest("/users", users::router())
        .nest("/work", work::router())
        .nest("/featured", featured::router())
        .nest("/admin/capture-health", capture_health::router())
        .nest("/admin/storage", storage::router())
        // Intentional double rate limiting: routes like /auth, /device, and /worlds
        // have tier-specific limiters that enforce tight per-category budgets. The
        // global limiter below acts as an overall request ceiling across all routes.
        // Requests to tier-specific routes consume tokens from both their tier bucket
        // and the global bucket — this is by design.
        .layer(make_layer(&rl.global, "global"))
}

async fn health() -> &'static str {
    "ok"
}
