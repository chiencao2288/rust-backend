use auth_service::common::AppError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tokio::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

#[tokio::test]
async fn test_error_envelope_structure() {
    let _guard = ENV_LOCK.lock().await;

    let err = AppError::InvalidCredentials;
    assert_eq!(err.code(), "INVALID_CREDENTIALS");

    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["data"]["code"], "INVALID_CREDENTIALS");
    assert_eq!(json["data"]["message"], "Invalid email or password");
    assert!(json["data"].get("details").is_none());
}

#[tokio::test]
async fn test_validation_failed_with_details() {
    let _guard = ENV_LOCK.lock().await;

    let details = serde_json::json!({
        "email": {
            "code": "INVALID_EMAIL",
            "message": "email must be a valid email address"
        },
        "password": {
            "code": "PASSWORD_INVALID_FORMAT",
            "message": "password must contain at least one uppercase letter"
        }
    });

    let err = AppError::ValidationFailed {
        message: "Validation failed".into(),
        details,
    };

    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["data"]["code"], "VALIDATION_FAILED");
    assert_eq!(json["data"]["message"], "Validation failed");
    assert_eq!(json["data"]["details"]["email"]["code"], "INVALID_EMAIL");
    assert_eq!(
        json["data"]["details"]["password"]["code"],
        "PASSWORD_INVALID_FORMAT"
    );
}

#[tokio::test]
async fn test_internal_server_error_never_leaks_details() {
    let _guard = ENV_LOCK.lock().await;

    let err = AppError::Internal("sensitive database error or stacktrace".into());
    assert_eq!(err.code(), "INTERNAL_SERVER_ERROR");

    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["data"]["code"], "INTERNAL_SERVER_ERROR");
    assert_eq!(json["data"]["message"], "Internal server error");
    assert!(json["data"].get("details").is_none());
}
