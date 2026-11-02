use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// Authenticated user identity context injected into request extensions by `require_auth`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUser {
    /// Unique user identifier.
    pub user_id: Uuid,
    /// Primary user email address.
    pub email: String,
}

/// State required by the authentication middleware to verify Bearer tokens with auth-service.
#[derive(Clone)]
pub struct AuthState {
    pub http_client: reqwest::Client,
    pub auth_service_url: String,
}

impl AuthState {
    /// Creates a new `AuthState` instance.
    pub fn new(http_client: reqwest::Client, auth_service_url: impl Into<String>) -> Self {
        Self {
            http_client,
            auth_service_url: auth_service_url.into(),
        }
    }
}

/// Axum middleware enforcing Bearer token authentication against `auth-service`.
///
/// On successful token verification, injects `AuthUser` into `request.extensions()`.
/// If the token is absent or invalid, returns an immediate `401 Unauthorized` response.
pub async fn require_auth(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = match request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        Some(h) => h,
        None => return unauthorized_response("Missing Authorization header"),
    };

    let token = match auth_header.strip_prefix("Bearer ").map(str::trim).filter(|s| !s.is_empty()) {
        Some(t) => t,
        None => return unauthorized_response("Invalid Bearer token format"),
    };

    let verified = match rn_contracts::auth::verify_token(
        &state.http_client,
        &state.auth_service_url,
        token,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Token verification failed: {e}");
            return unauthorized_response("Invalid or expired access token");
        }
    };

    request.extensions_mut().insert(AuthUser {
        user_id: verified.user_id,
        email: verified.email,
    });

    next.run(request).await
}

fn unauthorized_response(message: &str) -> Response {
    let body = json!({
        "success": false,
        "data": {
            "code": "UNAUTHORIZED",
            "message": message,
            "details": null
        }
    });
    (StatusCode::UNAUTHORIZED, Json(body)).into_response()
}
