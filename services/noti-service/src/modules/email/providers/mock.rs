use super::{BoxProviderFuture, EmailProvider};

/// Development mock email provider logging sent messages to tracing output without network calls.
pub struct MockEmailProvider;

impl EmailProvider for MockEmailProvider {
    fn send_email<'a>(
        &'a self,
        to: &'a str,
        subject: &'a str,
        html_body: &'a str,
    ) -> BoxProviderFuture<'a> {
        Box::pin(async move {
            tracing::info!(
                to = %to,
                subject = %subject,
                length = html_body.len(),
                "📧 [MOCK EMAIL DISPATCHED] (Development mode - not sending actual SMTP)"
            );
            Ok(())
        })
    }
}
