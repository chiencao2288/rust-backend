use std::future::Future;
use std::pin::Pin;

use crate::app::Config;
use crate::common::AppError;

pub mod mock;
pub mod smtp;

pub use mock::MockEmailProvider;
pub use smtp::SmtpEmailProvider;

/// Pinned, thread-safe future returned by asynchronous email providers.
pub type BoxProviderFuture<'a> = Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

/// Trait abstracting email dispatch implementations.
pub trait EmailProvider: Send + Sync {
    /// Dispatches an email with HTML body to the specified recipient.
    fn send_email<'a>(
        &'a self,
        to: &'a str,
        subject: &'a str,
        html_body: &'a str,
    ) -> BoxProviderFuture<'a>;
}

/// Factory function instantiating the configured `EmailProvider`.
pub fn create_email_provider(config: &Config) -> Result<Box<dyn EmailProvider>, AppError> {
    match config.email_provider.to_lowercase().as_str() {
        "smtp" => Ok(Box::new(SmtpEmailProvider::new(config)?)),
        _ => Ok(Box::new(MockEmailProvider)),
    }
}
