use chrono::Utc;
use noti_service::common::entities::Notification;
use noti_service::modules::in_app::dto::NotificationResponseDto;
use noti_service::modules::in_app::ApiDoc;
use utoipa::OpenApi;
use uuid::Uuid;

#[test]
fn test_notification_dto_read_at_mapping() {
    let now = Utc::now();
    let unread = Notification {
        id: Uuid::now_v7(),
        user_id: Uuid::now_v7(),
        title: "Test Title".into(),
        content: "Test Content".into(),
        category: "SYSTEM".into(),
        read_at: None,
        created_at: now,
    };

    let dto = NotificationResponseDto::from(unread);
    assert!(!dto.is_read);
    assert_eq!(dto.read_at, None);

    let read = Notification {
        id: Uuid::now_v7(),
        user_id: Uuid::now_v7(),
        title: "Test Title".into(),
        content: "Test Content".into(),
        category: "SECURITY".into(),
        read_at: Some(now),
        created_at: now,
    };

    let dto = NotificationResponseDto::from(read);
    assert!(dto.is_read);
    assert_eq!(dto.read_at, Some(now));
}

#[test]
fn test_noti_openapi_spec_generation() {
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).expect("OpenAPI spec should serialize");

    assert!(json.contains("/health"));
    assert!(json.contains("/api/v1/notifications"));
    assert!(json.contains("/api/v1/notifications/unread-count"));
    assert!(json.contains("/api/v1/notifications/{id}/read"));
    assert!(json.contains("/api/v1/notifications/read-all"));
    assert!(json.contains("bearer_auth"));
}
