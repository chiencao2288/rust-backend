use std::future::Future;
use std::pin::Pin;

/// Pinned, thread-safe asynchronous future.
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Future representing a running server task that resolves on exit.
pub type BoxedServerFuture = BoxFuture<Result<(), Box<dyn std::error::Error>>>;

/// Trait for configuration structs that can be loaded from environment variables.
pub trait EnvConfig: Sized {
    /// Loads the configuration, panicking on missing or invalid variables.
    fn load() -> Self;
}

/// Trait for servers that can be executed on a TCP port.
pub trait RpcServer: Send + 'static {
    /// Serves incoming RPC requests on the specified port until shutdown.
    fn serve(self, port: u16) -> BoxedServerFuture;
}

impl<Fut> RpcServer for Fut
where
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static,
{
    fn serve(self, _port: u16) -> BoxedServerFuture {
        Box::pin(self)
    }
}
