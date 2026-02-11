use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;
use tracing::warn;

/// Extracted client IP address, accounting for reverse proxies.
///
/// Uses the same logic as the rate limiter: parse `X-Forwarded-For` when
/// `trusted_proxy_hops > 0`, otherwise fall back to `ConnectInfo`.
///
/// Because the number of trusted hops comes from app config (not the request),
/// handlers should use [`ClientIp::resolve`] with the hop count from state.
#[derive(Debug, Clone)]
pub struct ClientIp(pub String);

impl ClientIp {
    /// Resolve the client IP from request headers and connection info.
    pub fn resolve(headers: &HeaderMap, connect_info: Option<SocketAddr>, hops: u8) -> Self {
        if hops > 0
            && let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        {
            let parts: Vec<&str> = xff.split(',').map(|s| s.trim()).collect();
            let idx = parts.len().saturating_sub(hops as usize + 1);
            if let Some(ip_str) = parts.get(idx)
                && ip_str.parse::<IpAddr>().is_ok()
            {
                return Self((*ip_str).to_string());
            }
        }

        Self(
            connect_info
                .map(|ci| ci.ip().to_string())
                .unwrap_or_else(|| {
                    warn!("No ConnectInfo or X-Forwarded-For available for client IP");
                    "unknown".to_string()
                }),
        )
    }
}
