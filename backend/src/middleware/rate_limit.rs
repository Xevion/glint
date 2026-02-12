use std::{
    future::Future,
    net::SocketAddr,
    num::NonZeroU32,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    response::{IntoResponse, Response},
};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter as GovernorRateLimiter, clock::Clock};
use serde::Deserialize;
use tower::{Layer, Service};
use tracing::warn;

use crate::analytics::Analytics;
use crate::error::AppError;
use crate::middleware::client_ip::ClientIp;

/// Top-level rate limiting configuration, deserialized from `[rate_limit]`.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub trusted_proxy_hops: u8,

    #[serde(default = "TierConfig::default_global")]
    pub global: TierConfig,

    #[serde(default = "TierConfig::default_auth")]
    pub auth: TierConfig,

    #[serde(default = "TierConfig::default_device")]
    pub device: TierConfig,

    #[serde(default = "TierConfig::default_upload")]
    pub upload: TierConfig,

    #[serde(default = "TierConfig::default_agent")]
    pub agent: TierConfig,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trusted_proxy_hops: 0,
            global: TierConfig::default_global(),
            auth: TierConfig::default_auth(),
            device: TierConfig::default_device(),
            upload: TierConfig::default_upload(),
            agent: TierConfig::default_agent(),
        }
    }
}

fn default_true() -> bool {
    true
}

/// Per-tier rate limit settings.
#[derive(Debug, Clone, Deserialize)]
pub struct TierConfig {
    pub requests_per_minute: NonZeroU32,
    pub burst: NonZeroU32,
}

impl TierConfig {
    pub fn default_global() -> Self {
        Self {
            requests_per_minute: NonZeroU32::new(300).unwrap(),
            burst: NonZeroU32::new(50).unwrap(),
        }
    }

    pub fn default_auth() -> Self {
        Self {
            requests_per_minute: NonZeroU32::new(10).unwrap(),
            burst: NonZeroU32::new(5).unwrap(),
        }
    }

    pub fn default_device() -> Self {
        Self {
            requests_per_minute: NonZeroU32::new(15).unwrap(),
            burst: NonZeroU32::new(5).unwrap(),
        }
    }

    pub fn default_upload() -> Self {
        Self {
            requests_per_minute: NonZeroU32::new(30).unwrap(),
            burst: NonZeroU32::new(10).unwrap(),
        }
    }

    pub fn default_agent() -> Self {
        Self {
            requests_per_minute: NonZeroU32::new(600).unwrap(),
            burst: NonZeroU32::new(100).unwrap(),
        }
    }

    fn to_quota(&self) -> Quota {
        Quota::per_minute(self.requests_per_minute).allow_burst(self.burst)
    }
}

/// Aborts the background cleanup task when dropped.
struct CleanupGuard(tokio::task::JoinHandle<()>);

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Thread-safe, Arc-wrapped keyed rate limiter. Clone is cheap.
/// The background cleanup task is automatically aborted when all clones are dropped.
#[derive(Clone)]
struct Limiter {
    inner: Arc<DefaultKeyedRateLimiter<String>>,
    /// Shared ownership ensures the task lives as long as any Limiter clone,
    /// then aborts via [`CleanupGuard::drop`].
    _cleanup: Arc<CleanupGuard>,
}

impl Limiter {
    fn new(tier: &TierConfig) -> Self {
        let inner = Arc::new(GovernorRateLimiter::keyed(tier.to_quota()));

        let limiter = Arc::clone(&inner);
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                limiter.retain_recent();
                limiter.shrink_to_fit();
            }
        });

        Self {
            inner,
            _cleanup: Arc::new(CleanupGuard(handle)),
        }
    }

    /// Check if the given key is allowed. Returns `Ok(())` or the number of
    /// seconds until the next request would be allowed.
    fn check(&self, key: &String) -> Result<(), u64> {
        match self.inner.check_key(key) {
            Ok(()) => Ok(()),
            Err(not_until) => {
                let wait = not_until.wait_time_from(governor::clock::DefaultClock::default().now());
                Err(wait.as_secs().max(1))
            }
        }
    }
}

/// Extract client IP from the request, respecting `X-Forwarded-For` when
/// `trusted_proxy_hops > 0`. Delegates to [`ClientIp::resolve`] for the
/// actual parsing logic.
fn extract_client_ip(
    headers: &axum::http::HeaderMap,
    connect_info: Option<SocketAddr>,
    hops: u8,
) -> String {
    ClientIp::resolve(headers, connect_info, hops).0
}

/// Tower [`Layer`] that applies rate limiting to requests.
///
/// Construct via [`RateLimitLayer::new`] for a specific tier. Set
/// `enabled: false` in the config to get pass-through behavior.
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Option<Limiter>,
    tier_name: &'static str,
    trusted_proxy_hops: u8,
    analytics: Option<Analytics>,
}

impl RateLimitLayer {
    /// Create a rate limit layer for the given tier.
    ///
    /// If `enabled` is false, returns a no-op layer that always forwards.
    pub fn new(
        tier: &TierConfig,
        tier_name: &'static str,
        trusted_proxy_hops: u8,
        enabled: bool,
        analytics: Option<Analytics>,
    ) -> Self {
        Self {
            limiter: if enabled {
                Some(Limiter::new(tier))
            } else {
                None
            },
            tier_name,
            trusted_proxy_hops,
            analytics,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
            tier_name: self.tier_name,
            trusted_proxy_hops: self.trusted_proxy_hops,
            analytics: self.analytics.clone(),
        }
    }
}

/// Tower [`Service`] that checks a keyed rate limiter before forwarding.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Option<Limiter>,
    tier_name: &'static str,
    trusted_proxy_hops: u8,
    analytics: Option<Analytics>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // Swap self.inner with a fresh clone so the next poll_ready cycle
        // operates on an un-polled service, while we use the already-polled
        // original to serve this request.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        // No-op when disabled
        let Some(ref limiter) = self.limiter else {
            return Box::pin(async move { inner.call(req).await });
        };

        let limiter = limiter.clone();
        let tier_name = self.tier_name;
        let hops = self.trusted_proxy_hops;
        let analytics = self.analytics.clone();

        // Extract IP before consuming request
        let connect_info = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0);
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let client_ip = extract_client_ip(req.headers(), connect_info, hops);

        Box::pin(async move {
            match limiter.check(&client_ip) {
                Ok(()) => inner.call(req).await,
                Err(retry_after_secs) => {
                    warn!(
                        client_ip = %client_ip,
                        tier = tier_name,
                        path = %path,
                        retry_after = retry_after_secs,
                        "Rate limited"
                    );

                    if let Some(ref analytics) = analytics {
                        analytics.track(
                            "rate_limit_hit",
                            &client_ip,
                            serde_json::json!({
                                "tier": tier_name,
                                "method": method,
                                "path": path,
                                "retry_after_secs": retry_after_secs,
                            }),
                        );
                    }

                    Ok(AppError::RateLimited { retry_after_secs }.into_response())
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn extract_ip_direct_connection() {
        let headers = HeaderMap::new();
        let addr: SocketAddr = "192.168.1.1:12345".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, Some(addr), 0), "192.168.1.1");
    }

    #[test]
    fn extract_ip_xff_single_proxy() {
        // XFF: "client, proxy_added_entry" — 1 trusted proxy appended the last entry
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.50, 10.0.0.1".parse().unwrap());
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, Some(addr), 1), "203.0.113.50");
    }

    #[test]
    fn extract_ip_xff_two_proxies() {
        // XFF: "client, proxy1_entry, proxy2_entry" — 2 trusted proxies
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.50, 10.0.0.1, 10.0.0.2".parse().unwrap(),
        );
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, Some(addr), 2), "203.0.113.50");
    }

    #[test]
    fn extract_ip_xff_ignored_when_no_trust() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.50".parse().unwrap());
        let addr: SocketAddr = "10.0.0.1:80".parse().unwrap();
        assert_eq!(extract_client_ip(&headers, Some(addr), 0), "10.0.0.1");
    }

    #[test]
    fn extract_ip_no_connect_info() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers, None, 0), "unknown");
    }

    #[test]
    fn extract_ip_xff_invalid_ip_falls_back() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "not-an-ip, 10.0.0.1".parse().unwrap());
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        // Invalid IP at the client position — falls back to ConnectInfo
        assert_eq!(extract_client_ip(&headers, Some(addr), 1), "127.0.0.1");
    }

    #[test]
    fn tier_config_defaults() {
        let global = TierConfig::default_global();
        assert_eq!(global.requests_per_minute.get(), 300);
        assert_eq!(global.burst.get(), 50);

        let auth = TierConfig::default_auth();
        assert_eq!(auth.requests_per_minute.get(), 10);
        assert_eq!(auth.burst.get(), 5);
    }
}
