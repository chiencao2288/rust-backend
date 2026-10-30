pub mod controller;
pub mod dto;
pub mod service;
pub mod validation;

pub use service::AuthService;

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::common::dev_request_capture_middleware;

#[derive(OpenApi)]
#[openapi(
    paths(
        controller::health,
        controller::register_otp,
        controller::register,
        controller::login,
        controller::refresh,
        controller::logout,
        controller::forgot_password,
        controller::reset_password,
        controller::delete_account,
        controller::verify_token,
    ),
    components(
        schemas(
            dto::RegisterOtpDto,
            dto::RegisterDto,
            dto::LoginDto,
            dto::RefreshTokenDto,
            dto::LogoutDto,
            dto::ForgotPasswordDto,
            dto::ResetPasswordDto,
            dto::VerifyTokenDto,
            dto::VerifyTokenResponseDto,
            dto::MessageResponseDto,
            crate::modules::tokens::AuthTokens,
        )
    ),
    tags(
        (name = "Auth", description = "Authentication and session management API"),
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

/// Configures and returns the Auth module router with all routes, middleware, and Swagger UI mounted.
pub fn auth_router(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(controller::health))
        .route("/api/v1/auth/register/otp", post(controller::register_otp))
        .route("/api/v1/auth/register", post(controller::register))
        .route("/api/v1/auth/login", post(controller::login))
        .route("/api/v1/auth/refresh", post(controller::refresh))
        .route("/api/v1/auth/logout", post(controller::logout))
        .route(
            "/api/v1/auth/forgot-password",
            post(controller::forgot_password),
        )
        .route(
            "/api/v1/auth/reset-password",
            post(controller::reset_password),
        )
        .route("/api/v1/auth/account", delete(controller::delete_account))
        .route("/api/v1/auth/verify", post(controller::verify_token))
        .layer(axum::middleware::from_fn(dev_request_capture_middleware))
        .with_state(auth_service)
}
