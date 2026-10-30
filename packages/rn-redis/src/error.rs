use thiserror::Error;

/// Errors produced by the Redis client.
#[derive(Debug, Error)]
pub enum RedisError {
    /// Error returned by the underlying Redis driver.
    #[error("Redis driver error: {0}")]
    Driver(#[from] fred::error::Error),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
}
