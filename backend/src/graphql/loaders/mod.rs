pub mod scene;
pub mod shader;

use async_graphql::dataloader::DataLoader;
use sqlx::PgPool;

use scene::{
    SceneCaptureCountLoader, SceneLatestVersionLoader, ScenePresetsLoader, SceneTagsLoader,
    SceneThumbnailLoader,
};
use shader::{
    ShaderAuthorsLoader, ShaderCategoriesLoader, ShaderExtractionSummaryLoader,
    ShaderFeaturesLoader, ShaderLatestVersionIdLoader, ShaderLatestVersionLoader,
    ShaderThumbnailLoader, ShaderVersionCountLoader,
};

/// Request-scoped collection of DataLoaders. Created fresh per GraphQL request
/// so each request gets isolated batching and caching.
pub struct RequestLoaders {
    pub shader_authors: DataLoader<ShaderAuthorsLoader>,
    pub shader_categories: DataLoader<ShaderCategoriesLoader>,
    pub shader_features: DataLoader<ShaderFeaturesLoader>,
    pub shader_thumbnails: DataLoader<ShaderThumbnailLoader>,
    pub shader_latest_versions: DataLoader<ShaderLatestVersionLoader>,
    pub shader_latest_version_ids: DataLoader<ShaderLatestVersionIdLoader>,
    pub shader_extraction_summaries: DataLoader<ShaderExtractionSummaryLoader>,
    pub shader_version_counts: DataLoader<ShaderVersionCountLoader>,
    pub scene_thumbnails: DataLoader<SceneThumbnailLoader>,
    pub scene_capture_counts: DataLoader<SceneCaptureCountLoader>,
    pub scene_latest_versions: DataLoader<SceneLatestVersionLoader>,
    pub scene_presets: DataLoader<ScenePresetsLoader>,
    pub scene_tags: DataLoader<SceneTagsLoader>,
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
            shader_latest_version_ids: DataLoader::new(
                ShaderLatestVersionIdLoader::new(pool.clone()),
                tokio::spawn,
            ),
            shader_extraction_summaries: DataLoader::new(
                ShaderExtractionSummaryLoader::new(pool.clone()),
                tokio::spawn,
            ),
            shader_version_counts: DataLoader::new(
                ShaderVersionCountLoader::new(pool.clone()),
                tokio::spawn,
            ),
            scene_thumbnails: DataLoader::new(
                SceneThumbnailLoader::new(pool.clone()),
                tokio::spawn,
            ),
            scene_capture_counts: DataLoader::new(
                SceneCaptureCountLoader::new(pool.clone()),
                tokio::spawn,
            ),
            scene_latest_versions: DataLoader::new(
                SceneLatestVersionLoader::new(pool.clone()),
                tokio::spawn,
            ),
            scene_presets: DataLoader::new(ScenePresetsLoader::new(pool.clone()), tokio::spawn),
            scene_tags: DataLoader::new(SceneTagsLoader::new(pool), tokio::spawn),
        }
    }
}
