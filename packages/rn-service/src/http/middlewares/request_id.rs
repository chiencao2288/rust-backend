use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// Header name used for distributed request tracing.
pub const HEADER_REQUEST_ID: &str = "x-request-id";

/// Axum middleware ensuring every request carries an `x-request-id` header (using UUIDv7 if absent).
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = match request.headers().get(HEADER_REQUEST_ID).and_then(|v| v.to_str().ok()) {
        Some(existing) if !existing.trim().is_empty() => existing.trim().to_string(),
        _ => Uuid::now_v7().to_string(),
    };

    if let Ok(val) = HeaderValue::from_str(&request_id) {
        request.headers_mut().insert(HEADER_REQUEST_ID, val);
    }

    let mut response = next.run(request).await;

    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(HEADER_REQUEST_ID, val);
    }

    response
}
