use std::future::Future;
use std::pin::Pin;

use crate::app::Config;
use crate::common::AppError;

pub mod mock;

pub use mock::MockSmsProvider;

/// Pinned, thread-safe future returned by asynchronous SMS providers.
pub type BoxSmsFuture<'a> = Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

/// Trait abstracting SMS text message dispatch implementations.
pub trait SmsProvider: Send + Sync {
    /// Dispatches an SMS message to the specified recipient phone number.
    fn send_sms<'a>(&'a self, to: &'a str, content: &'a str) -> BoxSmsFuture<'a>;
}

/// Factory function instantiating the configured `SmsProvider`.
pub fn create_sms_provider(config: &Config) -> Box<dyn SmsProvider> {
    match config.sms_provider.to_lowercase().as_str() {
        "mock" => Box::new(MockSmsProvider),
        _ => Box::new(MockSmsProvider),
    }
}
