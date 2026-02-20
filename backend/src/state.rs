use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use oauth2::{EndpointNotSet, EndpointSet};
use tokio::sync::{broadcast, mpsc};

use crate::{
    analytics::Analytics,
    cache::SessionCache,
    config::Config,
    db::{DbPool, DbTransaction},
    error::AppResult,
    graphql::events::DomainEvent,
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
    pub http: reqwest::Client,
    pub s3: Option<S3Client>,
    pub oauth: Option<OAuthClient>,
    pub modrinth: ModrinthClient,
    pub curseforge: Option<CurseForgeClient>,
    pub metadata_tx: mpsc::UnboundedSender<String>,
    pub event_tx: broadcast::Sender<DomainEvent>,
    pub analytics: Option<Analytics>,
    pub session_cache: SessionCache,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: DbPool,
        config: Config,
        http: reqwest::Client,
        s3: Option<S3Client>,
        oauth: Option<OAuthClient>,
        modrinth: ModrinthClient,
        curseforge: Option<CurseForgeClient>,
        metadata_tx: mpsc::UnboundedSender<String>,
        event_tx: broadcast::Sender<DomainEvent>,
        analytics: Option<Analytics>,
        session_cache: SessionCache,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                config,
                http,
                s3,
                oauth,
                modrinth,
                curseforge,
                metadata_tx,
                event_tx,
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

    pub fn http(&self) -> &reqwest::Client {
        &self.inner.http
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

    pub fn event_tx(&self) -> &broadcast::Sender<DomainEvent> {
        &self.inner.event_tx
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

    pub async fn begin_tx(&self) -> AppResult<DbTransaction<'_>> {
        Ok(self.inner.db.begin().await?)
    }
}
