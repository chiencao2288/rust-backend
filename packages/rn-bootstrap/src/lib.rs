//! Service bootstrap with config loading, tracing, panic hook, and HTTP server.

mod runtime;

pub use rn_macros::{ApiError, EnvConfig};
pub use runtime::{RNApp, RNBootstrapBuilder};

/// Entry point for building a bootstrap configuration.
pub struct RNBootstrap;

impl RNBootstrap {
    /// Creates a new bootstrap builder.
    pub fn builder() -> RNBootstrapBuilder {
        RNBootstrapBuilder::new()
    }
}
