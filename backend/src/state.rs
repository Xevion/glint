use std::ops::Deref;
use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use oauth2::{EndpointNotSet, EndpointSet};

use crate::{
    config::Config,
    db::{DbPool, DbTransaction},
    error::AppResult,
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
}

impl AppState {
    pub fn new(
        db: DbPool,
        config: Config,
        s3: Option<S3Client>,
        oauth: Option<OAuthClient>,
        modrinth: ModrinthClient,
        curseforge: Option<CurseForgeClient>,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                config,
                s3,
                oauth,
                modrinth,
                curseforge,
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
