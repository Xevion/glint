use std::collections::HashMap;

use anyhow::Context;
use rand::seq::SliceRandom;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::{FeaturedPair, FeaturedSide};

struct CandidateRow {
    image_path: String,
    thumbhash: Option<String>,
    shader_name: String,
    shader_slug: String,
    shader_author: Option<String>,
    shader_version: String,
    scene_name: String,
    scene_id: String,
}

pub struct FeaturedRepo;

impl FeaturedRepo {
    /// Select random featured pairs from popular shaders.
    /// Roughly 50/50 split: popular-vs-vanilla and popular-vs-popular.
    pub async fn random_pairs(db: &DbPool, count: usize) -> AppResult<Vec<FeaturedPair>> {
        // Fetch completed captures from shaders ordered by popularity.
        // Only include scenes that are active and captures that have images.
        let candidates = sqlx::query_as!(
            CandidateRow,
            r#"
            SELECT DISTINCT ON (sh.id, sc.id)
                c.image_path AS "image_path!",
                c.thumbhash,
                sh.name AS "shader_name!",
                sh.slug AS "shader_slug!",
                (SELECT sa.name FROM shader_authors sa WHERE sa.shader_id = sh.id LIMIT 1) AS shader_author,
                sv.version AS "shader_version!",
                sc.name AS "scene_name!",
                sc.id AS "scene_id!"
            FROM captures c
            JOIN shader_versions sv ON c.shader_version_id = sv.id
            JOIN shaders sh ON sv.shader_id = sh.id
            JOIN scenes sc ON c.scene_id = sc.id
            WHERE c.status = 'completed'
              AND c.image_path IS NOT NULL
              AND sc.active = true
              AND sh.deleted_at IS NULL
            ORDER BY sh.id, sc.id, COALESCE(sh.upstream_downloads, 0) DESC, c.created_at DESC
            "#,
        )
        .fetch_all(db)
        .await
        .context("failed to fetch featured candidates")?;

        // Group by scene_id for pairing
        let mut by_scene: HashMap<&str, Vec<&CandidateRow>> = HashMap::new();
        for c in &candidates {
            by_scene.entry(&c.scene_id).or_default().push(c);
        }

        let mut vanilla_pairs: Vec<FeaturedPair> = Vec::new();
        let mut shader_pairs: Vec<FeaturedPair> = Vec::new();

        for scene_captures in by_scene.values() {
            let vanilla: Vec<_> = scene_captures
                .iter()
                .filter(|c| c.shader_slug == "vanilla")
                .collect();
            let non_vanilla: Vec<_> = scene_captures
                .iter()
                .filter(|c| c.shader_slug != "vanilla")
                .collect();

            // Vanilla vs popular pairs
            for v in &vanilla {
                for nv in &non_vanilla {
                    vanilla_pairs.push(make_pair(v, nv));
                }
            }

            // Popular vs popular pairs
            for (i, a) in non_vanilla.iter().enumerate() {
                for b in non_vanilla.iter().skip(i + 1) {
                    shader_pairs.push(make_pair(a, b));
                }
            }
        }

        let mut rng = rand::rng();
        vanilla_pairs.shuffle(&mut rng);
        shader_pairs.shuffle(&mut rng);

        // 50/50 split, filling from whichever pool has surplus
        let half = count / 2;
        let mut result: Vec<FeaturedPair> = Vec::with_capacity(count);
        result.extend(vanilla_pairs.into_iter().take(half));

        let remaining = count.saturating_sub(result.len());
        result.extend(shader_pairs.into_iter().take(remaining));

        // If still short (not enough shader-vs-shader), we already have what we have
        result.shuffle(&mut rng);
        Ok(result)
    }
}

fn make_side(c: &CandidateRow) -> FeaturedSide {
    FeaturedSide {
        image_path: c.image_path.clone(),
        thumbhash: c.thumbhash.clone(),
        shader_name: c.shader_name.clone(),
        shader_slug: c.shader_slug.clone(),
        shader_author: c.shader_author.clone(),
        shader_version: c.shader_version.clone(),
        scene_name: c.scene_name.clone(),
    }
}

fn make_pair(left: &CandidateRow, right: &CandidateRow) -> FeaturedPair {
    FeaturedPair {
        left: make_side(left),
        right: make_side(right),
    }
}
