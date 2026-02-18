use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::Background;

pub struct BackgroundRepo;

impl BackgroundRepo {
    /// List enabled backgrounds ordered by sort_order (public endpoint).
    pub async fn list_enabled(db: &DbPool) -> AppResult<Vec<Background>> {
        let rows = sqlx::query_as::<_, Background>(
            r#"
            SELECT id, image_path, thumbhash, theme_mode, enabled,
                   sort_order, original_filename, width, height,
                   file_size_bytes, content_type, created_at, updated_at
            FROM backgrounds
            WHERE enabled = true
            ORDER BY sort_order, created_at
            "#,
        )
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    /// List all backgrounds including disabled (admin).
    pub async fn list_all(db: &DbPool) -> AppResult<Vec<Background>> {
        let rows = sqlx::query_as::<_, Background>(
            r#"
            SELECT id, image_path, thumbhash, theme_mode, enabled,
                   sort_order, original_filename, width, height,
                   file_size_bytes, content_type, created_at, updated_at
            FROM backgrounds
            ORDER BY sort_order, created_at
            "#,
        )
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    /// Get a single background by ID.
    pub async fn get_by_id(db: &DbPool, id: &str) -> AppResult<Option<Background>> {
        let row = sqlx::query_as::<_, Background>(
            r#"
            SELECT id, image_path, thumbhash, theme_mode, enabled,
                   sort_order, original_filename, width, height,
                   file_size_bytes, content_type, created_at, updated_at
            FROM backgrounds
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    /// Insert a new background (during upload initiation).
    pub async fn insert(
        db: &DbPool,
        id: &str,
        image_path: &str,
        original_filename: Option<&str>,
    ) -> AppResult<Background> {
        let row = sqlx::query_as::<_, Background>(
            r#"
            INSERT INTO backgrounds (id, image_path, original_filename)
            VALUES ($1, $2, $3)
            RETURNING id, image_path, thumbhash, theme_mode, enabled,
                      sort_order, original_filename, width, height,
                      file_size_bytes, content_type, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(image_path)
        .bind(original_filename)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    /// Confirm upload by filling in metadata.
    pub async fn confirm_upload(
        db: &DbPool,
        id: &str,
        thumbhash: Option<&str>,
        width: Option<i32>,
        height: Option<i32>,
        file_size_bytes: Option<i64>,
        content_type: Option<&str>,
    ) -> AppResult<Background> {
        let row = sqlx::query_as::<_, Background>(
            r#"
            UPDATE backgrounds
            SET thumbhash = COALESCE($2, thumbhash),
                width = COALESCE($3, width),
                height = COALESCE($4, height),
                file_size_bytes = COALESCE($5, file_size_bytes),
                content_type = COALESCE($6, content_type),
                updated_at = now()
            WHERE id = $1
            RETURNING id, image_path, thumbhash, theme_mode, enabled,
                      sort_order, original_filename, width, height,
                      file_size_bytes, content_type, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(thumbhash)
        .bind(width)
        .bind(height)
        .bind(file_size_bytes)
        .bind(content_type)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    /// Update metadata (enabled, theme_mode, sort_order).
    pub async fn update(
        db: &DbPool,
        id: &str,
        enabled: Option<bool>,
        theme_mode: Option<&str>,
        sort_order: Option<i32>,
    ) -> AppResult<Background> {
        let row = sqlx::query_as::<_, Background>(
            r#"
            UPDATE backgrounds
            SET enabled = COALESCE($2, enabled),
                theme_mode = COALESCE($3, theme_mode),
                sort_order = COALESCE($4, sort_order),
                updated_at = now()
            WHERE id = $1
            RETURNING id, image_path, thumbhash, theme_mode, enabled,
                      sort_order, original_filename, width, height,
                      file_size_bytes, content_type, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(enabled)
        .bind(theme_mode)
        .bind(sort_order)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    /// Delete a background by ID. Returns the image_path for R2 cleanup.
    pub async fn delete(db: &DbPool, id: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            DELETE FROM backgrounds WHERE id = $1
            RETURNING image_path
            "#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?;
        Ok(row.map(|(path,)| path))
    }
}
