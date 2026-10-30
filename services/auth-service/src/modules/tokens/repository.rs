use chrono::{DateTime, Utc};
use rn_postgres::RNPostgres;
use uuid::Uuid;

use super::entity::RefreshToken;
use crate::common::AppError;

/// Data access repository for the `refresh_tokens` PostgreSQL table.
///
/// Encapsulates token lifecycle storage, lookups, and hard-deletion sweeps.
#[derive(Clone)]
pub struct TokensRepository {
    db: RNPostgres,
}

impl TokensRepository {
    /// Creates a new `TokensRepository` instance wrapping an `RNPostgres` client.
    ///
    /// # Arguments
    /// * `db` - The PostgreSQL client managing master and slave connection pools.
    pub fn new(db: RNPostgres) -> Self {
        Self { db }
    }

    /// Stores a new hashed refresh token on the primary master database.
    ///
    /// # Arguments
    /// * `user_id` - The user UUID to which this refresh token belongs.
    /// * `token_hash` - The SHA-256 hash string of the plaintext refresh token.
    /// * `expires_at` - UTC timestamp when this refresh token becomes invalid.
    ///
    /// # Returns
    /// * `Ok(())` upon successful persistence.
    ///
    /// # Errors
    /// * `AppError::Conflict` if `token_hash` violates uniqueness constraints.
    /// * `AppError::Internal` on database connection or query failure.
    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(self.db.master())
        .await?;
        Ok(())
    }

    /// Queries the read replica for an active and non-expired refresh token by its SHA-256 hash.
    ///
    /// # Arguments
    /// * `token_hash` - The SHA-256 hash string of the presented refresh token.
    ///
    /// # Returns
    /// * `Ok(Some(RefreshToken))` if a valid, unexpired token record exists.
    /// * `Ok(None)` if nonexistent or expired.
    ///
    /// # Errors
    /// * `AppError::Internal` on database query failure.
    pub async fn find_valid_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AppError> {
        let token = sqlx::query_as::<_, RefreshToken>(
            "SELECT id, user_id, token_hash, expires_at, created_at FROM refresh_tokens WHERE token_hash = $1 AND expires_at > now()",
        )
        .bind(token_hash)
        .fetch_optional(self.db.slave())
        .await?;
        Ok(token)
    }

    /// Hard-deletes a refresh token record by its primary key ID on the primary master.
    ///
    /// # Arguments
    /// * `id` - The refresh token's UUID primary key.
    ///
    /// # Returns
    /// * `Ok(())` upon execution.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn delete_refresh_token_by_id(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(id)
            .execute(self.db.master())
            .await?;
        Ok(())
    }

    /// Hard-deletes a refresh token record matching the specified SHA-256 hash on the primary master.
    ///
    /// # Arguments
    /// * `token_hash` - The SHA-256 hash string to purge.
    ///
    /// # Returns
    /// * `Ok(())` upon execution.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn delete_refresh_token_by_hash(&self, token_hash: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(self.db.master())
            .await?;
        Ok(())
    }

    /// Hard-deletes all refresh tokens belonging to a user across all devices on the primary master.
    ///
    /// # Arguments
    /// * `user_id` - The user UUID whose active sessions should be purged.
    ///
    /// # Returns
    /// * `Ok(())` upon execution.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn delete_all_refresh_tokens_for_user(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(self.db.master())
            .await?;
        Ok(())
    }
}
