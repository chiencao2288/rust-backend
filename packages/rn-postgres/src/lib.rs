//! High-performance PostgreSQL client library with read/write splitting.

pub mod builder;
pub mod client;
pub mod error;

pub use builder::RNPostgresBuilder;
pub use client::RNPostgres;
pub use error::PostgresError;
