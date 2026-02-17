use anyhow::Context;
use sqlx::Row;

use crate::db::DbPool;

pub struct StorageRepo;

/// A reference from a database row to an R2 URL.
pub struct DbReference {
    pub table: &'static str,
    pub row_id: String,
    pub url: String,
}

impl StorageRepo {
    /// Collect all R2 URLs/keys referenced by any DB table.
    pub async fn all_referenced_urls(pool: &DbPool) -> anyhow::Result<Vec<DbReference>> {
        let mut refs = Vec::new();

        // Captures
        let captures =
            sqlx::query("SELECT id, image_url FROM captures WHERE image_url IS NOT NULL")
                .fetch_all(pool)
                .await
                .context("Failed to query capture URLs")?;

        for row in captures {
            let id: String = row.get("id");
            let url: String = row.get("image_url");
            refs.push(DbReference {
                table: "captures",
                row_id: id,
                url,
            });
        }

        // Scene versions (package_url)
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
                url,
            });
        }

        // Backgrounds
        let backgrounds = sqlx::query("SELECT id, image_url FROM backgrounds")
            .fetch_all(pool)
            .await
            .context("Failed to query background URLs")?;

        for row in backgrounds {
            let id: String = row.get("id");
            let url: String = row.get("image_url");
            refs.push(DbReference {
                table: "backgrounds",
                row_id: id,
                url,
            });
        }

        Ok(refs)
    }
}
