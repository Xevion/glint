use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub mod utc_datetime;

pub type DbPool = Pool<Postgres>;
pub use utc_datetime::UtcDateTime;

pub async fn init_pool(database_url: &str) -> anyhow::Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
