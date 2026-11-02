use std::sync::Arc;

use rn_contracts::noti::SendSmsPayload;

use super::providers::SmsProvider;
use crate::common::AppError;

/// Core service managing SMS dispatching.
pub struct SmsService {
    provider: Arc<dyn SmsProvider>,
}

impl SmsService {
    /// Creates a new `SmsService` with the specified provider.
    pub fn new(provider: Arc<dyn SmsProvider>) -> Self {
        Self { provider }
    }

    /// Dispatches an SMS message through the active provider.
    pub async fn send_sms(&self, payload: &SendSmsPayload) -> Result<(), AppError> {
        self.provider
            .send_sms(&payload.to, &payload.content)
            .await?;
        Ok(())
    }
}
