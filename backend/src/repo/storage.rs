use anyhow::Context;
use sqlx::Row;

use crate::db::DbPool;

pub struct StorageRepo;

/// A reference from a database row to an R2 URL or key.
pub struct DbReference {
    pub table: &'static str,
    pub row_id: String,
    /// For URL-based columns, this is the full URL.
    /// For `pending_uploads.upload_key`, this is the raw R2 key.
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

        // World versions
        let versions =
            sqlx::query("SELECT id, file_url FROM world_versions WHERE file_url IS NOT NULL")
                .fetch_all(pool)
                .await
                .context("Failed to query world version URLs")?;

        for row in versions {
            let id: String = row.get("id");
            let url: String = row.get("file_url");
            refs.push(DbReference {
                table: "world_versions",
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

        // Pending uploads (upload_key is already a key, not a URL)
        let pending = sqlx::query("SELECT upload_id, upload_key FROM pending_uploads")
            .fetch_all(pool)
            .await
            .context("Failed to query pending uploads")?;

        for row in pending {
            let id: String = row.get("upload_id");
            let key: String = row.get("upload_key");
            refs.push(DbReference {
                table: "pending_uploads",
                row_id: id,
                url: key,
            });
        }

        Ok(refs)
    }
}
