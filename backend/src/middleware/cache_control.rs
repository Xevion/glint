//! Middleware that injects `Cache-Control` headers on successful GET responses.
//!
//! Applied per-router to set appropriate caching directives for different
//! endpoint groups. Only affects GET requests with 2xx status codes —
//! errors and mutations pass through unchanged.

use axum::{body::Body, extract::Request, http::HeaderValue, response::Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct CacheControlLayer {
    directive: HeaderValue,
}

impl CacheControlLayer {
    pub fn new(directive: &'static str) -> Self {
        Self {
            directive: HeaderValue::from_static(directive),
        }
    }
}

impl<S> Layer<S> for CacheControlLayer {
    type Service = CacheControlService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheControlService {
            inner,
            directive: self.directive.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CacheControlService<S> {
    inner: S,
    directive: HeaderValue,
}

impl<S> Service<Request> for CacheControlService<S>
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
        let is_get = req.method() == axum::http::Method::GET;
        let directive = self.directive.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            let mut response = future.await?;

            // Only cache successful GET responses that don't already have Cache-Control
            if is_get
                && response.status().is_success()
                && !response
                    .headers()
                    .contains_key(axum::http::header::CACHE_CONTROL)
            {
                response
                    .headers_mut()
                    .insert(axum::http::header::CACHE_CONTROL, directive);
            }

            Ok(response)
        })
    }
}
