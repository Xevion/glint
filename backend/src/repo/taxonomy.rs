use anyhow::Context;
use tracing::{debug, instrument};

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Category, Feature, Tag};

pub struct CategoryRepo;

impl CategoryRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: i32) -> AppResult<Option<Category>> {
        sqlx::query_as!(Category, "SELECT * FROM categories WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find category '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: i32) -> AppResult<Category> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Category '{}' not found", id)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<Category>> {
        sqlx::query_as!(Category, "SELECT * FROM categories WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to find category by slug '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn list(db: &DbPool) -> AppResult<Vec<Category>> {
        let categories = sqlx::query_as!(Category, "SELECT * FROM categories ORDER BY name")
            .fetch_all(db)
            .await
            .context("failed to list categories")?;

        debug!(count = categories.len(), "Listed categories");
        Ok(categories)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn create(
        db: &DbPool,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<Category> {
        let result = sqlx::query!(
            r#"
            INSERT INTO categories (slug, name, description)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            slug,
            name,
            description
        )
        .fetch_one(db)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result
            && db_err.code().as_deref() == Some("23505")
        {
            return Err(AppError::Conflict(format!(
                "Category with slug '{}' already exists",
                slug
            )));
        }

        let row = result.context(format!("failed to create category '{}'", slug))?;
        Self::get_by_id(db, row.id).await
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: i32) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM categories WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete category '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// List categories for a shader
    #[instrument(skip(db), level = "debug")]
    pub async fn list_for_shader(db: &DbPool, shader_id: &str) -> AppResult<Vec<Category>> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.* FROM categories c
            JOIN shader_categories sc ON sc.category_id = c.id
            WHERE sc.shader_id = $1
            ORDER BY c.name
            "#,
            shader_id
        )
        .fetch_all(db)
        .await
        .context(format!(
            "failed to list categories for shader '{}'",
            shader_id
        ))?;

        Ok(categories)
    }

    /// Add category to shader
    #[instrument(skip(db), level = "debug")]
    pub async fn add_to_shader(db: &DbPool, shader_id: &str, category_id: i32) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO shader_categories (shader_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            shader_id,
            category_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to add category '{}' to shader '{}'",
            category_id, shader_id
        ))?;

        Ok(())
    }

    /// Remove category from shader
    #[instrument(skip(db), level = "debug")]
    pub async fn remove_from_shader(
        db: &DbPool,
        shader_id: &str,
        category_id: i32,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM shader_categories WHERE shader_id = $1 AND category_id = $2",
            shader_id,
            category_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to remove category '{}' from shader '{}'",
            category_id, shader_id
        ))?;

        Ok(result.rows_affected() > 0)
    }
}

pub struct FeatureRepo;

impl FeatureRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: i32) -> AppResult<Option<Feature>> {
        sqlx::query_as!(Feature, "SELECT * FROM features WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find feature '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: i32) -> AppResult<Feature> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Feature '{}' not found", id)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<Feature>> {
        sqlx::query_as!(Feature, "SELECT * FROM features WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to find feature by slug '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn list(db: &DbPool) -> AppResult<Vec<Feature>> {
        let features = sqlx::query_as!(Feature, "SELECT * FROM features ORDER BY name")
            .fetch_all(db)
            .await
            .context("failed to list features")?;

        debug!(count = features.len(), "Listed features");
        Ok(features)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn create(
        db: &DbPool,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<Feature> {
        let result = sqlx::query!(
            r#"
            INSERT INTO features (slug, name, description)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            slug,
            name,
            description
        )
        .fetch_one(db)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result
            && db_err.code().as_deref() == Some("23505")
        {
            return Err(AppError::Conflict(format!(
                "Feature with slug '{}' already exists",
                slug
            )));
        }

        let row = result.context(format!("failed to create feature '{}'", slug))?;
        Self::get_by_id(db, row.id).await
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: i32) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM features WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete feature '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// List features for a shader
    #[instrument(skip(db), level = "debug")]
    pub async fn list_for_shader(db: &DbPool, shader_id: &str) -> AppResult<Vec<Feature>> {
        let features = sqlx::query_as!(
            Feature,
            r#"
            SELECT f.* FROM features f
            JOIN shader_features sf ON sf.feature_id = f.id
            WHERE sf.shader_id = $1
            ORDER BY f.name
            "#,
            shader_id
        )
        .fetch_all(db)
        .await
        .context(format!(
            "failed to list features for shader '{}'",
            shader_id
        ))?;

        Ok(features)
    }

    /// Add feature to shader
    #[instrument(skip(db), level = "debug")]
    pub async fn add_to_shader(db: &DbPool, shader_id: &str, feature_id: i32) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO shader_features (shader_id, feature_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            shader_id,
            feature_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to add feature '{}' to shader '{}'",
            feature_id, shader_id
        ))?;

        Ok(())
    }

    /// Remove feature from shader
    #[instrument(skip(db), level = "debug")]
    pub async fn remove_from_shader(
        db: &DbPool,
        shader_id: &str,
        feature_id: i32,
    ) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM shader_features WHERE shader_id = $1 AND feature_id = $2",
            shader_id,
            feature_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to remove feature '{}' from shader '{}'",
            feature_id, shader_id
        ))?;

        Ok(result.rows_affected() > 0)
    }
}

pub struct TagRepo;

impl TagRepo {
    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_id(db: &DbPool, id: i32) -> AppResult<Option<Tag>> {
        sqlx::query_as!(Tag, "SELECT * FROM tags WHERE id = $1", id)
            .fetch_optional(db)
            .await
            .context(format!("failed to find tag '{}'", id))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn get_by_id(db: &DbPool, id: i32) -> AppResult<Tag> {
        Self::find_by_id(db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Tag '{}' not found", id)))
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn find_by_slug(db: &DbPool, slug: &str) -> AppResult<Option<Tag>> {
        sqlx::query_as!(Tag, "SELECT * FROM tags WHERE slug = $1", slug)
            .fetch_optional(db)
            .await
            .context(format!("failed to find tag by slug '{}'", slug))
            .map_err(Into::into)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn list(db: &DbPool) -> AppResult<Vec<Tag>> {
        let tags = sqlx::query_as!(Tag, "SELECT * FROM tags ORDER BY name")
            .fetch_all(db)
            .await
            .context("failed to list tags")?;

        debug!(count = tags.len(), "Listed tags");
        Ok(tags)
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn create(
        db: &DbPool,
        slug: &str,
        name: &str,
        description: Option<&str>,
    ) -> AppResult<Tag> {
        let result = sqlx::query!(
            r#"
            INSERT INTO tags (slug, name, description)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
            slug,
            name,
            description
        )
        .fetch_one(db)
        .await;

        if let Err(sqlx::Error::Database(db_err)) = &result
            && db_err.code().as_deref() == Some("23505")
        {
            return Err(AppError::Conflict(format!(
                "Tag with slug '{}' already exists",
                slug
            )));
        }

        let row = result.context(format!("failed to create tag '{}'", slug))?;
        Self::get_by_id(db, row.id).await
    }

    #[instrument(skip(db), level = "debug")]
    pub async fn delete(db: &DbPool, id: i32) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM tags WHERE id = $1", id)
            .execute(db)
            .await
            .context(format!("failed to delete tag '{}'", id))?;

        Ok(result.rows_affected() > 0)
    }

    /// List tags for a scene
    #[instrument(skip(db), level = "debug")]
    pub async fn list_for_scene(db: &DbPool, scene_id: &str) -> AppResult<Vec<Tag>> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT t.* FROM tags t
            JOIN scene_tags st ON st.tag_id = t.id
            WHERE st.scene_id = $1
            ORDER BY t.name
            "#,
            scene_id
        )
        .fetch_all(db)
        .await
        .context(format!("failed to list tags for scene '{}'", scene_id))?;

        Ok(tags)
    }

    /// Add tag to scene
    #[instrument(skip(db), level = "debug")]
    pub async fn add_to_scene(db: &DbPool, scene_id: &str, tag_id: i32) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO scene_tags (scene_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            scene_id,
            tag_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to add tag '{}' to scene '{}'",
            tag_id, scene_id
        ))?;

        Ok(())
    }

    /// Remove tag from scene
    #[instrument(skip(db), level = "debug")]
    pub async fn remove_from_scene(db: &DbPool, scene_id: &str, tag_id: i32) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM scene_tags WHERE scene_id = $1 AND tag_id = $2",
            scene_id,
            tag_id
        )
        .execute(db)
        .await
        .context(format!(
            "failed to remove tag '{}' from scene '{}'",
            tag_id, scene_id
        ))?;

        Ok(result.rows_affected() > 0)
    }
}
