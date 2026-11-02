use thiserror::Error;

/// Errors produced by the RabbitMQ client.
#[derive(Debug, Error)]
pub enum RabbitMQError {
    /// Error returned by the underlying Lapin AMQP driver.
    #[error("RabbitMQ driver error: {0}")]
    Driver(#[from] lapin::Error),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
}
