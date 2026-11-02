#[path = "in_app.consumer.rs"]
pub mod consumer;
#[path = "in_app.controller.rs"]
pub mod controller;
#[path = "in_app.dto.rs"]
pub mod dto;
#[path = "in_app.service.rs"]
pub mod service;

pub use consumer::start_in_app_consumer;
pub use service::InAppService;

use axum::{
    routing::{delete, get, patch},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        controller::health,
        controller::list_notifications,
        controller::get_unread_count,
        controller::mark_as_read,
        controller::mark_all_as_read,
        controller::delete_notification,
    ),
    components(
        schemas(
            dto::NotificationResponseDto,
            dto::UnreadCountResponseDto,
            dto::MessageResponseDto,
        )
    ),
    tags(
        (name = "InApp", description = "In-App user notification center APIs"),
        (name = "Health", description = "Liveness and readiness probes")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("Token")
                        .build(),
                ),
            );
        }
    }
}

/// Registers the In-App module routes and Swagger UI documentation at /api/docs.
#[rustfmt::skip]
pub fn register(state: AppState) -> Router {
    let service = Arc::new(InAppService::new(&state));

    Router::new()
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(controller::health))
        .route("/api/v1/notifications", get(controller::list_notifications))
        .route("/api/v1/notifications/unread-count", get(controller::get_unread_count))
        .route("/api/v1/notifications/read-all", patch(controller::mark_all_as_read))
        .route("/api/v1/notifications/{id}/read", patch(controller::mark_as_read))
        .route("/api/v1/notifications/{id}", delete(controller::delete_notification))
        .with_state(service)
}
