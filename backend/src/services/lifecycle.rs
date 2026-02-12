use std::future::Future;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{info, warn};

/// Manages background service lifecycle with graceful shutdown.
///
/// Services are spawned via [`spawn`](ServiceRunner::spawn) and receive a
/// [`ServiceContext`] for cooperative cancellation. On shutdown, all services
/// are signaled simultaneously and given a bounded grace period to finish
/// in-progress work before the process exits.
pub struct ServiceRunner {
    token: CancellationToken,
    tracker: TaskTracker,
}

/// Handle given to each background service for cooperative shutdown.
///
/// Services should check for cancellation at natural yield points (between
/// interval ticks, between channel receives, between batch items) rather
/// than mid-operation. This ensures work-in-progress completes cleanly.
#[derive(Clone)]
pub struct ServiceContext {
    token: CancellationToken,
}

impl Default for ServiceRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRunner {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            tracker: TaskTracker::new(),
        }
    }

    /// Spawn a named background service.
    ///
    /// The service function receives a [`ServiceContext`] and should return
    /// when `ctx.cancelled()` fires. Each service gets a child token so
    /// individual service failures don't bypass the parent shutdown flow.
    pub fn spawn<F, Fut>(&self, name: &'static str, f: F)
    where
        F: FnOnce(ServiceContext) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let ctx = ServiceContext {
            token: self.token.child_token(),
        };
        self.tracker.spawn(async move {
            info!(service = name, "Service started");
            f(ctx).await;
            info!(service = name, "Service stopped");
        });
    }

    /// Signal all services to shut down, wait up to `timeout` for completion.
    ///
    /// Returns `true` if every service exited within the deadline.
    pub async fn shutdown(self, timeout: Duration) -> bool {
        info!("Signaling background services to shut down");
        self.token.cancel();
        self.tracker.close();

        tokio::select! {
            () = self.tracker.wait() => {
                info!("All background services stopped");
                true
            }
            () = tokio::time::sleep(timeout) => {
                warn!(
                    timeout_secs = timeout.as_secs(),
                    "Shutdown timed out, dropping remaining services"
                );
                false
            }
        }
    }
}

impl ServiceContext {
    /// Future that completes when shutdown is requested. Use in `tokio::select!`.
    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }

    /// Non-blocking check for whether shutdown has been requested.
    pub fn is_shutting_down(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Cancellation-aware interval tick.
    ///
    /// Returns `true` if the interval fired normally, `false` if shutdown was
    /// requested. Designed for the `while ctx.tick(&mut interval).await { ... }`
    /// pattern that replaces bare `loop { interval.tick().await; ... }`.
    pub async fn tick(&self, interval: &mut tokio::time::Interval) -> bool {
        tokio::select! {
            _ = interval.tick() => true,
            () = self.token.cancelled() => false,
        }
    }

    /// Cancellation-aware sleep.
    ///
    /// Returns `true` if the full duration elapsed, `false` if shutdown
    /// interrupted it.
    pub async fn sleep(&self, duration: Duration) -> bool {
        tokio::select! {
            () = tokio::time::sleep(duration) => true,
            () = self.token.cancelled() => false,
        }
    }
}

/// Wait for an OS termination signal (SIGINT or SIGTERM).
///
/// Used as the trigger for the Axum graceful shutdown hook and for
/// initiating background service teardown.
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");

        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result.expect("failed to listen for SIGINT");
                info!("Received SIGINT");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
        info!("Received Ctrl+C");
    }
}
