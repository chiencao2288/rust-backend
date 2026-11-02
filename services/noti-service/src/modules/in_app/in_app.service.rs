use uuid::Uuid;

use super::dto::{NotificationQueryDto, NotificationResponseDto, UnreadCountResponseDto};
use crate::app::AppState;
use crate::common::constant::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::common::repositories::NotificationRepository;
use crate::common::AppError;
use rn_contracts::noti::SendInAppPayload;

/// Core service managing in-app notifications, queries, and authentication verification.
pub struct InAppService {
    repo: NotificationRepository,
    http_client: reqwest::Client,
    auth_service_url: String,
}

impl InAppService {
    /// Creates a new `InAppService` initialized from the shared application state.
    pub fn new(state: &AppState) -> Self {
        Self {
            repo: state.noti_repo.clone(),
            http_client: state.http_client.clone(),
            auth_service_url: state.config.auth_service_url.clone(),
        }
    }

    /// Verifies an access token with auth-service and extracts the user ID.
    pub async fn authenticate_token(&self, token: &str) -> Result<Uuid, AppError> {
        let verified =
            rn_contracts::auth::verify_token(&self.http_client, &self.auth_service_url, token)
                .await
                .map_err(|e| AppError::Unauthorized(format!("Token verification failed: {e}")))?;

        Ok(verified.user_id)
    }

    /// Fetches a paginated slice of notifications for a user, mapping to response DTOs.
    pub async fn list_notifications(
        &self,
        user_id: Uuid,
        query: &NotificationQueryDto,
    ) -> Result<Vec<NotificationResponseDto>, AppError> {
        let limit = query.limit.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);
        let offset = query.offset.unwrap_or(0);
        let unread_only = query.unread_only.unwrap_or(false);

        let records = self
            .repo
            .find_by_user(user_id, limit, offset, unread_only)
            .await?;

        Ok(records
            .into_iter()
            .map(NotificationResponseDto::from)
            .collect())
    }

    /// Returns the total number of unread notifications for a user.
    pub async fn get_unread_count(
        &self,
        user_id: Uuid,
    ) -> Result<UnreadCountResponseDto, AppError> {
        let unread_count = self.repo.count_unread(user_id).await?;
        Ok(UnreadCountResponseDto { unread_count })
    }

    /// Marks a specific notification as read.
    pub async fn mark_as_read(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let modified = self.repo.mark_as_read(id, user_id).await?;
        if !modified {
            return Err(AppError::NotFound(
                "Notification not found or already read".into(),
            ));
        }
        Ok(())
    }

    /// Marks all unread notifications for a user as read.
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<u64, AppError> {
        self.repo.mark_all_as_read(user_id).await
    }

    /// Deletes a specific notification.
    pub async fn delete_notification(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let deleted = self.repo.delete_by_id(id, user_id).await?;
        if !deleted {
            return Err(AppError::NotFound("Notification not found".into()));
        }
        Ok(())
    }

    /// Inserts a new in-app notification originating from a RabbitMQ event.
    pub async fn create_notification(&self, payload: &SendInAppPayload) -> Result<Uuid, AppError> {
        self.repo
            .create(
                payload.user_id,
                &payload.title,
                &payload.content,
                &payload.category,
            )
            .await
    }
}
