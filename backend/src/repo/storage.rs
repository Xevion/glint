use anyhow::Context;
use sqlx::Row;

use crate::db::DbPool;

pub struct StorageRepo;

/// A reference from a database row to an R2 object.
/// The `reference` field is either an S3 key (for captures, backgrounds)
/// or a full URL (for scene_versions.package_url).
pub struct DbReference {
    pub table: &'static str,
    pub row_id: String,
    pub reference: String,
}

impl StorageRepo {
    /// Collect all R2 references from every DB table that stores objects.
    pub async fn all_references(pool: &DbPool) -> anyhow::Result<Vec<DbReference>> {
        let mut refs = Vec::new();

        // Captures (image_path = S3 key)
        let captures =
            sqlx::query("SELECT id, image_path FROM captures WHERE image_path IS NOT NULL")
                .fetch_all(pool)
                .await
                .context("Failed to query capture paths")?;

        for row in captures {
            let id: String = row.get("id");
            let path: String = row.get("image_path");
            refs.push(DbReference {
                table: "captures",
                row_id: id,
                reference: path,
            });
        }

        // Scene versions (package_url = full URL, needs key_from_url conversion)
        let versions =
            sqlx::query("SELECT id, package_url FROM scene_versions WHERE package_url IS NOT NULL")
                .fetch_all(pool)
                .await
                .context("Failed to query scene version package URLs")?;

        for row in versions {
            let id: String = row.get("id");
            let url: String = row.get("package_url");
            refs.push(DbReference {
                table: "scene_versions",
                row_id: id,
                reference: url,
            });
        }

        // Backgrounds (image_path = S3 key)
        let backgrounds = sqlx::query("SELECT id, image_path FROM backgrounds")
            .fetch_all(pool)
            .await
            .context("Failed to query background paths")?;

        for row in backgrounds {
            let id: String = row.get("id");
            let path: String = row.get("image_path");
            refs.push(DbReference {
                table: "backgrounds",
                row_id: id,
                reference: path,
            });
        }

        Ok(refs)
    }
}
