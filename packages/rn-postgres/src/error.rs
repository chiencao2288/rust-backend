use thiserror::Error;

/// Errors produced by the PostgreSQL client library.
#[derive(Debug, Error)]
pub enum PostgresError {
    /// Error returned by the underlying SQLx driver.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Client configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
}
