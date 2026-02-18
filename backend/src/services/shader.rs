use std::collections::HashMap;

use tracing::{debug, instrument};

use crate::{
    db::DbPool,
    error::AppResult,
    models::{Page, ShaderListItem, TrendingShader},
    repo::{
        CaptureRepo, CategoryRepo, FeatureRepo, ShaderAuthorRepo, ShaderRepo, ShaderVersionRepo,
        ShaderViewRepo,
    },
};

pub struct ShaderService;

impl ShaderService {
    /// Fetch shaders enriched with authors, categories, features, latest
    /// version, and a deterministic thumbnail with pagination support.
    ///
    /// When `include_all` is `false` (public listing), only shaders with at
    /// least one completed capture (thumbnail) are included. When `true`
    /// (admin listing), all shaders are returned.
    ///
    /// Pagination and thumbnail filtering are pushed to SQL so we only fetch
    /// the requested page of shaders, then enrich those in parallel.
    ///
    /// Returns `(items, total)` where total is the count before pagination.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_enriched(
        db: &DbPool,
        page: &Page,
        search: Option<&str>,
        sort: Option<&str>,
        include_all: bool,
    ) -> AppResult<(Vec<ShaderListItem>, i64)> {
        let (shaders, total) =
            ShaderRepo::list_paginated(db, page, search, sort, !include_all).await?;

        if shaders.is_empty() {
            return Ok((vec![], total));
        }

        let shader_ids: Vec<String> = shaders.iter().map(|s| s.id.0.clone()).collect();

        // Fetch enrichment data in parallel (only for this page)
        let (authors, categories, features, versions, thumbnails, mut extraction_summaries) = tokio::try_join!(
            ShaderAuthorRepo::list_all(db),
            CategoryRepo::list_all_for_shaders(db),
            FeatureRepo::list_all_for_shaders(db),
            ShaderVersionRepo::batch_latest_versions(db),
            CaptureRepo::batch_thumbnails_for_shaders(db, &shader_ids),
            ShaderVersionRepo::batch_extraction_summaries(db),
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
            .map(|shader| {
                let id = &shader.id;
                let id_str: &str = id.as_ref();
                let version = versions.get(id);
                let thumb = thumbnails.get(id_str);
                let summary = extraction_summaries.remove(id);
                let version_count = summary.as_ref().map_or(0, |s| s.total);
                ShaderListItem {
                    authors: authors_map.remove(id_str).unwrap_or_default(),
                    categories: categories_map.remove(id_str).unwrap_or_default(),
                    features: features_map.remove(id_str).unwrap_or_default(),
                    latest_version: version.map(|v| v.version.clone()),
                    image_path: thumb.map(|t| t.image_path.clone()),
                    thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                    version_count,
                    extraction_summary: summary,
                    shader,
                }
            })
            .collect();

        debug!(count = total, "Built enriched shader list");
        Ok((items, total))
    }

    /// Get trending shaders by recent view count, enriched with thumbnails.
    #[instrument(skip(db), level = "debug")]
    pub async fn list_trending(
        db: &DbPool,
        days: u32,
        page: &Page,
    ) -> AppResult<Vec<TrendingShader>> {
        let trending = ShaderViewRepo::trending(db, days as i32, page.limit_i64()).await?;

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
                    image_path: thumb.map(|t| t.image_path.clone()),
                    thumbhash: thumb.and_then(|t| t.thumbhash.clone()),
                })
            })
            .collect();

        Ok(result)
    }
}
