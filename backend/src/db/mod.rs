use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

pub mod utc_datetime;

pub type DbPool = Pool<Sqlite>;
pub use utc_datetime::UtcDateTime;

pub async fn init_pool(database_url: &str) -> anyhow::Result<DbPool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
