use std::net::SocketAddr;

use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use glint_backend::{config::Config, db, routes, services, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file from backend/ directory (works from any cwd)
    let backend_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dotenvy::from_path(backend_dir.join(".env")).ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "glint_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded");

    // Initialize database
    let pool = db::init_pool(&config.database_url).await?;
    tracing::info!("Database initialized");

    // Initialize S3/R2 client if configured
    let s3_client = if config.r2.is_configured() {
        let r2_config = &config.r2;
        let credentials = aws_sdk_s3::config::Credentials::new(
            r2_config.access_key_id.as_deref().unwrap_or_default(),
            r2_config.secret_access_key.as_deref().unwrap_or_default(),
            None,
            None,
            "glint",
        );

        let timeout_config = aws_sdk_s3::config::timeout::TimeoutConfig::builder()
            .operation_timeout(std::time::Duration::from_secs(120))
            .operation_attempt_timeout(std::time::Duration::from_secs(60))
            .build();

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(aws_sdk_s3::config::Region::new("auto"))
            .endpoint_url(r2_config.endpoint().unwrap_or_default())
            .credentials_provider(credentials)
            .timeout_config(timeout_config)
            .force_path_style(true)
            .build();

        let client = aws_sdk_s3::Client::from_conf(s3_config);
        tracing::info!(
            "R2/S3 client initialized (endpoint: {}, bucket: {})",
            r2_config.endpoint().unwrap_or_default(),
            r2_config.bucket.as_deref().unwrap_or("glint")
        );
        Some(client)
    } else {
        tracing::warn!("R2 not configured - upload functionality will be disabled");
        None
    };

    // Build application state
    let state = AppState::new(pool.clone(), config.clone(), s3_client.clone());

    // Start heartbeat monitoring background task
    tokio::spawn(services::heartbeat::monitor_heartbeats(
        pool.clone(),
        config.heartbeat.clone(),
    ));

    // Start upload cleanup background task
    let cleanup_bucket = config
        .r2
        .bucket
        .clone()
        .unwrap_or_else(|| "glint".to_string());
    tokio::spawn(services::upload_cleanup::cleanup_expired_uploads(
        pool,
        s3_client.clone(),
        cleanup_bucket,
    ));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = routes::router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Starting server on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
