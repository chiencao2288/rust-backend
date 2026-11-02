//! Procedural macros collection for the rn monorepo.
//!
//! Available features:
//! - `bootstrap`: Enables the `#[derive(EnvConfig)]` macro for environment-based configuration.
//! - `error`: Enables the `#[derive(ApiError)]` macro for standard Axum HTTP error responses.

#[cfg(feature = "bootstrap")]
mod bootstrap;

#[cfg(feature = "error")]
mod error;

/// Derives `load()` for a struct that reads fields from environment variables.
///
/// # Attributes
///
/// - `#[env("KEY")]` — required, panics if missing or (for `String`) empty.
/// - `#[env("KEY", fallback = "value")]` — uses fallback value if the variable is absent.
/// - `#[env("KEY")]` on `Option<String>` / `Option<T>` — returns `None` if absent.
///
/// # Supported types
///
/// `String`, `Option<String>`, and any numeric type that implements `FromStr`
/// (`u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`),
/// plus `Option<T>` wrappers for each.
#[cfg(feature = "bootstrap")]
#[proc_macro_derive(EnvConfig, attributes(env))]
pub fn derive_env_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bootstrap::derive_env_config(input)
}

/// Derives `axum::response::IntoResponse`, `code()`, and `status_code()` for an enum.
///
/// # Attributes on variants
///
/// - `#[status(STATUS_CODE)]` — e.g. `#[status(BAD_REQUEST)]`
/// - `#[status(STATUS_CODE, code = "CUSTOM_CODE")]` — overrides the auto-derived SCREAMING_SNAKE_CASE code.
#[cfg(feature = "error")]
#[proc_macro_derive(ApiError, attributes(status))]
pub fn derive_api_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    error::derive_api_error(input)
}
