use std::ops::Deref;
use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;

use crate::{config::Config, db::DbPool};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub db: DbPool,
    pub config: Config,
    pub s3: Option<S3Client>,
}

impl AppState {
    pub fn new(db: DbPool, config: Config, s3: Option<S3Client>) -> Self {
        Self {
            inner: Arc::new(AppStateInner { db, config, s3 }),
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
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
