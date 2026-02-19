use std::collections::HashMap;

use anyhow::Context;
use sqlx::Row;
use tracing::{debug, instrument};

use crate::error::{AppResult, SqlxResultExt};
use crate::models::{Category, Feature, Tag};

/// Generates a full taxonomy repository `impl` block for a `{id, slug, name, description}` table
/// with join-table link/unlink operations against a parent entity.
///
/// Since `macro_rules!` cannot forward string literals into `sqlx::query_as!` (a proc macro),
/// all queries use `sqlx::query_as::<_, Model>(sql)` with compile-time `concat!()` strings.
/// This is runtime-checked rather than compile-time-checked — an acceptable trade-off for
/// trivial CRUD queries that are covered by integration tests.
macro_rules! define_taxonomy_repo {
    (
        $Repo:ident, $Model:ident, $table:literal,
        parent: $parent:literal, join_table: $join_table:literal, fk: $fk:literal
    ) => {
        pub struct $Repo;

        ::paste::paste! {
            impl $Repo {
                #[instrument(skip(executor), level = "debug")]
                pub async fn find_by_id(
                    executor: impl sqlx::PgExecutor<'_>,
                    id: i32,
                ) -> AppResult<Option<$Model>> {
                    sqlx::query_as::<_, $Model>(
                        concat!("SELECT * FROM ", $table, " WHERE id = $1"),
                    )
                    .bind(id)
                    .fetch_optional(executor)
                    .await
                    .context(format!(concat!("failed to find ", $table, " '{}'"), id))
                    .map_err(Into::into)
                }

                #[instrument(skip(executor), level = "debug")]
                pub async fn find_by_slug(
                    executor: impl sqlx::PgExecutor<'_>,
                    slug: &str,
                ) -> AppResult<Option<$Model>> {
                    sqlx::query_as::<_, $Model>(
                        concat!("SELECT * FROM ", $table, " WHERE slug = $1"),
                    )
                    .bind(slug)
                    .fetch_optional(executor)
                    .await
                    .context(format!(
                        concat!("failed to find ", $table, " by slug '{}'"),
                        slug
                    ))
                    .map_err(Into::into)
                }

                #[instrument(skip(executor), level = "debug")]
                pub async fn list(
                    executor: impl sqlx::PgExecutor<'_>,
                ) -> AppResult<Vec<$Model>> {
                    let items = sqlx::query_as::<_, $Model>(
                        concat!("SELECT * FROM ", $table, " ORDER BY name"),
                    )
                    .fetch_all(executor)
                    .await
                    .context(concat!("failed to list ", $table))?;

                    debug!(count = items.len(), concat!("Listed ", $table));
                    Ok(items)
                }

                #[instrument(skip(executor), level = "debug")]
                pub async fn create(
                    executor: impl sqlx::PgExecutor<'_>,
                    slug: &str,
                    name: &str,
                    description: Option<&str>,
                ) -> AppResult<$Model> {
                    let result = sqlx::query_as::<_, $Model>(concat!(
                        "INSERT INTO ", $table, " (slug, name, description) ",
                        "VALUES ($1, $2, $3) RETURNING *"
                    ))
                    .bind(slug)
                    .bind(name)
                    .bind(description)
                    .fetch_one(executor)
                    .await;

                    result.conflict_on_unique(format!(
                        concat!(stringify!($Model), " with slug '{}' already exists"),
                        slug
                    ))
                }

                #[instrument(skip(executor), level = "debug")]
                pub async fn delete(
                    executor: impl sqlx::PgExecutor<'_>,
                    id: i32,
                ) -> AppResult<bool> {
                    let result = sqlx::query(
                        concat!("DELETE FROM ", $table, " WHERE id = $1"),
                    )
                    .bind(id)
                    .execute(executor)
                    .await
                    .context(format!(concat!("failed to delete ", $table, " '{}'"), id))?;

                    Ok(result.rows_affected() > 0)
                }

                /// List all parent-to-taxonomy mappings for batch enrichment.
                #[instrument(skip(executor), level = "debug")]
                pub async fn [<list_all_for_ $parent s>](
                    executor: impl sqlx::PgExecutor<'_>,
                ) -> AppResult<Vec<(String, $Model)>> {
                    let rows = sqlx::query(concat!(
                        "SELECT j.", $parent, "_id, t.id, t.slug, t.name, t.description ",
                        "FROM ", $table, " t ",
                        "JOIN ", $join_table, " j ON j.", $fk, " = t.id ",
                        "ORDER BY t.name"
                    ))
                    .fetch_all(executor)
                    .await
                    .context(concat!(
                        "failed to list all ", $parent, " ", $table
                    ))?;

                    Ok(rows
                        .into_iter()
                        .map(|r| {
                            let parent_id: String = r.get(concat!($parent, "_id"));
                            let model = $Model {
                                id: r.get("id"),
                                slug: r.get("slug"),
                                name: r.get("name"),
                                description: r.get("description"),
                            };
                            (parent_id, model)
                        })
                        .collect())
                }

                /// List taxonomy items for a single parent entity.
                #[instrument(skip(executor), level = "debug")]
                pub async fn [<list_for_ $parent>](
                    executor: impl sqlx::PgExecutor<'_>,
                    [<$parent _id>]: &str,
                ) -> AppResult<Vec<$Model>> {
                    let items = sqlx::query_as::<_, $Model>(concat!(
                        "SELECT t.* FROM ", $table, " t ",
                        "JOIN ", $join_table, " j ON j.", $fk, " = t.id ",
                        "WHERE j.", $parent, "_id = $1 ",
                        "ORDER BY t.name"
                    ))
                    .bind([<$parent _id>])
                    .fetch_all(executor)
                    .await
                    .context(format!(
                        concat!("failed to list ", $table, " for ", $parent, " '{}'"),
                        [<$parent _id>]
                    ))?;

                    Ok(items)
                }

                /// List taxonomy items for multiple parent entities in a single query.
                #[instrument(skip(executor), level = "debug")]
                pub async fn [<list_for_ $parent s>](
                    executor: impl sqlx::PgExecutor<'_>,
                    [<$parent _ids>]: &[String],
                ) -> AppResult<HashMap<String, Vec<$Model>>> {
                    let rows = sqlx::query(concat!(
                        "SELECT j.", $parent, "_id, t.id, t.slug, t.name, t.description ",
                        "FROM ", $table, " t ",
                        "JOIN ", $join_table, " j ON j.", $fk, " = t.id ",
                        "WHERE j.", $parent, "_id = ANY($1) ",
                        "ORDER BY j.", $parent, "_id, t.name"
                    ))
                    .bind([<$parent _ids>])
                    .fetch_all(executor)
                    .await
                    .context(concat!(
                        "failed to list ", $table, " for ", $parent, "s (batch)"
                    ))?;

                    debug!(count = rows.len(), concat!("Listed ", $table, " (batch)"));
                    let mut map: HashMap<String, Vec<$Model>> = HashMap::new();
                    for r in rows {
                        let parent_id: String = r.get(concat!($parent, "_id"));
                        let model = $Model {
                            id: r.get("id"),
                            slug: r.get("slug"),
                            name: r.get("name"),
                            description: r.get("description"),
                        };
                        map.entry(parent_id).or_default().push(model);
                    }
                    Ok(map)
                }

                /// Link a taxonomy item to a parent entity.
                #[instrument(skip(executor), level = "debug")]
                pub async fn [<add_to_ $parent>](
                    executor: impl sqlx::PgExecutor<'_>,
                    [<$parent _id>]: &str,
                    [<$fk:snake>]: i32,
                ) -> AppResult<()> {
                    sqlx::query(concat!(
                        "INSERT INTO ", $join_table,
                        " (", $parent, "_id, ", $fk, ") VALUES ($1, $2) ",
                        "ON CONFLICT DO NOTHING"
                    ))
                    .bind([<$parent _id>])
                    .bind([<$fk:snake>])
                    .execute(executor)
                    .await
                    .context(format!(
                        concat!("failed to add ", $fk, " '{}' to ", $parent, " '{}'"),
                        [<$fk:snake>], [<$parent _id>]
                    ))?;

                    Ok(())
                }

                /// Remove a taxonomy item from a parent entity.
                #[instrument(skip(executor), level = "debug")]
                pub async fn [<remove_from_ $parent>](
                    executor: impl sqlx::PgExecutor<'_>,
                    [<$parent _id>]: &str,
                    [<$fk:snake>]: i32,
                ) -> AppResult<bool> {
                    let result = sqlx::query(concat!(
                        "DELETE FROM ", $join_table,
                        " WHERE ", $parent, "_id = $1 AND ", $fk, " = $2"
                    ))
                    .bind([<$parent _id>])
                    .bind([<$fk:snake>])
                    .execute(executor)
                    .await
                    .context(format!(
                        concat!("failed to remove ", $fk, " '{}' from ", $parent, " '{}'"),
                        [<$fk:snake>], [<$parent _id>]
                    ))?;

                    Ok(result.rows_affected() > 0)
                }
            }
        }
    };
}

define_taxonomy_repo! {
    CategoryRepo, Category, "categories",
    parent: "shader", join_table: "shader_categories", fk: "category_id"
}

define_taxonomy_repo! {
    FeatureRepo, Feature, "features",
    parent: "shader", join_table: "shader_features", fk: "feature_id"
}

define_taxonomy_repo! {
    TagRepo, Tag, "tags",
    parent: "scene", join_table: "scene_tags", fk: "tag_id"
}
