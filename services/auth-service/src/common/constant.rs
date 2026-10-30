//! Application-wide constants.

/// Masked placeholder string used when sanitizing sensitive credentials in logs or dev debug payloads.
pub const MASKED_STRING: &str = "******";

/// Field names that must be sanitized across request bodies and debug outputs.
pub const SENSITIVE_FIELD_KEYS: &[&str] = &[
    "password",
    "new_password",
    "token",
    "access_token",
    "refresh_token",
    "credential",
    "id_token",
    "client_secret",
];

/// Default access token session lifetime in seconds (30 minutes).
pub const DEFAULT_SESSION_TTL_SECS: u64 = 1800;

/// Default refresh token lifetime in days (30 days).
pub const DEFAULT_REFRESH_TOKEN_TTL_DAYS: i64 = 30;

/// Temporary OTP code expiration lifetime in seconds (5 minutes).
pub const OTP_TTL_SECS: u64 = 300;

/// Anti-abuse account deletion cooldown lifetime in seconds (30 days).
pub const ACCOUNT_COOLDOWN_SECS: u64 = 30 * 86400;
