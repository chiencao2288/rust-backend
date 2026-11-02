use super::{BoxSmsFuture, SmsProvider};

/// Development mock SMS provider logging dispatched text messages to tracing output.
pub struct MockSmsProvider;

impl SmsProvider for MockSmsProvider {
    fn send_sms<'a>(&'a self, to: &'a str, content: &'a str) -> BoxSmsFuture<'a> {
        Box::pin(async move {
            tracing::info!(
                to = %to,
                content = %content,
                "📱 [MOCK SMS DISPATCHED] (Development mode - ready for Phone OTP integration)"
            );
            Ok(())
        })
    }
}
