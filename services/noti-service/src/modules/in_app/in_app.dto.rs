use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::common::entities::Notification;

/// Query parameters for listing user notifications.
#[derive(Debug, Deserialize, IntoParams)]
pub struct NotificationQueryDto {
    /// Number of records to return (Default: 20, Max: 100).
    pub limit: Option<u32>,
    /// Number of records to skip.
    pub offset: Option<u32>,
    /// When true, returns only unread notifications.
    pub unread_only: Option<bool>,
}

/// Outbound notification representation returned to clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NotificationResponseDto {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: String,
    /// Boolean flag indicating if the notification has been opened.
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Notification> for NotificationResponseDto {
    fn from(noti: Notification) -> Self {
        Self {
            id: noti.id,
            title: noti.title,
            content: noti.content,
            category: noti.category,
            is_read: noti.read_at.is_some(),
            read_at: noti.read_at,
            created_at: noti.created_at,
        }
    }
}

/// Unread notification count envelope.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UnreadCountResponseDto {
    /// Total count of notifications where `read_at` is NULL.
    pub unread_count: i64,
}

/// Generic message confirmation response.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageResponseDto {
    pub message: String,
}
