use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Standard application domain and HTTP exception error.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation failed")]
    ValidationFailed {
        message: String,
        details: serde_json::Value,
    },

    #[error("Invalid email or password")]
    InvalidCredentials,

    #[error("Account is inactive or suspended")]
    AccountInactive,

    #[error("Account is banned")]
    AccountBanned,

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Returns a machine-readable, stable error code for client-side condition handling.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ValidationFailed { .. } => "VALIDATION_FAILED",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::AccountInactive => "ACCOUNT_INACTIVE",
            Self::AccountBanned => "ACCOUNT_BANNED",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Internal(_) => "INTERNAL_SERVER_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            Self::ValidationFailed { message, details } => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_FAILED".to_string(),
                message.clone(),
                Some(details.clone()),
            ),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS".to_string(),
                self.to_string(),
                None,
            ),
            Self::AccountInactive => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_INACTIVE".to_string(),
                self.to_string(),
                None,
            ),
            Self::AccountBanned => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_BANNED".to_string(),
                self.to_string(),
                None,
            ),
            Self::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST".to_string(),
                msg.clone(),
                None,
            ),
            Self::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED".to_string(),
                msg.clone(),
                None,
            ),
            Self::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN".to_string(),
                msg.clone(),
                None,
            ),
            Self::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND".to_string(),
                msg.clone(),
                None,
            ),
            Self::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT".to_string(),
                msg.clone(),
                None,
            ),
            Self::Internal(msg) => {
                tracing::error!("Internal server error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR".to_string(),
                    "Internal server error".to_string(),
                    None,
                )
            }
        };

        let mut data = json!({
            "code": code,
            "message": message,
        });

        if let Some(d) = details {
            if let Some(obj) = data.as_object_mut() {
                obj.insert("details".to_string(), d);
            }
        }

        let body = json!({
            "success": false,
            "data": data,
        });

        (status, Json(body)).into_response()
    }
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

impl From<rn_redis::RedisError> for AppError {
    fn from(err: rn_redis::RedisError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<rn_utils::CryptoError> for AppError {
    fn from(err: rn_utils::CryptoError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        Self::Internal(err.to_string())
    }
}
