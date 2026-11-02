use auth_service::common::constant::*;
use auth_service::common::AppError;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[test]
fn test_hierarchical_redis_key_generation() {
    let token = "access_123";
    assert_eq!(
        format!("{REDIS_KEY_SESSION}:{token}"),
        "auth:session:access_123"
    );

    let email = "user@example.com";
    assert_eq!(
        format!("{REDIS_KEY_OTP_CODE}:register:{email}"),
        "auth:otp:code:register:user@example.com"
    );

    assert_eq!(
        format!("{REDIS_KEY_OTP_CODE}:reset_password:{email}"),
        "auth:otp:code:reset_password:user@example.com"
    );

    assert_eq!(
        format!("{REDIS_KEY_OTP_COOLDOWN}:register:{email}"),
        "auth:otp:cooldown:register:user@example.com"
    );

    assert_eq!(
        format!("{REDIS_KEY_OTP_RATE}:register:{email}"),
        "auth:otp:rate:register:user@example.com"
    );

    assert_eq!(
        format!("{REDIS_KEY_OTP_ATTEMPTS}:register:{email}"),
        "auth:otp:attempts:register:user@example.com"
    );

    let hash = "sha256_hash_value";
    assert_eq!(
        format!("{REDIS_KEY_DELETED_ACCOUNT_COOLDOWN}:{hash}"),
        "auth:cooldown:deleted_account:sha256_hash_value"
    );
}

#[tokio::test]
async fn test_too_many_requests_error_response_envelope() {
    let err =
        AppError::TooManyRequests("Please wait 60 seconds before requesting another code".into());
    assert_eq!(err.code(), "TOO_MANY_REQUESTS");
    assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);

    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["data"]["code"], "TOO_MANY_REQUESTS");
    assert_eq!(
        json["data"]["message"],
        "Please wait 60 seconds before requesting another code"
    );
    assert!(json["data"].get("details").is_none());
}
