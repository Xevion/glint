use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::Loader;
use sqlx::PgPool;

use crate::id::ShaderVersionId;
use crate::models::{Category, ExtractionSummary, Feature, ShaderAuthor, ShaderVersion};
use crate::repo::{
    CaptureRepo, CategoryRepo, FeatureRepo, ShaderAuthorRepo, ShaderVersionRepo, ThumbnailInfo,
};

pub struct ShaderAuthorsLoader {
    pool: PgPool,
}

impl ShaderAuthorsLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderAuthorsLoader {
    type Value = Vec<ShaderAuthor>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        ShaderAuthorRepo::list_by_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct ShaderCategoriesLoader {
    pool: PgPool,
}

impl ShaderCategoriesLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderCategoriesLoader {
    type Value = Vec<Category>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        CategoryRepo::list_for_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct ShaderFeaturesLoader {
    pool: PgPool,
}

impl ShaderFeaturesLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderFeaturesLoader {
    type Value = Vec<Feature>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        FeatureRepo::list_for_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct ShaderThumbnailLoader {
    pool: PgPool,
}

impl ShaderThumbnailLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderThumbnailLoader {
    type Value = ThumbnailInfo;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        CaptureRepo::batch_thumbnails_for_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}

pub struct ShaderLatestVersionLoader {
    pool: PgPool,
}

impl ShaderLatestVersionLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderLatestVersionLoader {
    type Value = ShaderVersion;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let map = ShaderVersionRepo::batch_latest_versions_for_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))?;

        // Re-key from ShaderId to String
        Ok(map.into_iter().map(|(k, v)| (k.0, v)).collect())
    }
}

pub struct ShaderExtractionSummaryLoader {
    pool: PgPool,
}

impl ShaderExtractionSummaryLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderExtractionSummaryLoader {
    type Value = ExtractionSummary;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let map = ShaderVersionRepo::batch_extraction_summaries_for_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))?;

        Ok(map.into_iter().map(|(k, v)| (k.0, v)).collect())
    }
}

pub struct ShaderLatestVersionIdLoader {
    pool: PgPool,
}

impl ShaderLatestVersionIdLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderLatestVersionIdLoader {
    type Value = ShaderVersionId;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let map = ShaderVersionRepo::batch_latest_versions_for_shaders(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))?;

        Ok(map.into_iter().map(|(k, v)| (k.0, v.id)).collect())
    }
}

pub struct ShaderVersionCountLoader {
    pool: PgPool,
}

impl ShaderVersionCountLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Loader<String> for ShaderVersionCountLoader {
    type Value = i64;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        ShaderVersionRepo::batch_version_counts(&self.pool, keys)
            .await
            .map_err(|e| Arc::new(sqlx::Error::Protocol(e.to_string())))
    }
}
