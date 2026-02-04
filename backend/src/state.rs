use std::ops::Deref;
use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use oauth2::{EndpointNotSet, EndpointSet};

use crate::{config::Config, db::DbPool};

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
}

impl AppState {
    pub fn new(
        db: DbPool,
        config: Config,
        s3: Option<S3Client>,
        oauth: Option<OAuthClient>,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                config,
                s3,
                oauth,
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
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
