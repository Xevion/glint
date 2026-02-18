use std::collections::HashMap;

use anyhow::Context;
use rand::seq::SliceRandom;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::FeaturedPair;

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

fn make_pair(left: &CandidateRow, right: &CandidateRow) -> FeaturedPair {
    FeaturedPair {
        left_image_path: left.image_path.clone(),
        left_thumbhash: left.thumbhash.clone(),
        left_shader_name: left.shader_name.clone(),
        left_shader_slug: left.shader_slug.clone(),
        left_shader_author: left.shader_author.clone(),
        left_shader_version: left.shader_version.clone(),
        left_scene_name: left.scene_name.clone(),
        right_image_path: right.image_path.clone(),
        right_thumbhash: right.thumbhash.clone(),
        right_shader_name: right.shader_name.clone(),
        right_shader_slug: right.shader_slug.clone(),
        right_shader_author: right.shader_author.clone(),
        right_shader_version: right.shader_version.clone(),
        right_scene_name: right.scene_name.clone(),
    }
}
