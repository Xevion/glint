use std::sync::Arc;

use crate::{config::Config, db::DbPool};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub db: DbPool,
    pub config: Config,
}

impl AppState {
    pub fn new(db: DbPool, config: Config) -> Self {
        Self {
            inner: Arc::new(AppStateInner { db, config }),
        }
    }

    pub fn db(&self) -> &DbPool {
        &self.inner.db
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }
}
