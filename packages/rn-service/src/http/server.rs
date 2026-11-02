use std::future::Future;
use std::net::SocketAddr;

use axum::Router;

/// Binds an Axum HTTP listener to the target address and serves traffic with graceful shutdown.
pub async fn serve_http(
    addr: SocketAddr,
    router: Router,
    shutdown: impl Future<Output = ()> + Send,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("HTTP server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}
