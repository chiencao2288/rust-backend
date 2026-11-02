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

/// Minimum cooldown between consecutive OTP requests for the same email (60 seconds).
pub const OTP_COOLDOWN_SEC: u64 = 60;

/// Maximum number of OTP generation requests permitted per window (5 requests).
pub const OTP_MAX_HOURLY_ATTEMPTS: u32 = 5;

/// Window duration for tracking hourly OTP generation rate limits (1 hour).
pub const OTP_RATE_LIMIT_WINDOW_SEC: u64 = 3600;

/// Maximum number of failed OTP verification attempts before the code is permanently invalidated.
pub const OTP_MAX_FAILED_VERIFICATIONS: u32 = 5;

/// Anti-abuse account deletion cooldown lifetime in seconds (30 days).
pub const ACCOUNT_COOLDOWN_SECS: u64 = 30 * 86400;

/// Redis key namespace for active user sessions: `auth:session:{token}`
pub const REDIS_KEY_SESSION: &str = "auth:session";

/// Redis key namespace for OTP verification codes: `auth:otp:code:{action}:{email}`
pub const REDIS_KEY_OTP_CODE: &str = "auth:otp:code";

/// Redis key namespace for OTP resend cooldown: `auth:otp:cooldown:{action}:{email}`
pub const REDIS_KEY_OTP_COOLDOWN: &str = "auth:otp:cooldown";

/// Redis key namespace for OTP generation rate counter: `auth:otp:rate:{action}:{email}`
pub const REDIS_KEY_OTP_RATE: &str = "auth:otp:rate";

/// Redis key namespace for failed OTP attempts counter: `auth:otp:attempts:{action}:{email}`
pub const REDIS_KEY_OTP_ATTEMPTS: &str = "auth:otp:attempts";

/// Redis key namespace for deleted account cooldown: `auth:cooldown:deleted_account:{email_hash}`
pub const REDIS_KEY_DELETED_ACCOUNT_COOLDOWN: &str = "auth:cooldown:deleted_account";
