use rn_postgres::RNPostgres;
use uuid::Uuid;

use crate::common::entities::Notification;
use crate::common::AppError;

/// Data access repository for the `notifications` PostgreSQL table.
#[derive(Clone)]
pub struct NotificationRepository {
    db: RNPostgres,
}

impl NotificationRepository {
    /// Creates a new `NotificationRepository` instance wrapping `RNPostgres`.
    pub fn new(db: RNPostgres) -> Self {
        Self { db }
    }

    /// Inserts a new in-app notification on the primary master database.
    pub async fn create(
        &self,
        user_id: Uuid,
        title: &str,
        content: &str,
        category: &str,
    ) -> Result<Uuid, AppError> {
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO notifications (user_id, title, content, category)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(title)
        .bind(content)
        .bind(category)
        .fetch_one(self.db.master())
        .await?;

        Ok(id)
    }

    /// Fetches a paginated list of notifications for a specific user from the read replica.
    pub async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
        unread_only: bool,
    ) -> Result<Vec<Notification>, AppError> {
        let notifications = if unread_only {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT id, user_id, title, content, category, read_at, created_at
                FROM notifications
                WHERE user_id = $1 AND read_at IS NULL
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(self.db.slave())
            .await?
        } else {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT id, user_id, title, content, category, read_at, created_at
                FROM notifications
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(self.db.slave())
            .await?
        };

        Ok(notifications)
    }

    /// Counts the total number of unread notifications for a specific user.
    pub async fn count_unread(&self, user_id: Uuid) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM notifications
            WHERE user_id = $1 AND read_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.slave())
        .await?;

        Ok(count)
    }

    /// Marks a single notification as read if not already read. Returns true if modified.
    pub async fn mark_as_read(&self, id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET read_at = now()
            WHERE id = $1 AND user_id = $2 AND read_at IS NULL
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(self.db.master())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Marks all unread notifications for a specific user as read. Returns count of rows updated.
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET read_at = now()
            WHERE user_id = $1 AND read_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(self.db.master())
        .await?;

        Ok(result.rows_affected())
    }

    /// Deletes a specific notification belonging to a user. Returns true if deleted.
    pub async fn delete_by_id(&self, id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM notifications
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(self.db.master())
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
