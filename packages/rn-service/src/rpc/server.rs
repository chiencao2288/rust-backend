use std::future::Future;
use std::net::SocketAddr;

/// Runs an RPC server instance on the specified port.
pub async fn serve_rpc<F>(
    addr: SocketAddr,
    runner: impl FnOnce(SocketAddr) -> F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Future<Output = Result<(), Box<dyn std::error::Error>>> + Send,
{
    tracing::info!("RPC server listening on {addr}");
    runner(addr).await
}
