use rn_redis::RNRedis;

use super::entity::SessionData;
use crate::common::AppError;

const LUA_CHECK_AND_DELETE: &str = r#"
    if redis.call('GET', KEYS[1]) == ARGV[1] then
        return redis.call('DEL', KEYS[1])
    else
        return 0
    end
"#;

/// High-level session and ephemeral state repository backed by Redis Cluster.
///
/// Encapsulates key namespacing, JSON serialization, and atomic Lua script execution.
#[derive(Clone)]
pub struct SessionStore {
    redis: RNRedis,
}

impl SessionStore {
    /// Creates a new `SessionStore` wrapping an `RNRedis` client.
    ///
    /// # Arguments
    /// * `redis` - The underlying cluster-native Redis client.
    pub fn new(redis: RNRedis) -> Self {
        Self { redis }
    }

    /// Stores active user session data in Redis with a specified time-to-live.
    ///
    /// # Arguments
    /// * `token` - The plaintext access token serving as the session key identifier.
    /// * `session` - The user session data payload to serialize and cache.
    /// * `ttl_secs` - Expiration window in seconds.
    ///
    /// # Returns
    /// * `Ok(())` on successful cache write.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication failure.
    pub async fn save_session(
        &self,
        token: &str,
        session: &SessionData,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = format!("auth:session:at_{token}");
        self.redis.set_ex(&key, session, ttl_secs).await?;
        Ok(())
    }

    /// Retrieves and deserializes session data matching an access token from Redis.
    ///
    /// # Arguments
    /// * `token` - The access token key to look up.
    ///
    /// # Returns
    /// * `Ok(Some(SessionData))` if an active non-expired session exists.
    /// * `Ok(None)` if nonexistent or expired.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication or deserialization failure.
    pub async fn get_session(&self, token: &str) -> Result<Option<SessionData>, AppError> {
        let key = format!("auth:session:at_{token}");
        let session = self.redis.get::<SessionData>(&key).await?;
        Ok(session)
    }

    /// Evicts an active session from Redis cache by access token.
    ///
    /// # Arguments
    /// * `token` - The access token key to delete.
    ///
    /// # Returns
    /// * `Ok(())` upon deletion.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication failure.
    pub async fn delete_session(&self, token: &str) -> Result<(), AppError> {
        let key = format!("auth:session:at_{token}");
        self.redis.del(&key).await?;
        Ok(())
    }

    /// Stores a temporary registration OTP code in Redis with a strict expiration window.
    ///
    /// # Arguments
    /// * `email` - The target registration email address.
    /// * `otp` - The generated 6-digit numeric verification code.
    /// * `ttl_secs` - Expiration window in seconds (e.g. 300).
    ///
    /// # Returns
    /// * `Ok(())` upon write.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication failure.
    pub async fn set_register_otp(
        &self,
        email: &str,
        otp: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = format!("auth:otp:register:{email}");
        self.redis.set_ex_str(&key, otp, ttl_secs).await?;
        Ok(())
    }

    /// Atomically verifies and consumes a registration OTP code using an inline Lua script.
    ///
    /// If the supplied OTP matches the stored code, deletes the key and returns `true`.
    /// Otherwise, returns `false` without modifying the stored code.
    ///
    /// # Arguments
    /// * `email` - The target registration email address.
    /// * `input_otp` - The verification code supplied by the client.
    ///
    /// # Returns
    /// * `Ok(true)` if the code was valid and consumed.
    /// * `Ok(false)` if invalid, mismatching, or expired.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis script evaluation failure.
    pub async fn verify_and_delete_register_otp(
        &self,
        email: &str,
        input_otp: &str,
    ) -> Result<bool, AppError> {
        let key = format!("auth:otp:register:{email}");
        let is_valid: i64 = self
            .redis
            .eval(LUA_CHECK_AND_DELETE, vec![&key], vec![input_otp])
            .await?;
        Ok(is_valid == 1)
    }

    /// Stores a temporary password reset OTP code in Redis with a strict expiration window.
    ///
    /// # Arguments
    /// * `email` - The target account email address.
    /// * `otp` - The generated 6-digit numeric verification code.
    /// * `ttl_secs` - Expiration window in seconds (e.g. 300).
    ///
    /// # Returns
    /// * `Ok(())` upon write.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication failure.
    pub async fn set_reset_otp(
        &self,
        email: &str,
        otp: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = format!("auth:otp:reset_pwd:{email}");
        self.redis.set_ex_str(&key, otp, ttl_secs).await?;
        Ok(())
    }

    /// Atomically verifies and consumes a password reset OTP code using an inline Lua script.
    ///
    /// # Arguments
    /// * `email` - The target account email address.
    /// * `input_otp` - The verification code supplied by the client.
    ///
    /// # Returns
    /// * `Ok(true)` if the code was valid and consumed.
    /// * `Ok(false)` if invalid, mismatching, or expired.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis script evaluation failure.
    pub async fn verify_and_delete_reset_otp(
        &self,
        email: &str,
        input_otp: &str,
    ) -> Result<bool, AppError> {
        let key = format!("auth:otp:reset_pwd:{email}");
        let is_valid: i64 = self
            .redis
            .eval(LUA_CHECK_AND_DELETE, vec![&key], vec![input_otp])
            .await?;
        Ok(is_valid == 1)
    }

    /// Sets an account deletion cooldown flag in Redis to block immediate re-registration abuse.
    ///
    /// # Arguments
    /// * `email_hash` - The SHA-256 hash string of the deleted account email.
    /// * `ttl_secs` - Cooldown duration in seconds (e.g. 30 days = 2,592,000s).
    ///
    /// # Returns
    /// * `Ok(())` upon write.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication failure.
    pub async fn set_account_cooldown(
        &self,
        email_hash: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = format!("auth:cooldown:{email_hash}");
        self.redis.set_ex_str(&key, "1", ttl_secs).await?;
        Ok(())
    }

    /// Checks if a given email hash is currently under an active account deletion cooldown.
    ///
    /// # Arguments
    /// * `email_hash` - The SHA-256 hash string of the prospective email address.
    ///
    /// # Returns
    /// * `Ok(true)` if an active cooldown key exists in Redis.
    /// * `Ok(false)` if no cooldown key is present.
    ///
    /// # Errors
    /// * `AppError::Internal` on Redis communication failure.
    pub async fn is_in_account_cooldown(&self, email_hash: &str) -> Result<bool, AppError> {
        let key = format!("auth:cooldown:{email_hash}");
        let exists = self.redis.exists(&key).await?;
        Ok(exists)
    }
}
