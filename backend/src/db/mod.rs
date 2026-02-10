use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tracing::debug;

pub type DbPool = Pool<Postgres>;
pub type DbTransaction<'a> = sqlx::Transaction<'a, Postgres>;

/// Apply all view definitions from views.sql.
/// Uses CREATE OR REPLACE VIEW so this is idempotent.
pub async fn apply_views(pool: &DbPool) -> anyhow::Result<()> {
    let views_sql = include_str!("../../views.sql");
    sqlx::raw_sql(views_sql).execute(pool).await?;
    debug!("Database views applied");
    Ok(())
}

pub async fn init_pool(database_url: &str) -> anyhow::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Apply/refresh view definitions
    apply_views(&pool).await?;

    Ok(pool)
}
