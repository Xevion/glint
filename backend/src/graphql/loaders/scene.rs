use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::Loader;
use sqlx::PgPool;

use crate::models::{ScenePreset, SceneVersion, Tag};
use crate::repo::{CaptureRepo, ScenePresetRepo, SceneVersionRepo, TagRepo, ThumbnailInfo};

pub struct SceneThumbnailLoader {
    pool: PgPool,
}

impl SceneThumbnailLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for SceneThumbnailLoader {
    type Value = ThumbnailInfo;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        CaptureRepo::batch_thumbnails_for_scenes(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct SceneCaptureCountLoader {
    pool: PgPool,
}

impl SceneCaptureCountLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for SceneCaptureCountLoader {
    type Value = i64;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        CaptureRepo::batch_count_for_scenes(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct SceneLatestVersionLoader {
    pool: PgPool,
}

impl SceneLatestVersionLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for SceneLatestVersionLoader {
    type Value = SceneVersion;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let map = SceneVersionRepo::get_latest_batch(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))?;

        // Re-key from SceneId to String
        Ok(map.into_iter().map(|(k, v)| (k.0, v)).collect())
    }
}

pub struct ScenePresetsLoader {
    pool: PgPool,
}

impl ScenePresetsLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ScenePresetsLoader {
    type Value = Vec<ScenePreset>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        ScenePresetRepo::list_by_scenes(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct SceneTagsLoader {
    pool: PgPool,
}

impl SceneTagsLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for SceneTagsLoader {
    type Value = Vec<Tag>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        TagRepo::list_for_scenes(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}
