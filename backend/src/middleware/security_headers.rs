//! Middleware that injects security headers on all API responses.
//!
//! These complement the headers set by SvelteKit for HTML responses.
//! The Axum backend is only directly reachable by internal services and the
//! Minecraft mod, but defense-in-depth means we set them here too.

use axum::{body::Body, extract::Request, http::HeaderValue, response::Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct SecurityHeadersLayer;

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersService { inner }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersService<S> {
    inner: S,
}

impl<S> Service<Request> for SecurityHeadersService<S>
where
    S: Service<Request, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let future = self.inner.call(req);

        Box::pin(async move {
            let mut response = future.await?;
            let headers = response.headers_mut();

            headers.insert(
                "x-content-type-options",
                HeaderValue::from_static("nosniff"),
            );
            headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
            headers.insert(
                "referrer-policy",
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                "permissions-policy",
                HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
            );

            Ok(response)
        })
    }
}
