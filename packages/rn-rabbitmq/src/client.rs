use std::sync::Arc;

use lapin::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicQosOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, Consumer, ExchangeKind};
use serde::Serialize;

use crate::builder::RNRabbitMQBuilder;
use crate::error::RabbitMQError;

/// RabbitMQ client wrapper providing high-level AMQP operations.
#[derive(Clone)]
pub struct RNRabbitMQ {
    connection: Arc<Connection>,
    channel: Channel,
    prefetch_count: u16,
}

impl RNRabbitMQ {
    /// Creates a new `RNRabbitMQ` instance.
    pub fn new(connection: Connection, channel: Channel, prefetch_count: u16) -> Self {
        Self {
            connection: Arc::new(connection),
            channel,
            prefetch_count,
        }
    }

    /// Creates a new builder instance.
    pub fn builder() -> RNRabbitMQBuilder {
        RNRabbitMQBuilder::new()
    }

    /// Connects to RabbitMQ with default configuration.
    pub async fn connect(url: &str) -> Result<Self, RabbitMQError> {
        Self::builder().with_url(url).build().await
    }

    /// Returns a reference to the active AMQP connection.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Returns a reference to the primary AMQP channel.
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Creates a dedicated new AMQP channel from the connection.
    pub async fn create_channel(&self) -> Result<Channel, RabbitMQError> {
        let channel = self.connection.create_channel().await?;
        channel
            .basic_qos(self.prefetch_count, BasicQosOptions::default())
            .await?;
        Ok(channel)
    }

    /// Declares a durable exchange if it does not already exist.
    pub async fn declare_exchange(
        &self,
        exchange: &str,
        kind: ExchangeKind,
    ) -> Result<(), RabbitMQError> {
        let options = ExchangeDeclareOptions {
            durable: true,
            ..Default::default()
        };
        self.channel
            .exchange_declare(exchange, kind, options, FieldTable::default())
            .await?;
        Ok(())
    }

    /// Declares a durable queue and binds it to an exchange with a routing key.
    pub async fn declare_and_bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
    ) -> Result<(), RabbitMQError> {
        let q_options = QueueDeclareOptions {
            durable: true,
            ..Default::default()
        };
        self.channel
            .queue_declare(queue, q_options, FieldTable::default())
            .await?;

        self.channel
            .queue_bind(
                queue,
                exchange,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }

    /// Serializes a payload to JSON and publishes it to an exchange with persistent delivery mode.
    pub async fn publish_json<T: Serialize>(
        &self,
        exchange: &str,
        routing_key: &str,
        payload: &T,
    ) -> Result<(), RabbitMQError> {
        let bytes = serde_json::to_vec(payload)?;
        let properties = BasicProperties::default()
            .with_content_type("application/json".into())
            .with_delivery_mode(2); // Persistent

        self.channel
            .basic_publish(
                exchange,
                routing_key,
                BasicPublishOptions::default(),
                &bytes,
                properties,
            )
            .await?
            .await?; // Await publish confirmation

        Ok(())
    }

    /// Creates a message consumer on a specific queue with basic QoS prefetching.
    pub async fn create_consumer(
        &self,
        queue: &str,
        consumer_tag: &str,
    ) -> Result<Consumer, RabbitMQError> {
        let channel = self.create_channel().await?;
        let options = BasicConsumeOptions {
            no_ack: false,
            ..Default::default()
        };

        let consumer = channel
            .basic_consume(queue, consumer_tag, options, FieldTable::default())
            .await?;

        Ok(consumer)
    }
}
