use rn_bootstrap::ApiError;
use thiserror::Error;

/// Standard application domain and HTTP exception error.
#[derive(Debug, Error, ApiError)]
pub enum AppError {
    #[status(BAD_REQUEST)]
    #[error("Validation failed")]
    ValidationFailed {
        message: String,
        details: serde_json::Value,
    },

    #[status(UNAUTHORIZED)]
    #[error("Invalid email or password")]
    InvalidCredentials,

    #[status(FORBIDDEN)]
    #[error("Account is inactive or suspended")]
    AccountInactive,

    #[status(FORBIDDEN)]
    #[error("Account is banned")]
    AccountBanned,

    #[status(BAD_REQUEST)]
    #[error("{0}")]
    BadRequest(String),

    #[status(UNAUTHORIZED)]
    #[error("{0}")]
    Unauthorized(String),

    #[status(FORBIDDEN)]
    #[error("{0}")]
    Forbidden(String),

    #[status(NOT_FOUND)]
    #[error("{0}")]
    NotFound(String),

    #[status(CONFLICT)]
    #[error("{0}")]
    Conflict(String),

    #[status(TOO_MANY_REQUESTS)]
    #[error("{0}")]
    TooManyRequests(String),

    #[status(INTERNAL_SERVER_ERROR, code = "INTERNAL_SERVER_ERROR")]
    #[error("{0}")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if rn_postgres::RNPostgres::is_unique_violation(&err) {
            return Self::Conflict("Resource already exists".into());
        }
        Self::Internal(err.to_string())
    }
}

impl From<rn_postgres::PostgresError> for AppError {
    fn from(err: rn_postgres::PostgresError) -> Self {
        match err {
            rn_postgres::PostgresError::Database(e) => Self::from(e),
            rn_postgres::PostgresError::Configuration(msg) => Self::Internal(msg),
        }
    }
}

macro_rules! impl_from_internal {
    ($($err:ty),* $(,)?) => {
        $(
            impl From<$err> for AppError {
                fn from(err: $err) -> Self {
                    Self::Internal(err.to_string())
                }
            }
        )*
    };
}

impl_from_internal!(rn_redis::RedisError, rn_utils::CryptoError, reqwest::Error);
