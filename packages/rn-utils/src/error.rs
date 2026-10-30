use thiserror::Error;

/// Errors produced by cryptographic and utility operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Password hashing or verification failure.
    #[error("Password error: {0}")]
    Password(String),
}
