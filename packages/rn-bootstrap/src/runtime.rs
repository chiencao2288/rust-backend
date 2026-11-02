use std::future::Future;
use std::net::SocketAddr;

use axum::Router;
use tokio::signal;

/// Handle passed to the bootstrap closure, providing server lifecycle methods.
pub struct RNApp;

impl RNApp {
    /// Binds and serves an HTTP router with automatic graceful shutdown on SIGINT/SIGTERM.
    pub async fn serve_http(
        &self,
        router: Router,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        tracing::info!("HTTP server listening on http://{addr}");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

/// Waits for SIGINT (Ctrl+C) or SIGTERM, then returns.
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

/// Builder for configuring and running a service.
#[derive(Debug, Clone, Default)]
pub struct RNBootstrapBuilder {
    service_name: Option<String>,
    default_log_level: Option<String>,
}

impl RNBootstrapBuilder {
    /// Creates a default builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the service name used in log output.
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// Sets the default log level filter applied when `RUST_LOG` is not set.
    pub fn with_default_log_level(mut self, level: impl Into<String>) -> Self {
        self.default_log_level = Some(level.into());
        self
    }

    /// Initializes the runtime and runs the provided async closure.
    ///
    /// This method:
    /// 1. Installs a panic hook that logs panics via `tracing::error!`.
    /// 2. Initializes the `tracing` subscriber (text or JSON based on `LOG_FORMAT`).
    /// 3. Creates a Tokio runtime and blocks on the closure.
    /// 4. If the closure returns `Err`, logs the error and exits with code 1.
    /// 5. If a panic occurs, the panic hook logs it and the process aborts.
    pub fn run<F, Fut>(self, f: F) -> !
    where
        F: FnOnce(RNApp) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), Box<dyn std::error::Error>>> + Send,
    {
        let default_level = self.default_log_level.unwrap_or_else(|| "info".to_string());

        init_tracing(&default_level);
        install_panic_hook();

        if let Some(name) = &self.service_name {
            tracing::info!("Starting service: {name}");
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let result = runtime.block_on(f(RNApp));

        match result {
            Ok(()) => {
                tracing::info!("Service stopped gracefully");
                std::process::exit(0);
            }
            Err(e) => {
                tracing::error!("Fatal error: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Initializes the tracing subscriber.
///
/// Uses JSON format when `LOG_FORMAT=json`, otherwise uses human-readable text.
/// Respects `RUST_LOG` when set; falls back to the provided default level.
fn init_tracing(default_level: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| default_level.into());

    let is_json = std::env::var("LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if is_json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
}

/// Installs a panic hook that logs panics through `tracing::error!` before aborting.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|l| format!(" at {}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();

        tracing::error!("PANIC{location}: {payload}");
        default_hook(info);
    }));
}
