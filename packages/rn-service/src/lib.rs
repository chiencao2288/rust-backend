//! Unified microservice framework for rn-services.
//!
//! Provides service lifecycle orchestration, HTTP & RPC server execution,
//! and standard HTTP middlewares.

mod builder;
pub mod http;
pub mod rpc;
mod signal;
mod stage;
mod telemetry;
mod traits;

pub use builder::{RNService, RNServiceBuilder, RNServiceConfigStage};
pub use http::middlewares::auth::{require_auth, AuthState, AuthUser};
pub use traits::{BoxFuture, BoxedServerFuture, EnvConfig, RpcServer};
pub use http::middlewares::request_id::{request_id_middleware, HEADER_REQUEST_ID};
pub use rn_macros::{ApiError, EnvConfig};
pub use signal::shutdown_signal;
pub use stage::RNServiceStateStage;
pub use telemetry::{init_tracing, install_panic_hook};
pub use traits::{BoxFuture, BoxedServerFuture, EnvConfig as EnvConfigTrait, RpcServer};
