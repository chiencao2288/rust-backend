use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use super::dto::{
    MessageResponseDto, NotificationQueryDto, NotificationResponseDto, UnreadCountResponseDto,
};
use super::service::InAppService;
use crate::common::AppError;

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

    auth_header
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Unauthorized("Invalid Bearer token format".into()))
}

/// Liveness and readiness probe for noti-service.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy and ready to serve traffic", body = MessageResponseDto)
    ),
    tag = "Health"
)]
pub async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "service": "noti-service",
            "status": "healthy"
        })),
    )
}

/// Lists paginated in-app notifications for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notifications",
    params(NotificationQueryDto),
    responses(
        (status = 200, description = "List of notifications", body = Vec<NotificationResponseDto>),
        (status = 401, description = "Missing or invalid access token")
    ),
    security(("bearer_auth" = [])),
    tag = "InApp"
)]
pub async fn list_notifications(
    State(service): State<Arc<InAppService>>,
    headers: HeaderMap,
    Query(query): Query<NotificationQueryDto>,
) -> Result<(StatusCode, Json<Vec<NotificationResponseDto>>), AppError> {
    let token = extract_bearer_token(&headers)?;
    let user_id = service.authenticate_token(token).await?;
    let notifications = service.list_notifications(user_id, &query).await?;
    Ok((StatusCode::OK, Json(notifications)))
}

/// Returns the total unread notification count for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notifications/unread-count",
    responses(
        (status = 200, description = "Unread notifications count", body = UnreadCountResponseDto),
        (status = 401, description = "Missing or invalid access token")
    ),
    security(("bearer_auth" = [])),
    tag = "InApp"
)]
pub async fn get_unread_count(
    State(service): State<Arc<InAppService>>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<UnreadCountResponseDto>), AppError> {
    let token = extract_bearer_token(&headers)?;
    let user_id = service.authenticate_token(token).await?;
    let count = service.get_unread_count(user_id).await?;
    Ok((StatusCode::OK, Json(count)))
}

/// Marks a single notification as read.
#[utoipa::path(
    patch,
    path = "/api/v1/notifications/{id}/read",
    params(
        ("id" = Uuid, Path, description = "Notification UUID to mark as read")
    ),
    responses(
        (status = 200, description = "Notification marked as read", body = MessageResponseDto),
        (status = 404, description = "Notification not found or already read"),
        (status = 401, description = "Missing or invalid access token")
    ),
    security(("bearer_auth" = [])),
    tag = "InApp"
)]
pub async fn mark_as_read(
    State(service): State<Arc<InAppService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let token = extract_bearer_token(&headers)?;
    let user_id = service.authenticate_token(token).await?;
    service.mark_as_read(id, user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Notification marked as read" })),
    ))
}

/// Marks all unread notifications for the authenticated user as read.
#[utoipa::path(
    patch,
    path = "/api/v1/notifications/read-all",
    responses(
        (status = 200, description = "All notifications marked as read", body = MessageResponseDto),
        (status = 401, description = "Missing or invalid access token")
    ),
    security(("bearer_auth" = [])),
    tag = "InApp"
)]
pub async fn mark_all_as_read(
    State(service): State<Arc<InAppService>>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let token = extract_bearer_token(&headers)?;
    let user_id = service.authenticate_token(token).await?;
    let updated = service.mark_all_as_read(user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": format!("{updated} notifications marked as read") })),
    ))
}

/// Deletes a specific notification belonging to the authenticated user.
#[utoipa::path(
    delete,
    path = "/api/v1/notifications/{id}",
    params(
        ("id" = Uuid, Path, description = "Notification UUID to delete")
    ),
    responses(
        (status = 200, description = "Notification deleted successfully", body = MessageResponseDto),
        (status = 404, description = "Notification not found"),
        (status = 401, description = "Missing or invalid access token")
    ),
    security(("bearer_auth" = [])),
    tag = "InApp"
)]
pub async fn delete_notification(
    State(service): State<Arc<InAppService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let token = extract_bearer_token(&headers)?;
    let user_id = service.authenticate_token(token).await?;
    service.delete_notification(id, user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Notification deleted successfully" })),
    ))
}
