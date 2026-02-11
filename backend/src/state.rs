use std::ops::Deref;
use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use oauth2::{EndpointNotSet, EndpointSet};
use tokio::sync::mpsc;

use crate::{
    analytics::Analytics,
    cache::SessionCache,
    config::Config,
    db::{DbPool, DbTransaction},
    error::{AppError, AppResult},
    platform::{curseforge::CurseForgeClient, modrinth::ModrinthClient},
};

/// OAuth2 client with auth and token endpoints set (required for authorization code flow)
pub type OAuthClient = oauth2::basic::BasicClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub db: DbPool,
    pub config: Config,
    pub s3: Option<S3Client>,
    pub oauth: Option<OAuthClient>,
    pub modrinth: ModrinthClient,
    pub curseforge: Option<CurseForgeClient>,
    pub metadata_tx: mpsc::UnboundedSender<String>,
    pub analytics: Option<Analytics>,
    pub session_cache: SessionCache,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: DbPool,
        config: Config,
        s3: Option<S3Client>,
        oauth: Option<OAuthClient>,
        modrinth: ModrinthClient,
        curseforge: Option<CurseForgeClient>,
        metadata_tx: mpsc::UnboundedSender<String>,
        analytics: Option<Analytics>,
        session_cache: SessionCache,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                config,
                s3,
                oauth,
                modrinth,
                curseforge,
                metadata_tx,
                analytics,
                session_cache,
            }),
        }
    }

    pub fn db(&self) -> &DbPool {
        &self.inner.db
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn s3(&self) -> Option<&S3Client> {
        self.inner.s3.as_ref()
    }

    pub fn oauth(&self) -> Option<&OAuthClient> {
        self.inner.oauth.as_ref()
    }

    pub fn modrinth(&self) -> &ModrinthClient {
        &self.inner.modrinth
    }

    pub fn curseforge(&self) -> Option<&CurseForgeClient> {
        self.inner.curseforge.as_ref()
    }

    pub fn metadata_tx(&self) -> &mpsc::UnboundedSender<String> {
        &self.inner.metadata_tx
    }

    pub fn analytics(&self) -> Option<&Analytics> {
        self.inner.analytics.as_ref()
    }

    pub fn session_cache(&self) -> &SessionCache {
        &self.inner.session_cache
    }

    /// Fire-and-forget analytics event. No-op if analytics is not configured.
    pub fn track(&self, event_name: &str, distinct_id: &str, properties: serde_json::Value) {
        if let Some(analytics) = self.analytics() {
            analytics.track(event_name, distinct_id, properties);
        }
    }

    /// Report an error to PostHog analytics and return it for handler use.
    /// Use in handlers: `return Err(state.track_err(error, "handler_name"))`
    pub fn track_err(&self, error: AppError, handler: &str) -> AppError {
        let (status, code) = error.status_and_code();
        self.track(
            "api_error",
            "system",
            serde_json::json!({
                "handler": handler,
                "status": status.as_u16(),
                "code": code,
                "message": error.to_string(),
            }),
        );
        error
    }

    pub async fn begin_tx(&self) -> AppResult<DbTransaction<'_>> {
        Ok(self.inner.db.begin().await?)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
