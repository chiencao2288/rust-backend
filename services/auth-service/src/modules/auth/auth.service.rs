use chrono::{Duration, Utc};
use rn_utils::{
    generate_numeric_otp, generate_secure_token, hash_password, sha256_hash, verify_password,
};
use serde::Deserialize;
use uuid::Uuid;

use super::dto::LoginDto;
use crate::app::AppState;
use crate::common::constant::{
    ACCOUNT_COOLDOWN_SECS, OTP_COOLDOWN_SEC, OTP_MAX_HOURLY_ATTEMPTS, OTP_RATE_LIMIT_WINDOW_SEC,
    OTP_TTL_SECS,
};
use crate::common::entities::{AuthTokens, IdentityProvider, SessionData, UserStatus};
use crate::common::repositories::{
    OtpVerifyResult, SessionStore, TokensRepository, UsersRepository,
};
use crate::common::AppError;
#[derive(Debug, Deserialize)]
struct GoogleTokenInfo {
    sub: String,
    email: Option<String>,
    aud: Option<String>,
}

/// Core authentication service managing business use cases.
///
/// Coordinates identity verification, credential hashing, token rotation,
/// account lifecycle operations, and cache invalidation across the system.
#[derive(Clone)]
pub struct AuthService {
    users_repo: UsersRepository,
    tokens_repo: TokensRepository,
    session_store: SessionStore,
    http_client: reqwest::Client,
    session_ttl_sec: u64,
    refresh_token_ttl_day: i64,
    google_client_id: Option<String>,
    google_oauth_url: String,
}

impl AuthService {
    /// Creates a new `AuthService` initialized from the shared application state.
    pub fn new(state: AppState) -> Self {
        Self {
            users_repo: state.users_repo,
            tokens_repo: state.tokens_repo,
            session_store: state.session_store,
            http_client: state.http_client,
            session_ttl_sec: state.config.session_ttl_sec,
            refresh_token_ttl_day: state.config.refresh_token_ttl_day,
            google_client_id: state.config.google_client_id,
            google_oauth_url: state.config.google_oauth_url,
        }
    }

    /// Handles request for an email verification code prior to account registration.
    ///
    /// Validates email format, verifies that the email is not under a 30-day deletion
    /// cooldown, ensures the email has not already been registered, and stores a
    /// 6-digit numeric OTP in Redis with a 5-minute TTL.
    ///
    /// # Arguments
    /// * `email` - The prospective user's email address.
    ///
    /// # Returns
    /// * `Ok(())` on successful OTP generation and dispatch.
    ///
    /// # Errors
    /// * `AppError::BadRequest` if the email string is empty or invalid.
    /// * `AppError::Forbidden` if the email address is currently under a deletion cooldown.
    /// * `AppError::Conflict` if an active account with this email address already exists.
    pub async fn request_register_otp(&self, email: &str) -> Result<(), AppError> {
        let email = email.trim().to_lowercase();

        let email_hash = sha256_hash(&email);
        if self
            .session_store
            .is_in_account_cooldown(&email_hash)
            .await?
        {
            return Err(AppError::Forbidden(
                "Account with this email was recently deleted. Please try again later.".into(),
            ));
        }

        if self.users_repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("Email is already registered".into()));
        }
        self.session_store
            .check_and_set_otp_rate_limit(
                "register",
                &email,
                OTP_COOLDOWN_SEC,
                OTP_MAX_HOURLY_ATTEMPTS,
                OTP_RATE_LIMIT_WINDOW_SEC,
            )
            .await?;

        let otp = generate_numeric_otp(6);
        self.session_store
            .set_register_otp(&email, &otp, OTP_TTL_SECS)
            .await?;

        tracing::info!(email = %email, otp = %otp, "Registration OTP generated");
        Ok(())
    }

    /// Validates a pre-registration OTP code, creates user and credential records, and issues tokens.
    ///
    /// Executes an atomic Lua script on Redis to verify and immediately consume the OTP.
    /// Hashes the plaintext password with Argon2id using a cryptographically secure salt,
    /// persists the user record with `ACTIVE` status, and marks the identity as verified.
    ///
    /// # Arguments
    /// * `email` - The user's email address.
    /// * `password` - Plaintext password (must be at least 8 characters long).
    /// * `otp` - The 6-digit verification code sent to the user's email.
    ///
    /// # Returns
    /// * `Ok(AuthTokens)` containing the newly minted access token and refresh token.
    ///
    /// # Errors
    /// * `AppError::BadRequest` if the password is under 8 characters or the OTP is invalid/expired.
    /// * `AppError::Conflict` if a concurrent request registered the email in the interim.
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        otp: &str,
    ) -> Result<AuthTokens, AppError> {
        let email = email.trim().to_lowercase();
        match self
            .session_store
            .verify_and_delete_register_otp(&email, otp.trim())
            .await?
        {
            OtpVerifyResult::Valid => {}
            OtpVerifyResult::AttemptsExceeded => {
                return Err(AppError::BadRequest(
                    "Too many failed verification attempts. Verification code has been invalidated.".into(),
                ));
            }
            OtpVerifyResult::Invalid => {
                return Err(AppError::BadRequest("Invalid or expired OTP".into()));
            }
        }

        let password_hash = hash_password(password)?;
        let user_id = self.users_repo.create_user(&email).await?;
        self.users_repo
            .create_identity(
                user_id,
                IdentityProvider::Email,
                &email,
                Some(&password_hash),
            )
            .await?;

        self.issue_tokens(user_id, &email).await
    }

    /// Authenticates a user using email/password credentials or a verified Google ID token.
    ///
    /// For password logins: looks up the user identity, checks account status, and compares
    /// the plaintext password against the Argon2id hash.
    /// For Google logins: validates the ID token with Google's tokeninfo endpoint, matches
    /// the Google `sub` identifier, performs automatic account linking if an email account
    /// already exists, or creates a new user on first sign-in.
    ///
    /// # Arguments
    /// * `dto` - The polymorphic `LoginDto` payload (either `Email` or `Google`).
    ///
    /// # Returns
    /// * `Ok(AuthTokens)` with a fresh access token and refresh token.
    ///
    /// # Errors
    /// * `AppError::Unauthorized` if credentials or the Google ID token is invalid.
    /// * `AppError::AccountInactive` if the account is suspended or banned.
    /// * `AppError::NotFound` if the referenced account does not exist.
    pub async fn login(&self, dto: LoginDto) -> Result<AuthTokens, AppError> {
        match dto {
            LoginDto::Email { email, password } => {
                let email = email.trim().to_lowercase();
                let user = self
                    .users_repo
                    .find_by_email(&email)
                    .await?
                    .ok_or(AppError::InvalidCredentials)?;

                if user.status != UserStatus::Active {
                    return Err(AppError::AccountInactive);
                }

                let identity = self
                    .users_repo
                    .find_user_identity(user.id, IdentityProvider::Email)
                    .await?
                    .ok_or(AppError::InvalidCredentials)?;

                let hash = identity.credential.ok_or(AppError::InvalidCredentials)?;
                if !verify_password(&password, &hash)? {
                    return Err(AppError::InvalidCredentials);
                }

                self.issue_tokens(user.id, &email).await
            }

            LoginDto::Google { id_token } => {
                let (sub, email) = self.verify_google_token(&id_token).await?;

                let user_id = if let Some(identity) = self
                    .users_repo
                    .find_identity(IdentityProvider::Google, &sub)
                    .await?
                {
                    let user = self
                        .users_repo
                        .find_by_id(identity.user_id)
                        .await?
                        .ok_or(AppError::NotFound("User not found".into()))?;

                    if user.status != UserStatus::Active {
                        return Err(AppError::AccountInactive);
                    }
                    user.id
                } else if let Some(user) = self.users_repo.find_by_email(&email).await? {
                    if user.status != UserStatus::Active {
                        return Err(AppError::AccountInactive);
                    }
                    self.users_repo
                        .create_identity(user.id, IdentityProvider::Google, &sub, None)
                        .await?;
                    user.id
                } else {
                    let new_id = self.users_repo.create_user(&email).await?;
                    self.users_repo
                        .create_identity(new_id, IdentityProvider::Google, &sub, None)
                        .await?;
                    new_id
                };

                self.issue_tokens(user_id, &email).await
            }
        }
    }

    /// Performs token rotation, invalidating the old refresh token and issuing a new pair.
    ///
    /// Computes the SHA-256 hash of the presented refresh token, looks up the record
    /// in PostgreSQL to confirm it is valid and non-expired, checks that the user account
    /// remains active, hard-deletes the old refresh token, and creates a fresh token pair.
    ///
    /// # Arguments
    /// * `refresh_token` - The plaintext refresh token presented by the client.
    ///
    /// # Returns
    /// * `Ok(AuthTokens)` containing a new access token and a rotated refresh token.
    ///
    /// # Errors
    /// * `AppError::Unauthorized` if the refresh token is missing, expired, or revoked.
    /// * `AppError::AccountInactive` if the user account is suspended or banned.
    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthTokens, AppError> {
        let token_hash = sha256_hash(refresh_token.trim());
        let token_record = self
            .tokens_repo
            .find_valid_refresh_token(&token_hash)
            .await?
            .ok_or(AppError::Unauthorized(
                "Invalid or expired refresh token".into(),
            ))?;

        let user = self
            .users_repo
            .find_by_id(token_record.user_id)
            .await?
            .ok_or(AppError::NotFound("User not found".into()))?;

        if user.status != UserStatus::Active {
            return Err(AppError::AccountInactive);
        }

        self.tokens_repo
            .delete_refresh_token_by_id(token_record.id)
            .await?;
        self.issue_tokens(user.id, &user.email).await
    }

    /// Terminates an active user session and hard-deletes the associated refresh token.
    ///
    /// Immediately evicts the session key from Redis to invalidate the access token,
    /// and hard-deletes the matching refresh token record from PostgreSQL.
    ///
    /// # Arguments
    /// * `access_token` - The active access token to evict from Redis cache.
    /// * `refresh_token` - The refresh token to purge from PostgreSQL.
    ///
    /// # Returns
    /// * `Ok(())` upon successful termination.
    pub async fn logout(&self, access_token: &str, refresh_token: &str) -> Result<(), AppError> {
        self.session_store
            .delete_session(access_token.trim())
            .await?;
        let token_hash = sha256_hash(refresh_token.trim());
        self.tokens_repo
            .delete_refresh_token_by_hash(&token_hash)
            .await?;
        Ok(())
    }

    /// Generates and sends a temporary password reset code.
    ///
    /// Looks up the email address. If an active user exists, generates a 6-digit numeric OTP,
    /// stores it in Redis with a 5-minute TTL, and logs the action.
    ///
    /// # Arguments
    /// * `email` - Target user email address.
    ///
    /// # Returns
    /// * `Ok(())` unconditionally to prevent email enumeration attacks.
    pub async fn request_password_reset_otp(&self, email: &str) -> Result<(), AppError> {
        let email = email.trim().to_lowercase();
        if self.users_repo.find_by_email(&email).await?.is_some() {
            self.session_store
                .check_and_set_otp_rate_limit(
                    "reset_password",
                    &email,
                    OTP_COOLDOWN_SEC,
                    OTP_MAX_HOURLY_ATTEMPTS,
                    OTP_RATE_LIMIT_WINDOW_SEC,
                )
                .await?;

            let otp = generate_numeric_otp(6);
            self.session_store
                .set_reset_otp(&email, &otp, OTP_TTL_SECS)
                .await?;
        }
        Ok(())
    }

    /// Verifies password reset OTP, updates password credentials, and revokes all active sessions.
    ///
    /// Atomically consumes the reset OTP from Redis. If the user previously authenticated
    /// exclusively through third-party OAuth, automatically links an `EMAIL` identity.
    /// Purges all existing refresh tokens in PostgreSQL to log out other devices.
    ///
    /// # Arguments
    /// * `email` - The target account email address.
    /// * `otp` - The 6-digit verification code.
    /// * `new_password` - New plaintext password (minimum 8 characters).
    ///
    /// # Returns
    /// * `Ok(())` on successful password update.
    ///
    /// # Errors
    /// * `AppError::BadRequest` if the new password is under 8 characters or the OTP is invalid/expired.
    /// * `AppError::NotFound` if the user record does not exist.
    /// * `AppError::AccountInactive` if the user is suspended or banned.
    pub async fn reset_password(
        &self,
        email: &str,
        otp: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let email = email.trim().to_lowercase();
        match self
            .session_store
            .verify_and_delete_reset_otp(&email, otp.trim())
            .await?
        {
            OtpVerifyResult::Valid => {}
            OtpVerifyResult::AttemptsExceeded => {
                return Err(AppError::BadRequest(
                    "Too many failed verification attempts. Verification code has been invalidated.".into(),
                ));
            }
            OtpVerifyResult::Invalid => {
                return Err(AppError::BadRequest("Invalid or expired OTP".into()));
            }
        }

        let user = self
            .users_repo
            .find_by_email(&email)
            .await?
            .ok_or(AppError::NotFound("User not found".into()))?;

        if user.status != UserStatus::Active {
            return Err(AppError::AccountInactive);
        }

        let password_hash = hash_password(new_password)?;

        if self
            .users_repo
            .find_user_identity(user.id, IdentityProvider::Email)
            .await?
            .is_some()
        {
            self.users_repo
                .update_identity_credential(user.id, &password_hash)
                .await?;
        } else {
            self.users_repo
                .create_identity(
                    user.id,
                    IdentityProvider::Email,
                    &email,
                    Some(&password_hash),
                )
                .await?;
        }

        self.tokens_repo
            .delete_all_refresh_tokens_for_user(user.id)
            .await?;
        Ok(())
    }

    /// Soft-deletes a user account, anonymizes PII email, wipes tokens, and establishes a 30-day cooldown.
    ///
    /// Validates the active access token, verifies the user is not banned, updates the
    /// user status to `DELETED`, sets `deleted_at = now()`, anonymizes the email to
    /// `deleted_<uuid>@auth.local`, deletes all authentication identities, deletes all
    /// refresh tokens, and registers a 30-day re-registration cooldown in Redis.
    ///
    /// # Arguments
    /// * `access_token` - The caller's active bearer token.
    ///
    /// # Returns
    /// * `Ok(())` on successful deletion.
    ///
    /// # Errors
    /// * `AppError::Unauthorized` if the token is invalid or expired.
    /// * `AppError::AccountBanned` if a banned account attempts deletion.
    pub async fn delete_account(&self, access_token: &str) -> Result<(), AppError> {
        let session = self.verify_token(access_token).await?;
        let user = self
            .users_repo
            .find_by_id(session.user_id)
            .await?
            .ok_or(AppError::NotFound("User not found".into()))?;

        if user.status == UserStatus::Banned {
            return Err(AppError::AccountBanned);
        }

        let tombstone_email = format!("deleted_{}@auth.local", session.user_id);
        self.users_repo
            .soft_delete_user(session.user_id, &tombstone_email)
            .await?;
        self.users_repo
            .delete_identities_by_user(session.user_id)
            .await?;
        self.tokens_repo
            .delete_all_refresh_tokens_for_user(session.user_id)
            .await?;
        self.session_store
            .delete_session(access_token.trim())
            .await?;

        let email_hash = sha256_hash(&session.email.to_lowercase());
        self.session_store
            .set_account_cooldown(&email_hash, ACCOUNT_COOLDOWN_SECS)
            .await?;

        tracing::warn!(user_id = %session.user_id, "User account soft-deleted");
        Ok(())
    }

    /// Verifies an access token session directly from in-memory Redis cache with sub-millisecond latency.
    ///
    /// # Arguments
    /// * `token` - The access token string to look up.
    ///
    /// # Returns
    /// * `Ok(SessionData)` containing user ID and email if the session is active.
    ///
    /// # Errors
    /// * `AppError::Unauthorized` if the token does not exist or has expired in Redis.
    pub async fn verify_token(&self, token: &str) -> Result<SessionData, AppError> {
        self.session_store
            .get_session(token.trim())
            .await?
            .ok_or(AppError::Unauthorized(
                "Invalid or expired access token".into(),
            ))
    }

    /// Generates and persists a new access token and refresh token pair.
    async fn issue_tokens(&self, user_id: Uuid, email: &str) -> Result<AuthTokens, AppError> {
        let access_token = generate_secure_token(32);
        let refresh_token = generate_secure_token(32);

        let session = SessionData {
            user_id,
            email: email.to_string(),
        };

        self.session_store
            .save_session(&access_token, &session, self.session_ttl_sec)
            .await?;

        let token_hash = sha256_hash(&refresh_token);
        let expires_at = Utc::now() + Duration::days(self.refresh_token_ttl_day);
        self.tokens_repo
            .create_refresh_token(user_id, &token_hash, expires_at)
            .await?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            token_type: "Bearer".into(),
            expires_in: self.session_ttl_sec,
        })
    }

    /// Validates a Google ID token with Google's public verification endpoint.
    async fn verify_google_token(&self, id_token: &str) -> Result<(String, String), AppError> {
        let info = self
            .http_client
            .get(&self.google_oauth_url)
            .query(&[("id_token", id_token.trim())])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| AppError::Unauthorized(format!("Invalid Google token: {e}")))?
            .json::<GoogleTokenInfo>()
            .await
            .map_err(|e| AppError::Unauthorized(format!("Malformed Google response: {e}")))?;

        if let Some(expected_aud) = &self.google_client_id {
            if info.aud.as_deref() != Some(expected_aud) {
                return Err(AppError::Unauthorized("Google audience mismatch".into()));
            }
        }

        let email = info
            .email
            .ok_or_else(|| AppError::BadRequest("Google profile missing email".into()))?
            .trim()
            .to_lowercase();

        Ok((info.sub, email))
    }
}
