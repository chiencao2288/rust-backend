use rn_postgres::RNPostgres;
use rn_redis::RNRedis;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;

use auth_service::config::Config;
use auth_service::modules::auth::{auth_router, AuthService};
use auth_service::modules::tokens::{SessionStore, TokensRepository};
use auth_service::modules::users::UsersRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,auth_service=debug".into()),
        )
        .init();

    // Strict fail-fast configuration: terminates process on any missing required variable
    let config = Config::load_or_exit();

    let mut db_builder = RNPostgres::builder().with_master_url(&config.database_master_url);
    for slave_url in &config.database_slave_urls {
        db_builder = db_builder.add_slave_url(slave_url);
    }
    let db = db_builder.build().await?;

    let mut redis_builder = RNRedis::builder().with_nodes(&config.redis_nodes);
    if let Some(pwd) = &config.redis_password {
        redis_builder = redis_builder.with_password(pwd);
    }
    let redis = redis_builder.build().await?;

    let users_repo = UsersRepository::new(db.clone());
    let tokens_repo = TokensRepository::new(db);
    let session_store = SessionStore::new(redis);
    let http_client = reqwest::Client::new();

    let auth_service = Arc::new(AuthService::new(
        users_repo,
        tokens_repo,
        session_store,
        http_client,
        config.session_ttl_secs,
        config.refresh_token_ttl_days,
        config.google_client_id,
    ));

    let app = auth_router(auth_service);

    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Auth service running at http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("Received SIGTERM, shutting down"),
    }
}
