mod adopt;
mod auth;
mod backgrounds;
mod capture_health;
mod captures;
pub mod csp_report;
mod device;
mod extraction;
mod featured;
mod runs;
mod scenes;
mod shaders;
mod sitemap;
mod stats;
mod storage;
mod user;
mod users;
mod work;
use axum::{Router, routing::get};
use tracing::instrument;

use crate::{
    analytics::Analytics,
    middleware::{
        cache_control::CacheControlLayer,
        rate_limit::{RateLimitConfig, RateLimitLayer},
    },
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
        // Sitemap is outside rate limiting — crawlers should never be throttled.
        .route("/api/sitemap.xml", get(sitemap::sitemap))
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

    // Cache directives applied per-subrouter. Endpoints that set their own
    // Cache-Control header (ETag detail endpoints, trending) are not overridden
    // by the layer — it only injects headers on responses that lack them.
    let cache_short =
        CacheControlLayer::new("public, max-age=60, s-maxage=60, stale-while-revalidate=300");
    let cache_backgrounds =
        CacheControlLayer::new("public, max-age=300, s-maxage=300, stale-while-revalidate=600");
    let cache_featured = CacheControlLayer::new("public, max-age=60, s-maxage=30");
    let cache_stats =
        CacheControlLayer::new("public, max-age=300, s-maxage=300, stale-while-revalidate=600");
    let no_store = CacheControlLayer::new("no-store");

    Router::new()
        .nest("/auth", auth::router().layer(make_layer(&rl.auth, "auth")))
        .nest(
            "/device",
            device::router().layer(make_layer(&rl.device, "device")),
        )
        .nest(
            "/runs",
            runs::router()
                .layer(no_store.clone())
                .layer(make_layer(&rl.agent, "agent")),
        )
        .merge(runs::failure_router().layer(make_layer(&rl.agent, "agent")))
        .merge(runs::upload_router().layer(make_layer(&rl.agent, "agent")))
        .nest(
            "/backgrounds",
            backgrounds::router().layer(cache_backgrounds),
        )
        .nest("/user", user::router())
        .nest(
            "/shaders",
            shaders::router()
                .merge(adopt::router())
                .layer(cache_short.clone()),
        )
        .nest("/scenes", scenes::router().layer(cache_short.clone()))
        .nest("/captures", captures::router().layer(cache_short))
        .nest("/stats", stats::router().layer(cache_stats))
        .nest("/users", users::router())
        .nest(
            "/work",
            work::router()
                .layer(no_store)
                .layer(make_layer(&rl.agent, "agent")),
        )
        .nest("/featured", featured::router().layer(cache_featured))
        .nest("/admin/capture-health", capture_health::router())
        .nest("/admin/shaders", extraction::router())
        .nest("/admin/storage", storage::router())
        // Intentional double rate limiting: routes like /auth and /device
        // have tier-specific limiters that enforce tight per-category budgets. The
        // global limiter below acts as an overall request ceiling across all routes.
        // Requests to tier-specific routes consume tokens from both their tier bucket
        // and the global bucket — this is by design.
        .layer(make_layer(&rl.global, "global"))
}

/// Health check — must never be cached.
#[instrument]
async fn health() -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static str,
) {
    ([(axum::http::header::CACHE_CONTROL, "no-store")], "ok")
}
