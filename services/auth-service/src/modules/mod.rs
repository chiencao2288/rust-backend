use axum::Router;

use crate::app::AppState;

pub mod auth;

/// Registers all application modules using the shared application state.
pub fn register_routes(state: AppState) -> Router {
    Router::new().merge(auth::register(state))
}
