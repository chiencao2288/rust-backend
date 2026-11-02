//! High-performance RabbitMQ client library for rn-services.

pub mod builder;
pub mod client;
pub mod error;

pub use builder::RNRabbitMQBuilder;
pub use client::RNRabbitMQ;
pub use error::RabbitMQError;
pub use lapin::{message::Delivery, Consumer, ExchangeKind};
