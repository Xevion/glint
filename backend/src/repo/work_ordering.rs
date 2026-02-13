//! Post-processing for work items: spatial clustering and execution ordering.
//!
//! The SQL layer returns work items grouped by world → shader popularity → profile.
//! This module refines the scene ordering within each profile pass using spatial
//! clustering and nearest-neighbor traversal to minimize expensive transitions.
//!
//! **Execution cost hierarchy** (most to least expensive):
//! 1. World load (~10-30s)
//! 2. Shader load ≈ Profile switch (~2-5s)
//! 3. Scene teleport (~1-3s)
//! 4. Derived scene transition (~0.1s) — same position, different time/weather

use crate::id::{SceneId, ShaderVersionId, ShaderVersionProfileId, WorldId};
use crate::models::WorkItem;

/// Scenes within this XZ distance are "derived" — same spot, different time/weather.
/// No teleport needed, just gamerule changes.
const DERIVED_DISTANCE_THRESHOLD: f64 = 2.0;

/// Scenes within this XZ distance share loaded chunks, making teleport faster.
/// 512 blocks ≈ 32 chunks of overlap.
const PROXIMITY_DISTANCE_THRESHOLD: f64 = 512.0;

// ---------------------------------------------------------------------------
// Intermediate grouping types (not exported — internal to ordering logic)
// ---------------------------------------------------------------------------

/// Items for a single (shader_version, profile) within one world.
struct ProfileGroup {
    profile_id: Option<ShaderVersionProfileId>,
    items: Vec<WorkItem>,
}

/// Items for a single shader_version within one world.
struct ShaderGroup {
    shader_version_id: ShaderVersionId,
    profiles: Vec<ProfileGroup>,
}

/// Items for a single world.
struct WorldGroup {
    world_id: WorldId,
    shaders: Vec<ShaderGroup>,
}

/// A scene and its position, with all work items for that scene.
struct ScenePosition {
    scene_id: SceneId,
    x: f64,
    z: f64,
    items: Vec<WorkItem>,
}

/// A cluster of scenes that are spatially close enough to avoid full teleports.
struct SceneCluster {
    /// Centroid position for nearest-neighbor selection between clusters.
    centroid_x: f64,
    centroid_z: f64,
    /// Derived sub-clusters within this proximity cluster. Each sub-cluster
    /// contains scenes at the same position (< 2 blocks apart), sorted by
    /// `time_of_day_ticks` ASC. Transitioning within a sub-cluster costs ~0.1s
    /// (just time/weather change). Transitioning between sub-clusters costs
    /// ~1-3s (teleport within shared chunks).
    derived_subclusters: Vec<Vec<WorkItem>>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Reorder work items for optimal capture execution.
///
/// Assumes the input is already sorted by world → shader popularity → profile
/// from the SQL layer. Refines scene ordering within each profile pass using
/// spatial clustering and nearest-neighbor traversal.
pub fn optimize_execution_order(items: Vec<WorkItem>) -> Vec<WorkItem> {
    if items.len() <= 1 {
        return items;
    }

    let world_groups = group_into_hierarchy(items);
    flatten_to_execution_order(world_groups)
}

// ---------------------------------------------------------------------------
// Step 1: Group into hierarchy
// ---------------------------------------------------------------------------

/// Group flat work items into World → Shader → Profile hierarchy.
///
/// Preserves the SQL ordering within each level (world order, shader popularity
/// order, profile sort order).
fn group_into_hierarchy(items: Vec<WorkItem>) -> Vec<WorldGroup> {
    let mut worlds: Vec<WorldGroup> = Vec::new();

    for item in items {
        // Find or create WorldGroup
        let world = match worlds.last_mut() {
            Some(w) if w.world_id == item.world_id => w,
            _ => {
                worlds.push(WorldGroup {
                    world_id: item.world_id.clone(),
                    shaders: Vec::new(),
                });
                worlds.last_mut().unwrap()
            }
        };

        // Find or create ShaderGroup within world
        let shader = match world.shaders.last_mut() {
            Some(s) if s.shader_version_id == item.shader_version_id => s,
            _ => {
                world.shaders.push(ShaderGroup {
                    shader_version_id: item.shader_version_id.clone(),
                    profiles: Vec::new(),
                });
                world.shaders.last_mut().unwrap()
            }
        };

        // Find or create ProfileGroup within shader
        let profile = match shader.profiles.last_mut() {
            Some(p) if p.profile_id == item.profile_id => p,
            _ => {
                shader.profiles.push(ProfileGroup {
                    profile_id: item.profile_id.clone(),
                    items: Vec::new(),
                });
                shader.profiles.last_mut().unwrap()
            }
        };

        profile.items.push(item);
    }

    worlds
}

// ---------------------------------------------------------------------------
// Step 2: Scene clustering
// ---------------------------------------------------------------------------

/// Euclidean distance in the XZ plane (Y is height, irrelevant for chunk overlap).
fn xz_distance(x1: f64, z1: f64, x2: f64, z2: f64) -> f64 {
    ((x1 - x2).powi(2) + (z1 - z2).powi(2)).sqrt()
}

/// Cluster work items by spatial proximity.
///
/// Two tiers:
/// - **Derived** (< 2 blocks): same position, different time/weather. Grouped
///   together and ordered by `time_of_day_ticks` ASC within each derived group.
/// - **Proximity** (< 512 blocks): share loaded chunks. Grouped into the same
///   cluster for nearest-neighbor traversal.
fn cluster_scenes(items: Vec<WorkItem>) -> Vec<SceneCluster> {
    if items.is_empty() {
        return Vec::new();
    }

    // First, group items by scene_id so we have stable per-scene positions
    let mut scene_groups: Vec<ScenePosition> = Vec::new();
    for item in items {
        match scene_groups
            .iter()
            .position(|sp| sp.scene_id == item.scene_id)
        {
            Some(idx) => scene_groups[idx].items.push(item),
            None => {
                let sp = ScenePosition {
                    scene_id: item.scene_id.clone(),
                    x: item.scene_x,
                    z: item.scene_z,
                    items: vec![item],
                };
                scene_groups.push(sp);
            }
        }
    }

    // Build derived groups: scenes within DERIVED_DISTANCE_THRESHOLD of each other
    let mut derived_groups: Vec<Vec<ScenePosition>> = Vec::new();
    for scene in scene_groups {
        let placed = derived_groups.iter().position(|group| {
            group
                .iter()
                .any(|sp| xz_distance(sp.x, sp.z, scene.x, scene.z) < DERIVED_DISTANCE_THRESHOLD)
        });
        match placed {
            Some(idx) => derived_groups[idx].push(scene),
            None => derived_groups.push(vec![scene]),
        }
    }

    // Build proximity clusters: derived groups within PROXIMITY_DISTANCE_THRESHOLD
    let mut clusters: Vec<SceneCluster> = Vec::new();
    for derived_group in derived_groups {
        let group_centroid = centroid_of_scenes(&derived_group);

        let placed = clusters.iter().position(|c| {
            xz_distance(
                c.centroid_x,
                c.centroid_z,
                group_centroid.0,
                group_centroid.1,
            ) < PROXIMITY_DISTANCE_THRESHOLD
        });

        // Flatten all scenes in this derived group into one list sorted by
        // time_of_day_ticks. These scenes are at the same position, so
        // transitioning between them is just a time/weather change (~0.1s).
        let mut derived_items: Vec<WorkItem> =
            derived_group.into_iter().flat_map(|sp| sp.items).collect();
        derived_items.sort_by_key(|i| i.scene_time_of_day_ticks);

        match placed {
            Some(idx) => {
                let cluster = &mut clusters[idx];
                cluster.derived_subclusters.push(derived_items);
                recompute_centroid(cluster);
            }
            None => {
                clusters.push(SceneCluster {
                    centroid_x: group_centroid.0,
                    centroid_z: group_centroid.1,
                    derived_subclusters: vec![derived_items],
                });
            }
        }
    }

    clusters
}

/// Compute centroid of a set of scene positions.
fn centroid_of_scenes(scenes: &[ScenePosition]) -> (f64, f64) {
    if scenes.is_empty() {
        return (0.0, 0.0);
    }
    let sum_x: f64 = scenes.iter().map(|sp| sp.x).sum();
    let sum_z: f64 = scenes.iter().map(|sp| sp.z).sum();
    let n = scenes.len() as f64;
    (sum_x / n, sum_z / n)
}

/// Recompute cluster centroid from all items in its derived subclusters.
fn recompute_centroid(cluster: &mut SceneCluster) {
    let mut sum_x = 0.0;
    let mut sum_z = 0.0;
    let mut count = 0usize;
    for subcluster in &cluster.derived_subclusters {
        for item in subcluster {
            sum_x += item.scene_x;
            sum_z += item.scene_z;
            count += 1;
        }
    }
    if count > 0 {
        cluster.centroid_x = sum_x / count as f64;
        cluster.centroid_z = sum_z / count as f64;
    }
}

// ---------------------------------------------------------------------------
// Step 3: Nearest-neighbor scene ordering
// ---------------------------------------------------------------------------

/// Order scene clusters using greedy nearest-neighbor traversal.
///
/// Starts from the largest cluster (most items → most time saved by visiting
/// early when chunks are fresh) and greedily picks the nearest unvisited
/// cluster by centroid distance.
fn order_clusters_nearest_neighbor(mut clusters: Vec<SceneCluster>) -> Vec<SceneCluster> {
    if clusters.len() <= 1 {
        return clusters;
    }

    let mut ordered = Vec::with_capacity(clusters.len());

    // Start with the largest cluster
    let start_idx = clusters
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.derived_subclusters.iter().map(|g| g.len()).sum::<usize>())
        .map(|(i, _)| i)
        .unwrap_or(0);

    let current = clusters.swap_remove(start_idx);
    let mut current_x = current.centroid_x;
    let mut current_z = current.centroid_z;
    ordered.push(current);

    while !clusters.is_empty() {
        let nearest_idx = clusters
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = xz_distance(current_x, current_z, a.centroid_x, a.centroid_z);
                let dist_b = xz_distance(current_x, current_z, b.centroid_x, b.centroid_z);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap();

        let next = clusters.swap_remove(nearest_idx);
        current_x = next.centroid_x;
        current_z = next.centroid_z;
        ordered.push(next);
    }

    ordered
}

// ---------------------------------------------------------------------------
// Step 4: Flatten back to Vec<WorkItem>
// ---------------------------------------------------------------------------

/// Flatten the grouped hierarchy back into a flat list in execution order.
fn flatten_to_execution_order(worlds: Vec<WorldGroup>) -> Vec<WorkItem> {
    let mut result = Vec::new();

    for world in worlds {
        for shader in world.shaders {
            for profile in shader.profiles {
                let clusters = cluster_scenes(profile.items);
                let ordered_clusters = order_clusters_nearest_neighbor(clusters);

                for cluster in ordered_clusters {
                    for subcluster in cluster.derived_subclusters {
                        result.extend(subcluster);
                    }
                }
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{SceneId, ShaderId, ShaderVersionId, ShaderVersionProfileId, WorldId};

    /// Create a minimal WorkItem for testing. Only fields relevant to ordering
    /// are set meaningfully; the rest use placeholder values.
    fn make_item(
        world_id: &str,
        shader_version_id: &str,
        profile_id: Option<&str>,
        scene_id: &str,
        x: f64,
        z: f64,
        time_of_day: i32,
    ) -> WorkItem {
        WorkItem {
            shader_version_id: ShaderVersionId(shader_version_id.to_string()),
            shader_id: ShaderId("sh1".to_string()),
            shader_slug: "test-shader".to_string(),
            shader_name: "Test Shader".to_string(),
            version: "1.0".to_string(),
            download_url: None,
            file_hash: None,
            scene_id: SceneId(scene_id.to_string()),
            scene_slug: scene_id.to_string(),
            scene_name: scene_id.to_string(),
            scene_dimension: "overworld".to_string(),
            scene_x: x,
            scene_y: 64.0,
            scene_z: z,
            scene_yaw: 0.0,
            scene_pitch: 0.0,
            scene_time_of_day_ticks: time_of_day,
            scene_weather: "clear".to_string(),
            scene_weather_intensity: 0.0,
            scene_moon_phase: None,
            scene_biome: None,
            world_id: WorldId(world_id.to_string()),
            world_slug: world_id.to_string(),
            world_name: world_id.to_string(),
            world_file_url: None,
            world_file_hash: None,
            world_size_bytes: None,
            world_version_id: None,
            scene_version_id: None,
            profile_id: profile_id.map(|p| ShaderVersionProfileId(p.to_string())),
            profile_name: profile_id.map(|p| p.to_string()),
        }
    }

    /// Extract scene_id strings from items for easy assertion.
    fn scene_ids(items: &[WorkItem]) -> Vec<&str> {
        items.iter().map(|i| i.scene_id.0.as_str()).collect()
    }

    /// Extract (world, shader, profile, scene) tuples for full ordering assertions.
    fn item_keys(items: &[WorkItem]) -> Vec<(&str, &str, Option<&str>, &str)> {
        items
            .iter()
            .map(|i| {
                (
                    i.world_id.0.as_str(),
                    i.shader_version_id.0.as_str(),
                    i.profile_id.as_ref().map(|p| p.0.as_str()),
                    i.scene_id.0.as_str(),
                )
            })
            .collect()
    }

    #[test]
    fn test_empty_input() {
        let result = optimize_execution_order(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_item_passthrough() {
        let items = vec![make_item("w1", "sv1", None, "s1", 0.0, 0.0, 6000)];
        let result = optimize_execution_order(items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scene_id.0, "s1");
    }

    #[test]
    fn test_derived_scenes_clustered_by_time() {
        // Three scenes at the same position (distance=0), different time_of_day.
        // Should be ordered by time_of_day ASC.
        let items = vec![
            make_item("w1", "sv1", None, "hills_night", 100.0, 200.0, 13000),
            make_item("w1", "sv1", None, "hills_sunset", 100.0, 200.0, 18000),
            make_item("w1", "sv1", None, "hills", 100.0, 200.0, 1500),
        ];

        let result = optimize_execution_order(items);
        assert_eq!(
            scene_ids(&result),
            vec!["hills", "hills_night", "hills_sunset"]
        );
    }

    #[test]
    fn test_hierarchy_preserved() {
        // World order and shader/profile grouping should be maintained.
        let items = vec![
            make_item("w1", "sv1", Some("low"), "s1", 0.0, 0.0, 6000),
            make_item("w1", "sv1", Some("low"), "s2", 100.0, 100.0, 6000),
            make_item("w1", "sv1", Some("high"), "s1", 0.0, 0.0, 6000),
            make_item("w1", "sv1", Some("high"), "s2", 100.0, 100.0, 6000),
            make_item("w2", "sv2", None, "s3", 50.0, 50.0, 6000),
        ];

        let result = optimize_execution_order(items);
        let keys = item_keys(&result);

        // w1 items come before w2 items
        let w1_end = keys.iter().rposition(|(w, _, _, _)| *w == "w1").unwrap();
        let w2_start = keys.iter().position(|(w, _, _, _)| *w == "w2").unwrap();
        assert!(w1_end < w2_start, "world grouping broken");

        // Within w1/sv1, "low" profiles come before "high" profiles
        let low_end = keys
            .iter()
            .rposition(|(_, _, p, _)| *p == Some("low"))
            .unwrap();
        let high_start = keys
            .iter()
            .position(|(_, _, p, _)| *p == Some("high"))
            .unwrap();
        assert!(low_end < high_start, "profile grouping broken");
    }

    #[test]
    fn test_proximity_cluster_ordering() {
        // Three scene groups:
        // - A at (0, 0) and B at (100, 0): within 512 blocks → same proximity cluster
        // - C at (2000, 0): far away → separate cluster
        // Nearest-neighbor from the larger cluster (A+B) should visit C last.
        let items = vec![
            make_item("w1", "sv1", None, "a", 0.0, 0.0, 6000),
            make_item("w1", "sv1", None, "c", 2000.0, 0.0, 6000),
            make_item("w1", "sv1", None, "b", 100.0, 0.0, 6000),
        ];

        let result = optimize_execution_order(items);
        let ids = scene_ids(&result);

        // A and B should be adjacent (same proximity cluster), C separate
        let a_pos = ids.iter().position(|s| *s == "a").unwrap();
        let b_pos = ids.iter().position(|s| *s == "b").unwrap();
        let c_pos = ids.iter().position(|s| *s == "c").unwrap();

        // a and b should be next to each other
        assert!(
            (a_pos as i64 - b_pos as i64).abs() == 1,
            "proximity scenes a and b should be adjacent, got a={a_pos}, b={b_pos}"
        );
        // c should come after both a and b (it's the distant cluster)
        assert!(
            c_pos > a_pos && c_pos > b_pos,
            "distant scene c should come last"
        );
    }

    #[test]
    fn test_profile_scene_ordering_repeated() {
        // Each profile should get the same scene ordering independently.
        // Two profiles, three derived scenes each.
        let items = vec![
            make_item("w1", "sv1", Some("low"), "hills_night", 100.0, 200.0, 13000),
            make_item("w1", "sv1", Some("low"), "hills", 100.0, 200.0, 1500),
            make_item(
                "w1",
                "sv1",
                Some("low"),
                "hills_sunset",
                100.0,
                200.0,
                18000,
            ),
            make_item(
                "w1",
                "sv1",
                Some("high"),
                "hills_night",
                100.0,
                200.0,
                13000,
            ),
            make_item("w1", "sv1", Some("high"), "hills", 100.0, 200.0, 1500),
            make_item(
                "w1",
                "sv1",
                Some("high"),
                "hills_sunset",
                100.0,
                200.0,
                18000,
            ),
        ];

        let result = optimize_execution_order(items);

        // First 3 should be low profile, next 3 high profile
        let low_scenes: Vec<&str> = result[..3].iter().map(|i| i.scene_id.0.as_str()).collect();
        let high_scenes: Vec<&str> = result[3..].iter().map(|i| i.scene_id.0.as_str()).collect();

        assert_eq!(low_scenes, vec!["hills", "hills_night", "hills_sunset"]);
        assert_eq!(high_scenes, vec!["hills", "hills_night", "hills_sunset"]);
    }

    #[test]
    fn test_xz_distance_ignores_y() {
        // Distance should only consider X and Z, not Y
        let d = xz_distance(0.0, 0.0, 3.0, 4.0);
        assert!((d - 5.0).abs() < 0.001, "expected 5.0, got {d}");
    }

    #[test]
    fn test_derived_threshold_boundary() {
        // Scenes at exactly 1.9 blocks apart should be in the same derived subcluster
        let items = vec![
            make_item("w1", "sv1", None, "s1", 0.0, 0.0, 6000),
            make_item("w1", "sv1", None, "s2", 1.9, 0.0, 12000),
        ];

        let clusters = cluster_scenes(items);
        assert_eq!(clusters.len(), 1, "should be one proximity cluster");
        // Both scenes within 2 blocks → one derived subcluster, time-sorted
        assert_eq!(
            clusters[0].derived_subclusters.len(),
            1,
            "should be one derived subcluster (both scenes within 2 blocks)"
        );
        assert_eq!(clusters[0].derived_subclusters[0].len(), 2);
        assert_eq!(
            clusters[0].derived_subclusters[0][0].scene_time_of_day_ticks,
            6000
        );
        assert_eq!(
            clusters[0].derived_subclusters[0][1].scene_time_of_day_ticks,
            12000
        );
    }

    #[test]
    fn test_derived_threshold_exceeded() {
        // Scenes at 3.0 blocks apart should be in separate derived subclusters
        // but same proximity cluster (< 512 blocks)
        let items = vec![
            make_item("w1", "sv1", None, "s1", 0.0, 0.0, 6000),
            make_item("w1", "sv1", None, "s2", 3.0, 0.0, 12000),
        ];

        let clusters = cluster_scenes(items);
        assert_eq!(clusters.len(), 1, "should be one proximity cluster");
        assert_eq!(
            clusters[0].derived_subclusters.len(),
            2,
            "should be two derived subclusters (scenes > 2 blocks apart)"
        );
    }

    #[test]
    fn test_real_world_glint_scenario() {
        // Simulate the actual Glint data: Hills/Hills Night/Hills Sunset at same
        // position, Mountain Lake nearby (21 shared chunks), Mesa far away.
        let items = vec![
            make_item("glint", "sv1", None, "hills", 100.0, 200.0, 1500),
            make_item("glint", "sv1", None, "hills_night", 100.0, 200.0, 13000),
            make_item("glint", "sv1", None, "hills_sunset", 100.0, 200.0, 18000),
            make_item("glint", "sv1", None, "mountain_lake", 250.0, 150.0, 6000),
            make_item("glint", "sv1", None, "mesa", 5000.0, 3000.0, 6000),
        ];

        let result = optimize_execution_order(items);
        let ids = scene_ids(&result);

        // Hills cluster should be contiguous and time-ordered
        let hills_idx = ids.iter().position(|s| *s == "hills").unwrap();
        let night_idx = ids.iter().position(|s| *s == "hills_night").unwrap();
        let sunset_idx = ids.iter().position(|s| *s == "hills_sunset").unwrap();
        assert_eq!(night_idx, hills_idx + 1);
        assert_eq!(sunset_idx, hills_idx + 2);

        // Mountain lake should be closer to hills than mesa
        let lake_idx = ids.iter().position(|s| *s == "mountain_lake").unwrap();
        let mesa_idx = ids.iter().position(|s| *s == "mesa").unwrap();
        // The proximity cluster (hills + mountain_lake) should come before mesa
        assert!(lake_idx < mesa_idx, "mountain_lake should come before mesa");
    }
}
