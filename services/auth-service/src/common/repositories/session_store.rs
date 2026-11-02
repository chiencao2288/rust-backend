use rn_redis::RNRedis;

use crate::common::constant::{
    OTP_MAX_FAILED_VERIFICATIONS, REDIS_KEY_DELETED_ACCOUNT_COOLDOWN, REDIS_KEY_OTP_ATTEMPTS,
    REDIS_KEY_OTP_CODE, REDIS_KEY_OTP_COOLDOWN, REDIS_KEY_OTP_RATE, REDIS_KEY_SESSION,
};
use crate::common::entities::SessionData;
use crate::common::AppError;

/// Lua script for atomic OTP verification with brute-force attempt tracking.
///
/// Returns:
///  1: OTP matches; consumed and deleted.
/// -2: Too many failed verification attempts; OTP permanently invalidated and deleted.
///  0: Incorrect OTP; attempt counter incremented.
/// -1: OTP expired or does not exist.
const LUA_VERIFY_AND_CONSUME_OTP: &str = r#"
    local stored_otp = redis.call('GET', KEYS[1])
    if not stored_otp then
        return -1
    end

    local attempts = redis.call('INCR', KEYS[2])
    if attempts == 1 then
        redis.call('EXPIRE', KEYS[2], tonumber(ARGV[3]))
    end

    if stored_otp == ARGV[1] then
        redis.call('DEL', KEYS[1])
        redis.call('DEL', KEYS[2])
        return 1
    end

    if attempts >= tonumber(ARGV[2]) then
        redis.call('DEL', KEYS[1])
        redis.call('DEL', KEYS[2])
        return -2
    end

    return 0
"#;

/// Result of an OTP verification attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtpVerifyResult {
    /// OTP code is valid and has been atomically consumed.
    Valid,
    /// OTP code is incorrect or expired.
    Invalid,
    /// Maximum verification attempts exceeded; the OTP has been destroyed.
    AttemptsExceeded,
}

/// High-level session and ephemeral state repository backed by Redis Cluster.
#[derive(Clone)]
pub struct SessionStore {
    redis: RNRedis,
}

impl SessionStore {
    /// Creates a new `SessionStore` wrapping an `RNRedis` client.
    pub fn new(redis: RNRedis) -> Self {
        Self { redis }
    }

    /// Stores active user session data in Redis with a specified time-to-live.
    pub async fn save_session(
        &self,
        token: &str,
        session: &SessionData,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = format!("{REDIS_KEY_SESSION}:{token}");
        self.redis.set_ex(&key, session, ttl_secs).await?;
        Ok(())
    }

    /// Retrieves and deserializes session data matching an access token from Redis.
    pub async fn get_session(&self, token: &str) -> Result<Option<SessionData>, AppError> {
        let key = format!("{REDIS_KEY_SESSION}:{token}");
        let session = self.redis.get::<SessionData>(&key).await?;
        Ok(session)
    }

    /// Evicts an active session from Redis cache by access token.
    pub async fn delete_session(&self, token: &str) -> Result<(), AppError> {
        let key = format!("{REDIS_KEY_SESSION}:{token}");
        self.redis.del(&key).await?;
        Ok(())
    }

    /// Checks rate limits and applies a cooldown before an OTP is generated:
    /// 1. Cooldown check: rejects requests sent within `cooldown_secs` of the last request.
    /// 2. Hourly rate limit: rejects requests exceeding `max_attempts` within `window_secs`.
    pub async fn check_and_set_otp_rate_limit(
        &self,
        action: &str,
        email: &str,
        cooldown_secs: u64,
        max_attempts: u32,
        window_secs: u64,
    ) -> Result<(), AppError> {
        let cooldown_key = format!("{REDIS_KEY_OTP_COOLDOWN}:{action}:{email}");
        if self.redis.exists(&cooldown_key).await? {
            let remaining = self.redis.ttl(&cooldown_key).await?.max(1);
            return Err(AppError::TooManyRequests(format!(
                "Please wait {remaining} seconds before requesting another code"
            )));
        }

        let rate_key = format!("{REDIS_KEY_OTP_RATE}:{action}:{email}");
        let count = self.redis.incr(&rate_key).await?;
        if count == 1 {
            self.redis.expire(&rate_key, window_secs as i64).await?;
        }
        if count > i64::from(max_attempts) {
            return Err(AppError::TooManyRequests(
                "Too many verification requests. Please try again later.".into(),
            ));
        }

        self.redis
            .set_ex_str(&cooldown_key, "1", cooldown_secs)
            .await?;
        Ok(())
    }

    /// Stores an OTP verification code in Redis and resets the failed attempts counter.
    pub async fn set_otp(
        &self,
        action: &str,
        email: &str,
        otp: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let code_key = format!("{REDIS_KEY_OTP_CODE}:{action}:{email}");
        let attempts_key = format!("{REDIS_KEY_OTP_ATTEMPTS}:{action}:{email}");

        self.redis.set_ex_str(&code_key, otp, ttl_secs).await?;
        self.redis.del(&attempts_key).await?;
        Ok(())
    }

    /// Sets a registration OTP code.
    pub async fn set_register_otp(
        &self,
        email: &str,
        otp: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        self.set_otp("register", email, otp, ttl_secs).await
    }

    /// Sets a password reset OTP code.
    pub async fn set_reset_otp(
        &self,
        email: &str,
        otp: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        self.set_otp("reset_password", email, otp, ttl_secs).await
    }

    /// Atomically verifies and consumes an OTP code, tracking and bounding failed attempts.
    pub async fn verify_and_delete_otp(
        &self,
        action: &str,
        email: &str,
        input_otp: &str,
        max_attempts: u32,
        ttl_secs: u64,
    ) -> Result<OtpVerifyResult, AppError> {
        let code_key = format!("{REDIS_KEY_OTP_CODE}:{action}:{email}");
        let attempts_key = format!("{REDIS_KEY_OTP_ATTEMPTS}:{action}:{email}");

        let max_attempts_str = max_attempts.to_string();
        let ttl_str = ttl_secs.to_string();

        let res: i64 = self
            .redis
            .eval(
                LUA_VERIFY_AND_CONSUME_OTP,
                vec![&code_key, &attempts_key],
                vec![input_otp, &max_attempts_str, &ttl_str],
            )
            .await?;

        match res {
            1 => Ok(OtpVerifyResult::Valid),
            -2 => Ok(OtpVerifyResult::AttemptsExceeded),
            _ => Ok(OtpVerifyResult::Invalid),
        }
    }

    /// Verifies and deletes a registration OTP code.
    pub async fn verify_and_delete_register_otp(
        &self,
        email: &str,
        input_otp: &str,
    ) -> Result<OtpVerifyResult, AppError> {
        self.verify_and_delete_otp(
            "register",
            email,
            input_otp,
            OTP_MAX_FAILED_VERIFICATIONS,
            300,
        )
        .await
    }

    /// Verifies and deletes a password reset OTP code.
    pub async fn verify_and_delete_reset_otp(
        &self,
        email: &str,
        input_otp: &str,
    ) -> Result<OtpVerifyResult, AppError> {
        self.verify_and_delete_otp(
            "reset_password",
            email,
            input_otp,
            OTP_MAX_FAILED_VERIFICATIONS,
            300,
        )
        .await
    }

    /// Sets an account deletion cooldown flag in Redis.
    pub async fn set_account_cooldown(
        &self,
        email_hash: &str,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        let key = format!("{REDIS_KEY_DELETED_ACCOUNT_COOLDOWN}:{email_hash}");
        self.redis.set_ex_str(&key, "1", ttl_secs).await?;
        Ok(())
    }

    /// Checks if a given email hash is currently under an active account deletion cooldown.
    pub async fn is_in_account_cooldown(&self, email_hash: &str) -> Result<bool, AppError> {
        let key = format!("{REDIS_KEY_DELETED_ACCOUNT_COOLDOWN}:{email_hash}");
        let exists = self.redis.exists(&key).await?;
        Ok(exists)
    }
}
