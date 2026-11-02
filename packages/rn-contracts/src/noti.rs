use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Primary topic exchange for all notification events.
pub const NOTIFICATION_EXCHANGE: &str = "notifications.topic";

/// Routing key for email delivery events.
pub const ROUTING_KEY_EMAIL: &str = "notification.email";

/// Routing key for SMS delivery events.
pub const ROUTING_KEY_SMS: &str = "notification.sms";

/// Routing key for in-app notification persistence events.
pub const ROUTING_KEY_IN_APP: &str = "notification.in_app";

/// Queue dedicated to asynchronous email delivery workers.
pub const QUEUE_EMAIL: &str = "notifications.email";

/// Queue dedicated to asynchronous SMS delivery workers.
pub const QUEUE_SMS: &str = "notifications.sms";

/// Queue dedicated to in-app notification persistence workers.
pub const QUEUE_IN_APP: &str = "notifications.in_app";

/// Event payload for dispatching an email notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendEmailPayload {
    /// Recipient email address.
    pub to: String,
    /// Email subject line.
    pub subject: String,
    /// Template name to render (e.g. `register_otp`, `reset_password`).
    pub template: String,
    /// Dynamic template placeholder parameters (e.g. `otp`, `app_name`).
    pub params: HashMap<String, String>,
}

impl SendEmailPayload {
    /// Convenience constructor for building an email payload.
    pub fn new(
        to: impl Into<String>,
        subject: impl Into<String>,
        template: impl Into<String>,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            to: to.into(),
            subject: subject.into(),
            template: template.into(),
            params,
        }
    }
}

/// Event payload for dispatching an SMS text message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendSmsPayload {
    /// Recipient phone number (E.164 formatted).
    pub to: String,
    /// Plaintext SMS message body.
    pub content: String,
}

impl SendSmsPayload {
    /// Convenience constructor for building an SMS payload.
    pub fn new(to: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            content: content.into(),
        }
    }
}

/// Event payload for persisting an in-app user notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendInAppPayload {
    /// Target user identifier.
    pub user_id: Uuid,
    /// Notification title.
    pub title: String,
    /// Notification body content.
    pub content: String,
    /// Notification category (e.g. `SECURITY`, `SYSTEM`, `PROMOTION`).
    pub category: String,
}

impl SendInAppPayload {
    /// Convenience constructor for building an in-app notification payload.
    pub fn new(
        user_id: Uuid,
        title: impl Into<String>,
        content: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            user_id,
            title: title.into(),
            content: content.into(),
            category: category.into(),
        }
    }
}

/// Unified notification event container supporting all delivery channels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "channel", rename_all = "snake_case")]
pub enum NotificationEvent {
    /// Email notification.
    Email(SendEmailPayload),
    /// SMS text message.
    Sms(SendSmsPayload),
    /// In-app user notification.
    InApp(SendInAppPayload),
}
