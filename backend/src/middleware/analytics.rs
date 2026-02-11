use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use axum::{body::Body, extract::Request, response::Response};
use tower::{Layer, Service};

use crate::analytics::Analytics;

#[derive(Clone)]
pub struct AnalyticsLayer {
    analytics: Option<Analytics>,
}

impl AnalyticsLayer {
    pub fn new(analytics: Option<Analytics>) -> Self {
        Self { analytics }
    }
}

impl<S> Layer<S> for AnalyticsLayer {
    type Service = AnalyticsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AnalyticsMiddleware {
            inner,
            analytics: self.analytics.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AnalyticsMiddleware<S> {
    inner: S,
    analytics: Option<Analytics>,
}

impl<S> Service<Request<Body>> for AnalyticsMiddleware<S>
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
        let analytics = self.analytics.clone();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let start = Instant::now();

        let mut inner = self.inner.clone();
        Box::pin(async move {
            let response = inner.call(req).await?;

            // Only track API routes (skip health checks, static assets, etc.)
            if let Some(ref analytics) = analytics
                && path.starts_with("/api")
                && path != "/api/health"
            {
                let duration_ms = start.elapsed().as_millis() as u64;
                let status = response.status().as_u16();
                analytics.track(
                    "api_request",
                    "system",
                    serde_json::json!({
                        "method": method,
                        "path": path,
                        "status": status,
                        "duration_ms": duration_ms,
                    }),
                );
            }

            Ok(response)
        })
    }
}
