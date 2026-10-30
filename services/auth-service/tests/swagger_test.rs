use auth_service::modules::auth::ApiDoc;
use utoipa::OpenApi;

#[test]
fn test_openapi_spec_generation() {
    let openapi = ApiDoc::openapi();
    let json =
        serde_json::to_string_pretty(&openapi).expect("OpenAPI spec should serialize to JSON");

    assert!(json.contains("/api/v1/auth/register/otp"));
    assert!(json.contains("/api/v1/auth/register"));
    assert!(json.contains("/api/v1/auth/login"));
    assert!(json.contains("/api/v1/auth/refresh"));
    assert!(json.contains("/api/v1/auth/logout"));
    assert!(json.contains("/api/v1/auth/forgot-password"));
    assert!(json.contains("/api/v1/auth/reset-password"));
    assert!(json.contains("/api/v1/auth/account"));
    assert!(json.contains("/api/v1/auth/verify"));
    assert!(json.contains("/health"));
    assert!(json.contains("bearer_auth"));
}
