use std::net::SocketAddr;

use clap::Parser;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, info, warn};

use glint::{cli, config::Config, db, platform, routes, services, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file from backend/ directory (works from any cwd)
    let backend_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dotenvy::from_path(backend_dir.join(".env")).ok();

    // Parse CLI arguments (before logging init to get verbosity)
    let cli = cli::Cli::parse();

    // Initialize tracing with compact formatter
    glint::logging::init_with_verbosity(cli.verbose.verbose);

    // Load configuration
    let config = Config::load()?;
    debug!("Configuration loaded");

    // Initialize database
    let pool = db::init_pool(&config.database_url).await?;
    debug!("Database initialized");

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            cli::Command::Seed => {
                cli::seed::run(&pool).await?;
                return Ok(());
            }
        }
    }

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
        debug!(
            endpoint = %r2_config.endpoint().unwrap_or_default(),
            bucket = %r2_config.bucket.as_deref().unwrap_or("glint"),
            "R2 client initialized"
        );
        Some(client)
    } else {
        warn!("R2 not configured, upload functionality disabled");
        None
    };

    // Initialize OAuth client if configured
    let oauth_client = if config.discord.is_configured() {
        let discord = &config.discord;
        let client =
            oauth2::basic::BasicClient::new(ClientId::new(discord.client_id.clone().unwrap()))
                .set_client_secret(ClientSecret::new(discord.client_secret.clone().unwrap()))
                .set_auth_uri(
                    AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())
                        .expect("Invalid Discord auth URL"),
                )
                .set_token_uri(
                    TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
                        .expect("Invalid Discord token URL"),
                )
                .set_redirect_uri(
                    RedirectUrl::new(discord.redirect_uri.clone().unwrap())
                        .expect("Invalid Discord redirect URI"),
                );

        info!("Discord OAuth configured");
        Some(client)
    } else {
        warn!("Discord OAuth not configured, authentication disabled");
        None
    };

    // Initialize platform clients
    let modrinth_client =
        platform::modrinth::ModrinthClient::new(&config.platform.modrinth_user_agent);
    info!(
        user_agent = %config.platform.modrinth_user_agent,
        "Modrinth client initialized"
    );

    let curseforge_client = if let Some(ref api_key) = config.platform.curseforge_api_key {
        let client = platform::curseforge::CurseForgeClient::new(api_key);
        info!("CurseForge client initialized");
        Some(client)
    } else {
        warn!("CurseForge API key not configured, CurseForge integration disabled");
        None
    };

    // Build application state
    let state = AppState::new(
        pool.clone(),
        config.clone(),
        s3_client.clone(),
        oauth_client,
        modrinth_client,
        curseforge_client,
    );

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
    info!(addr = %addr, "Server started");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
