use std::collections::HashMap;

use tracing::{debug, instrument};

use crate::{
    db::DbPool,
    error::AppResult,
    models::{ShaderListItem, TrendingShader},
    repo::{
        CaptureRepo, CategoryRepo, FeatureRepo, ShaderAuthorRepo, ShaderRepo, ShaderVersionRepo,
        ShaderViewRepo,
    },
};

pub struct ShaderService;

impl ShaderService {
    /// Fetch all shaders enriched with authors, categories, features, latest
    /// version, and a deterministic thumbnail.
    ///
    /// Only shaders that have at least one completed capture (thumbnail) are
    /// included — the rest are hidden from the public listing.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_enriched(db: &DbPool) -> AppResult<Vec<ShaderListItem>> {
        let (shaders, authors, categories, features, versions, thumbnails) = tokio::try_join!(
            ShaderRepo::list(db),
            ShaderAuthorRepo::list_all(db),
            CategoryRepo::list_all_for_shaders(db),
            FeatureRepo::list_all_for_shaders(db),
            ShaderVersionRepo::batch_latest_versions(db),
            CaptureRepo::batch_thumbnails_by_shader(db),
        )?;

        // Group by shader_id
        let mut authors_map: HashMap<String, Vec<_>> = HashMap::new();
        for a in authors {
            authors_map
                .entry(a.shader_id.0.clone())
                .or_default()
                .push(a);
        }

        let mut categories_map: HashMap<String, Vec<_>> = HashMap::new();
        for (sid, cat) in categories {
            categories_map.entry(sid).or_default().push(cat);
        }

        let mut features_map: HashMap<String, Vec<_>> = HashMap::new();
        for (sid, feat) in features {
            features_map.entry(sid).or_default().push(feat);
        }

        let items = shaders
            .into_iter()
            .filter(|shader| thumbnails.contains_key(shader.id.as_ref()))
            .map(|shader| {
                let id = &shader.id;
                let id_str: &str = id.as_ref();
                let version = versions.get(id);
                let thumb = thumbnails.get(id_str);
                ShaderListItem {
                    authors: authors_map.remove(id_str).unwrap_or_default(),
                    categories: categories_map.remove(id_str).unwrap_or_default(),
                    features: features_map.remove(id_str).unwrap_or_default(),
                    latest_version: version.map(|v| v.version.clone()),
                    game_versions: version.and_then(|v| v.game_versions.clone()),
                    image_url: thumb.map(|t| t.image_url.clone()),
                    thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                    shader,
                }
            })
            .collect::<Vec<_>>();

        debug!(count = items.len(), "Built enriched shader list");
        Ok(items)
    }

    /// Get trending shaders by recent view count, enriched with thumbnails.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_trending(
        db: &DbPool,
        days: i32,
        limit: i64,
    ) -> AppResult<Vec<TrendingShader>> {
        let trending = ShaderViewRepo::trending(db, days, limit).await?;

        if trending.is_empty() {
            return Ok(vec![]);
        }

        let shader_ids: Vec<String> = trending.iter().map(|e| e.shader_id.0.clone()).collect();

        let (shaders_map, thumbnails) = tokio::try_join!(
            ShaderRepo::get_many(db, &shader_ids),
            CaptureRepo::batch_thumbnails_for_shaders(db, &shader_ids),
        )?;

        let result = trending
            .into_iter()
            .filter_map(|entry| {
                let id = entry.shader_id.as_ref();
                let shader = shaders_map.get(id).cloned();
                if shader.is_none() {
                    tracing::debug!(shader_id = id, "Trending shader not found, skipping");
                }
                let thumb = thumbnails.get(id);
                shader.map(|s| TrendingShader {
                    shader: s,
                    trending_views: entry.view_count,
                    image_url: thumb.map(|t| t.image_url.clone()),
                    thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                })
            })
            .collect();

        Ok(result)
    }
}
