use rn_postgres::RNPostgres;
use uuid::Uuid;

use crate::common::entities::{Identity, IdentityProvider, User};
use crate::common::AppError;

/// Data access repository for the `users` and `identities` PostgreSQL tables.
///
/// Automatically routes read queries to the slave replica pool and mutation
/// statements to the primary master pool.
#[derive(Clone)]
pub struct UsersRepository {
    db: RNPostgres,
}

impl UsersRepository {
    /// Creates a new `UsersRepository` instance wrapping an `RNPostgres` client.
    ///
    /// # Arguments
    /// * `db` - The PostgreSQL client managing master and slave connection pools.
    pub fn new(db: RNPostgres) -> Self {
        Self { db }
    }

    /// Queries the read replica for an active (non-deleted) user by email address.
    ///
    /// # Arguments
    /// * `email` - The primary email address to look up.
    ///
    /// # Returns
    /// * `Ok(Some(User))` if an active account is found.
    /// * `Ok(None)` if no matching active user exists.
    ///
    /// # Errors
    /// * `AppError::Internal` on database connection or query failure.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, status, deleted_at, created_at, updated_at FROM users WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(self.db.slave())
        .await?;
        Ok(user)
    }

    /// Queries the read replica for a user record by its primary key UUID.
    ///
    /// Returns both active and soft-deleted records.
    ///
    /// # Arguments
    /// * `id` - The user's unique identifier.
    ///
    /// # Returns
    /// * `Ok(Some(User))` if found, or `Ok(None)` if nonexistent.
    ///
    /// # Errors
    /// * `AppError::Internal` on database connection or query failure.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, status, deleted_at, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.db.slave())
        .await?;
        Ok(user)
    }

    /// Inserts a new user record on the primary master, generating a `uuidv7` primary key.
    ///
    /// # Arguments
    /// * `email` - The primary email address to register.
    ///
    /// # Returns
    /// * `Ok(Uuid)` containing the newly generated user identifier.
    ///
    /// # Errors
    /// * `AppError::Conflict` if a unique constraint violation occurs.
    /// * `AppError::Internal` on database execution failure.
    pub async fn create_user(&self, email: &str) -> Result<Uuid, AppError> {
        let id: Uuid = sqlx::query_scalar("INSERT INTO users (email) VALUES ($1) RETURNING id")
            .bind(email)
            .fetch_one(self.db.master())
            .await?;
        Ok(id)
    }

    /// Soft-deletes a user on the primary master and updates the email to an anonymized tombstone.
    ///
    /// Sets `status = 'DELETED'`, records `deleted_at = now()`, and scrubs PII from the `email` column.
    ///
    /// # Arguments
    /// * `id` - The user identifier to soft-delete.
    /// * `tombstone_email` - The anonymized placeholder string (e.g. `deleted_<uuid>@auth.local`).
    ///
    /// # Returns
    /// * `Ok(())` on successful update.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn soft_delete_user(&self, id: Uuid, tombstone_email: &str) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET status = 'DELETED', deleted_at = now(), email = $1, updated_at = now() WHERE id = $2",
        )
        .bind(tombstone_email)
        .bind(id)
        .execute(self.db.master())
        .await?;
        Ok(())
    }

    /// Queries the read replica for an identity matching a provider and identifier pair.
    ///
    /// # Arguments
    /// * `provider` - The identity verification provider (`Email` or `Google`).
    /// * `identifier` - The provider-specific identifier (email string or Google `sub` ID).
    ///
    /// # Returns
    /// * `Ok(Some(Identity))` if an identity is found, or `Ok(None)` if nonexistent.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn find_identity(
        &self,
        provider: IdentityProvider,
        identifier: &str,
    ) -> Result<Option<Identity>, AppError> {
        let identity = sqlx::query_as::<_, Identity>(
            r#"
            SELECT id, user_id, provider, identifier, credential, verified_at, created_at, updated_at
            FROM identities
            WHERE provider = $1 AND identifier = $2
            "#,
        )
        .bind(provider)
        .bind(identifier)
        .fetch_optional(self.db.slave())
        .await?;
        Ok(identity)
    }

    /// Queries the read replica for an identity associated with a specific user and provider.
    ///
    /// # Arguments
    /// * `user_id` - The user's primary key UUID.
    /// * `provider` - The identity provider to search for.
    ///
    /// # Returns
    /// * `Ok(Some(Identity))` if the user has an identity for this provider.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn find_user_identity(
        &self,
        user_id: Uuid,
        provider: IdentityProvider,
    ) -> Result<Option<Identity>, AppError> {
        let identity = sqlx::query_as::<_, Identity>(
            r#"
            SELECT id, user_id, provider, identifier, credential, verified_at, created_at, updated_at
            FROM identities
            WHERE user_id = $1 AND provider = $2
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .fetch_optional(self.db.slave())
        .await?;
        Ok(identity)
    }

    /// Inserts a new identity record on the primary master database.
    ///
    /// # Arguments
    /// * `user_id` - The owner user UUID.
    /// * `provider` - The identity provider type.
    /// * `identifier` - The unique identifier value.
    /// * `credential` - Optional Argon2id password hash (NULL for third-party OAuth).
    ///
    /// # Returns
    /// * `Ok(())` on successful insertion.
    ///
    /// # Errors
    /// * `AppError::Conflict` if `(provider, identifier)` violates unique constraints.
    /// * `AppError::Internal` on database execution failure.
    pub async fn create_identity(
        &self,
        user_id: Uuid,
        provider: IdentityProvider,
        identifier: &str,
        credential: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO identities (user_id, provider, identifier, credential, verified_at) VALUES ($1, $2, $3, $4, now())",
        )
        .bind(user_id)
        .bind(provider)
        .bind(identifier)
        .bind(credential)
        .execute(self.db.master())
        .await?;
        Ok(())
    }

    /// Updates the stored password hash for a user's `EMAIL` identity on the primary master.
    ///
    /// # Arguments
    /// * `user_id` - The user UUID.
    /// * `credential` - The updated Argon2id password hash string.
    ///
    /// # Returns
    /// * `Ok(())` on successful update.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn update_identity_credential(
        &self,
        user_id: Uuid,
        credential: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE identities SET credential = $1, updated_at = now() WHERE user_id = $2 AND provider = 'EMAIL'",
        )
        .bind(credential)
        .bind(user_id)
        .execute(self.db.master())
        .await?;
        Ok(())
    }

    /// Hard-deletes all identities belonging to a user on the primary master.
    ///
    /// # Arguments
    /// * `user_id` - The owner user UUID.
    ///
    /// # Returns
    /// * `Ok(())` upon successful deletion.
    ///
    /// # Errors
    /// * `AppError::Internal` on database execution failure.
    pub async fn delete_identities_by_user(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM identities WHERE user_id = $1")
            .bind(user_id)
            .execute(self.db.master())
            .await?;
        Ok(())
    }
}
