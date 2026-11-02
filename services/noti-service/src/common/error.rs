use rn_bootstrap::ApiError;
use thiserror::Error;

/// Standard application domain and HTTP exception error for noti-service.
#[derive(Debug, Error, ApiError)]
pub enum AppError {
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

    #[status(INTERNAL_SERVER_ERROR, code = "INTERNAL_SERVER_ERROR")]
    #[error("{0}")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<rn_postgres::PostgresError> for AppError {
    fn from(err: rn_postgres::PostgresError) -> Self {
        Self::Internal(err.to_string())
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

impl_from_internal!(
    rn_redis::RedisError,
    rn_rabbitmq::RabbitMQError,
    reqwest::Error,
);
