//! Per-request tracing spans with ULID-based request IDs.
//!
//! Creates an `info_span!("request", req_id = ...)` for every request,
//! logs request method/path at DEBUG, and logs response status/duration
//! at a level proportional to severity (2xx=debug, 4xx=info, 5xx=warn).

use axum::{body::Body, extract::Request, response::Response};
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use tracing::Instrument;

#[derive(Clone)]
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S> Service<Request> for RequestIdService<S>
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
        let req_id = ulid::Ulid::new().to_string();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let span = tracing::info_span!("request", req_id = %req_id, method = %method, path = %path);
        let start = Instant::now();

        let future = self.inner.call(req);

        Box::pin(
            async move {
                let result = future.await;

                let duration_ms = start.elapsed().as_millis() as u64;

                match &result {
                    Ok(response) => {
                        let status = response.status();
                        match status.as_u16() {
                            200..=399 => {
                                tracing::debug!(method = %method, path = %path, status = status.as_u16(), duration_ms, "Response")
                            }
                            400..=499 => {
                                tracing::info!(method = %method, path = %path, status = status.as_u16(), duration_ms, "Response")
                            }
                            _ => {
                                tracing::warn!(method = %method, path = %path, status = status.as_u16(), duration_ms, "Response")
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(method = %method, path = %path, error = ?e, duration_ms, "Request failed");
                    }
                }

                result
            }
            .instrument(span),
        )
    }
}
