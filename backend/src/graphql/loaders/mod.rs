pub mod shader;

use async_graphql::dataloader::DataLoader;
use sqlx::PgPool;

use shader::{
    ShaderAuthorsLoader, ShaderCategoriesLoader, ShaderExtractionSummaryLoader,
    ShaderFeaturesLoader, ShaderLatestVersionLoader, ShaderThumbnailLoader,
};

/// Request-scoped collection of DataLoaders. Created fresh per GraphQL request
/// so each request gets isolated batching and caching.
pub struct RequestLoaders {
    pub shader_authors: DataLoader<ShaderAuthorsLoader>,
    pub shader_categories: DataLoader<ShaderCategoriesLoader>,
    pub shader_features: DataLoader<ShaderFeaturesLoader>,
    pub shader_thumbnails: DataLoader<ShaderThumbnailLoader>,
    pub shader_latest_versions: DataLoader<ShaderLatestVersionLoader>,
    pub shader_extraction_summaries: DataLoader<ShaderExtractionSummaryLoader>,
}

impl RequestLoaders {
    pub fn new(pool: PgPool) -> Self {
        Self {
            shader_authors: DataLoader::new(ShaderAuthorsLoader::new(pool.clone()), tokio::spawn),
            shader_categories: DataLoader::new(
                ShaderCategoriesLoader::new(pool.clone()),
                tokio::spawn,
            ),
            shader_features: DataLoader::new(ShaderFeaturesLoader::new(pool.clone()), tokio::spawn),
            shader_thumbnails: DataLoader::new(
                ShaderThumbnailLoader::new(pool.clone()),
                tokio::spawn,
            ),
            shader_latest_versions: DataLoader::new(
                ShaderLatestVersionLoader::new(pool.clone()),
                tokio::spawn,
            ),
            shader_extraction_summaries: DataLoader::new(
                ShaderExtractionSummaryLoader::new(pool),
                tokio::spawn,
            ),
        }
    }
}
