use std::sync::Arc;

use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicNackOptions};
use rn_contracts::noti::{SendEmailPayload, QUEUE_EMAIL};
use rn_rabbitmq::RNRabbitMQ;

use super::service::EmailService;

/// Starts a background Tokio task consuming email delivery messages from RabbitMQ.
pub fn start_email_consumer(rabbitmq: RNRabbitMQ, email_service: Arc<EmailService>) {
    tokio::spawn(async move {
        tracing::info!("Starting RabbitMQ email consumer on queue `{QUEUE_EMAIL}`");

        let mut consumer = match rabbitmq
            .create_consumer(QUEUE_EMAIL, "noti-email-worker")
            .await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to register RabbitMQ email consumer: {e}");
                return;
            }
        };

        while let Some(delivery_result) = consumer.next().await {
            match delivery_result {
                Ok(delivery) => match serde_json::from_slice::<SendEmailPayload>(&delivery.data) {
                    Ok(payload) => {
                        if let Err(e) = email_service.send_email(&payload).await {
                            tracing::error!(to = %payload.to, "Failed to deliver email: {e}");
                            let _ = delivery
                                .nack(BasicNackOptions {
                                    requeue: false,
                                    ..Default::default()
                                })
                                .await;
                        } else {
                            let _ = delivery.ack(BasicAckOptions::default()).await;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Invalid SendEmailPayload JSON on RabbitMQ: {e}");
                        let _ = delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                    }
                },
                Err(e) => {
                    tracing::error!("RabbitMQ delivery stream error: {e}");
                }
            }
        }
    });
}
