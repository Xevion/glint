use std::sync::Arc;

use posthog_rs::{Client, Event};
use serde_json::Value;
use tracing::{debug, warn};

/// PostHog analytics client wrapper.
/// Provides fire-and-forget event tracking that never blocks request handling.
#[derive(Clone)]
pub struct Analytics {
    client: Arc<Client>,
}

impl Analytics {
    /// Create a new analytics client from a PostHog API key and host.
    pub async fn new(api_key: &str, host: &str) -> Self {
        let client = posthog_rs::client((api_key, host)).await;
        Self {
            client: Arc::new(client),
        }
    }

    /// Track an event asynchronously (fire-and-forget).
    /// Properties should be a `serde_json::Value::Object`.
    pub fn track(&self, event_name: &str, distinct_id: &str, properties: Value) {
        let mut event = Event::new(event_name, distinct_id);
        if let Value::Object(props) = properties {
            for (key, value) in props {
                if let Err(e) = event.insert_prop(&key, value) {
                    warn!(key = %key, error = %e, "Failed to insert PostHog event property");
                }
            }
        }

        let client = Arc::clone(&self.client);
        let event_name = event_name.to_owned();
        let distinct_id = distinct_id.to_owned();
        tokio::spawn(async move {
            if let Err(e) = client.capture(event).await {
                warn!(error = %e, event = %event_name, "Failed to capture PostHog event");
            } else {
                debug!(event = %event_name, distinct_id = %distinct_id, "PostHog event captured");
            }
        });
    }
}
