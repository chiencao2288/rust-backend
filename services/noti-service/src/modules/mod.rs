use std::sync::Arc;

use axum::Router;
use rn_contracts::noti::{
    NOTIFICATION_EXCHANGE, QUEUE_EMAIL, QUEUE_IN_APP, QUEUE_SMS, ROUTING_KEY_EMAIL,
    ROUTING_KEY_IN_APP, ROUTING_KEY_SMS,
};
use rn_rabbitmq::ExchangeKind;

use crate::app::AppState;
use crate::common::AppError;

pub mod email;
pub mod in_app;
pub mod sms;

/// Registers all HTTP endpoints across notification feature modules.
pub fn register_routes(state: AppState) -> Router {
    Router::new().merge(in_app::register(state))
}

/// Declares RabbitMQ topology (exchange, queues, bindings) and spawns consumer workers.
pub async fn start_background_workers(state: &AppState) -> Result<(), AppError> {
    let rmq = &state.rabbitmq;

    // 1. Declare Topic Exchange
    rmq.declare_exchange(NOTIFICATION_EXCHANGE, ExchangeKind::Topic)
        .await?;

    // 2. Declare and bind queues
    rmq.declare_and_bind_queue(QUEUE_EMAIL, NOTIFICATION_EXCHANGE, ROUTING_KEY_EMAIL)
        .await?;
    rmq.declare_and_bind_queue(QUEUE_SMS, NOTIFICATION_EXCHANGE, ROUTING_KEY_SMS)
        .await?;
    rmq.declare_and_bind_queue(QUEUE_IN_APP, NOTIFICATION_EXCHANGE, ROUTING_KEY_IN_APP)
        .await?;

    // 3. Initialize services
    let email_provider = email::create_email_provider(&state.config)?;
    let email_service = Arc::new(email::EmailService::new(email_provider.into()));

    let sms_provider = sms::create_sms_provider(&state.config);
    let sms_service = Arc::new(sms::SmsService::new(sms_provider.into()));

    let in_app_service = Arc::new(in_app::InAppService::new(state));

    // 4. Start background Tokio consumer tasks
    email::start_email_consumer(rmq.clone(), email_service);
    sms::start_sms_consumer(rmq.clone(), sms_service);
    in_app::start_in_app_consumer(rmq.clone(), in_app_service);

    tracing::info!("All RabbitMQ notification workers spawned successfully");
    Ok(())
}
