use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::{BoxProviderFuture, EmailProvider};
use crate::app::Config;
use crate::common::AppError;

/// Production SMTP email provider backed by the `lettre` asynchronous transport.
pub struct SmtpEmailProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
}

impl SmtpEmailProvider {
    /// Initializes an SMTP transport from application configuration.
    pub fn new(config: &Config) -> Result<Self, AppError> {
        let host = config.mail_host.as_deref().ok_or_else(|| {
            AppError::Internal("MAIL_HOST is required when EMAIL_PROVIDER=smtp".into())
        })?;

        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| AppError::Internal(format!("Invalid SMTP host: {e}")))?
            .port(config.mail_port);

        if let (Some(user), Some(pass)) = (&config.mail_user, &config.mail_pass) {
            let creds = Credentials::new(user.clone(), pass.clone());
            builder = builder.credentials(creds);
        }

        let transport = builder.build();

        let from_address = if config.mail_from_name.trim().is_empty() {
            config.mail_from.clone()
        } else {
            format!(
                "{} <{}>",
                config.mail_from_name.trim(),
                config.mail_from.trim()
            )
        };

        Ok(Self {
            transport,
            from_address,
        })
    }
}

impl EmailProvider for SmtpEmailProvider {
    fn send_email<'a>(
        &'a self,
        to: &'a str,
        subject: &'a str,
        html_body: &'a str,
    ) -> BoxProviderFuture<'a> {
        Box::pin(async move {
            let email = Message::builder()
                .from(
                    self.from_address
                        .parse()
                        .map_err(|e| AppError::Internal(format!("Invalid from address: {e}")))?,
                )
                .to(to.parse().map_err(|e| {
                    AppError::BadRequest(format!("Invalid recipient address '{to}': {e}"))
                })?)
                .subject(subject)
                .header(ContentType::TEXT_HTML)
                .body(html_body.to_string())
                .map_err(|e| AppError::Internal(format!("Failed to build email message: {e}")))?;

            self.transport
                .send(email)
                .await
                .map_err(|e| AppError::Internal(format!("SMTP dispatch failed: {e}")))?;

            tracing::info!(to = %to, subject = %subject, "SMTP email successfully delivered");
            Ok(())
        })
    }
}
