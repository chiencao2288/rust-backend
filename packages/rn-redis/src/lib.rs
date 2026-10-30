//! High-performance Redis client library.

pub mod builder;
pub mod client;
pub mod error;

pub use builder::RNRedisBuilder;
pub use client::RNRedis;
pub use error::RedisError;
