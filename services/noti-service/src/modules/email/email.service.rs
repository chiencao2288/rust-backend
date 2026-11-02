use std::sync::Arc;

use rn_contracts::noti::SendEmailPayload;

use super::providers::EmailProvider;
use super::template::TemplateEngine;
use crate::common::AppError;

/// Core service orchestrating email template rendering and delivery dispatch.
pub struct EmailService {
    provider: Arc<dyn EmailProvider>,
}

impl EmailService {
    /// Creates a new `EmailService` with the specified provider.
    pub fn new(provider: Arc<dyn EmailProvider>) -> Self {
        Self { provider }
    }

    /// Renders the target HTML template and delivers the email through the active provider.
    pub async fn send_email(&self, payload: &SendEmailPayload) -> Result<(), AppError> {
        let html_body = TemplateEngine::render(&payload.template, &payload.params)?;
        self.provider
            .send_email(&payload.to, &payload.subject, &html_body)
            .await?;
        Ok(())
    }
}
