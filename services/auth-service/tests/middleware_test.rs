use auth_service::common::{dev_request_capture_middleware, AppError};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::post,
    Router,
};
use http_body_util::BodyExt;
use serde_json::json;
use tokio::sync::Mutex;
use tower::ServiceExt;

static TEST_LOCK: Mutex<()> = Mutex::const_new(());

async fn failing_handler() -> Result<StatusCode, AppError> {
    Err(AppError::InvalidCredentials)
}

#[tokio::test]
async fn test_middleware_captures_client_request_in_dev() {
    let _guard = TEST_LOCK.lock().await;
    std::env::set_var("APP_ENV", "development");

    let app = Router::new()
        .route("/api/v1/auth/login", post(failing_handler))
        .layer(middleware::from_fn(dev_request_capture_middleware));

    let request = Request::builder()
        .uri("/api/v1/auth/login?debug=true")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "dev@example.com",
                "password": "secret_password"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["data"]["code"], "INVALID_CREDENTIALS");
    assert_eq!(json["data"]["message"], "Invalid email or password");

    // Exact structure requested by user:
    assert_eq!(json["request"]["method"], "POST");
    assert_eq!(json["request"]["url"], "/api/v1/auth/login");
    assert_eq!(json["request"]["query"]["debug"], "true");
    assert_eq!(json["request"]["body"]["email"], "dev@example.com");
    assert_eq!(json["request"]["body"]["password"], "******");
    assert!(json["request"]["timestamp"].is_string());
}

#[tokio::test]
async fn test_middleware_omits_request_in_production() {
    let _guard = TEST_LOCK.lock().await;
    std::env::set_var("APP_ENV", "production");

    let app = Router::new()
        .route("/api/v1/auth/login", post(failing_handler))
        .layer(middleware::from_fn(dev_request_capture_middleware));

    let request = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "email": "dev@example.com" }).to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["data"]["code"], "INVALID_CREDENTIALS");
    assert!(json.get("request").is_none());

    std::env::set_var("APP_ENV", "development");
}
