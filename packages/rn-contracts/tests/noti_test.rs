use std::collections::HashMap;

use rn_contracts::noti::{
    NotificationEvent, SendEmailPayload, SendInAppPayload, SendSmsPayload, NOTIFICATION_EXCHANGE,
    QUEUE_EMAIL, QUEUE_IN_APP, QUEUE_SMS, ROUTING_KEY_EMAIL, ROUTING_KEY_IN_APP, ROUTING_KEY_SMS,
};
use uuid::Uuid;

#[test]
fn test_noti_contract_constants() {
    assert_eq!(NOTIFICATION_EXCHANGE, "notifications.topic");
    assert_eq!(ROUTING_KEY_EMAIL, "notification.email");
    assert_eq!(ROUTING_KEY_SMS, "notification.sms");
    assert_eq!(ROUTING_KEY_IN_APP, "notification.in_app");
    assert_eq!(QUEUE_EMAIL, "notifications.email");
    assert_eq!(QUEUE_SMS, "notifications.sms");
    assert_eq!(QUEUE_IN_APP, "notifications.in_app");
}

#[test]
fn test_noti_payload_serialization() {
    let mut params = HashMap::new();
    params.insert("otp".into(), "123456".into());

    let email = SendEmailPayload::new("user@example.com", "Test Subject", "register_otp", params);
    let event = NotificationEvent::Email(email);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"channel\":\"email\""));
    assert!(json.contains("123456"));

    let deserialized: NotificationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}

#[test]
fn test_in_app_payload_serialization() {
    let user_id = Uuid::now_v7();
    let in_app = SendInAppPayload::new(user_id, "Welcome", "Welcome to our app!", "SYSTEM");
    let event = NotificationEvent::InApp(in_app);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"channel\":\"in_app\""));
    assert!(json.contains("Welcome to our app!"));

    let deserialized: NotificationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}

#[test]
fn test_sms_payload_serialization() {
    let sms = SendSmsPayload::new("+84912345678", "Your OTP is 987654");
    let event = NotificationEvent::Sms(sms);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"channel\":\"sms\""));
    assert!(json.contains("+84912345678"));

    let deserialized: NotificationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
