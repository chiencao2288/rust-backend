use axum::{
    body::{Body, Bytes},
    extract::Request,
    middleware::Next,
    response::Response,
};
use chrono::{SecondsFormat, Utc};
use rn_utils::sanitize_json_value;
use serde_json::{json, Value};

/// Checks if the service is operating in development mode.
fn is_dev_mode() -> bool {
    std::env::var("APP_ENV")
        .map(|v| v.trim().to_lowercase() != "production")
        .unwrap_or(true)
}

/// Parses request metadata (method, URL, query, and sanitized body) while buffering body bytes.
async fn extract_request_info(request: Request) -> (Request, Value) {
    let method = request.method().to_string();
    let url = request.uri().path().to_string();

    let mut query_map = serde_json::Map::new();
    if let Some(q) = request.uri().query() {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                query_map.insert(k.to_string(), json!(v));
            } else if !pair.is_empty() {
                query_map.insert(pair.to_string(), json!(true));
            }
        }
    }

    let (parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => Bytes::new(),
    };

    let mut body_json: Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    sanitize_json_value(&mut body_json);

    let client_request = json!({
        "method": method,
        "url": url,
        "body": body_json,
        "query": Value::Object(query_map),
        "params": json!({}),
        "timestamp": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    });

    let reconstructed_request = Request::from_parts(parts, Body::from(bytes));
    (reconstructed_request, client_request)
}

/// Injects captured client request metadata into an error response envelope.
async fn attach_request_to_error_response(response: Response, client_request: Value) -> Response {
    let (parts, body) = response.into_parts();

    if let Ok(bytes) = axum::body::to_bytes(body, 1024 * 1024).await {
        if let Ok(mut json_val) = serde_json::from_slice::<Value>(&bytes) {
            if let Some(root_obj) = json_val.as_object_mut() {
                root_obj.insert("request".to_string(), client_request);
                let new_body_bytes =
                    serde_json::to_vec(&json_val).unwrap_or_else(|_| bytes.to_vec());
                return Response::from_parts(parts, Body::from(new_body_bytes));
            }
        }
        return Response::from_parts(parts, Body::from(bytes));
    }

    Response::from_parts(parts, Body::empty())
}

/// Axum middleware capturing client request metadata for development error responses.
pub async fn dev_request_capture_middleware(request: Request, next: Next) -> Response {
    if !is_dev_mode() {
        return next.run(request).await;
    }

    let (reconstructed_request, client_request) = extract_request_info(request).await;
    let response = next.run(reconstructed_request).await;

    if !response.status().is_success() {
        attach_request_to_error_response(response, client_request).await
    } else {
        response
    }
}
