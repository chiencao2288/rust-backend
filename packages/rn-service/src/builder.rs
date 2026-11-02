use std::future::Future;

use crate::stage::RNServiceStateStage;
use crate::telemetry::{init_tracing, install_panic_hook};
use crate::traits::EnvConfig;

/// Entry point for building a microservice lifecycle pipeline.
pub struct RNService;

impl RNService {
    /// Creates a new service builder instance.
    pub fn builder() -> RNServiceBuilder {
        RNServiceBuilder::new()
    }
}

/// Initial builder stage configuring service metadata and logging directives.
#[derive(Debug, Clone, Default)]
pub struct RNServiceBuilder {
    service_name: Option<String>,
    default_log_level: Option<String>,
}

impl RNServiceBuilder {
    /// Creates a new default builder instance.
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

    /// Advances to the config-building stage, initializing tracing, panic hook, and loading config.
    pub fn build_config<C: EnvConfig>(self) -> RNServiceConfigStage<C> {
        let default_level = self.default_log_level.unwrap_or_else(|| "info".to_string());

        init_tracing(&default_level);
        install_panic_hook();

        if let Some(name) = &self.service_name {
            tracing::info!("Starting service: {name}");
        }

        let config = C::load();
        RNServiceConfigStage {
            config,
            service_name: self.service_name,
        }
    }
}

/// Stage where configuration has been verified, ready to build application state.
pub struct RNServiceConfigStage<C> {
    config: C,
    service_name: Option<String>,
}

impl<C: Send + 'static> RNServiceConfigStage<C> {
    /// Initializes application state asynchronously using the verified configuration.
    pub fn build_state<S, F, Fut>(self, f: F) -> RNServiceStateStage<S>
    where
        S: Clone + Send + Sync + 'static,
        F: FnOnce(C) -> Fut + Send + 'static,
        Fut: Future<Output = Result<S, Box<dyn std::error::Error>>> + Send + 'static,
    {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let state = match runtime.block_on(f(self.config)) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to initialize application state: {e}");
                std::process::exit(1);
            }
        };

        RNServiceStateStage::new(runtime, state, self.service_name)
    }
}
