use std::time::Duration;

use lapin::{Connection, ConnectionProperties};

use crate::client::RNRabbitMQ;
use crate::error::RabbitMQError;

/// Builder for configuring and creating an RNRabbitMQ client.
#[derive(Debug, Clone)]
pub struct RNRabbitMQBuilder {
    url: Option<String>,
    connection_timeout: Duration,
    prefetch_count: u16,
}

impl Default for RNRabbitMQBuilder {
    fn default() -> Self {
        Self {
            url: None,
            connection_timeout: Duration::from_secs(5),
            prefetch_count: 50,
        }
    }
}

impl RNRabbitMQBuilder {
    /// Creates a default builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the AMQP broker URL (e.g. `amqp://guest:guest@127.0.0.1:5672/%2f`).
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the connection timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Sets the QoS prefetch count for message consumers.
    pub fn with_prefetch_count(mut self, count: u16) -> Self {
        self.prefetch_count = count;
        self
    }

    /// Connects to RabbitMQ, opens a default channel, and returns an initialized `RNRabbitMQ` client.
    pub async fn build(self) -> Result<RNRabbitMQ, RabbitMQError> {
        let url = self
            .url
            .ok_or_else(|| RabbitMQError::Configuration("AMQP broker URL is required".into()))?;

        let options = ConnectionProperties::default().with_connection_name("rn-service".into());

        let connection = Connection::connect(&url, options).await?;
        let channel = connection.create_channel().await?;

        Ok(RNRabbitMQ::new(connection, channel, self.prefetch_count))
    }
}
