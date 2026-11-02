use std::sync::Arc;

use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicNackOptions};
use rn_contracts::noti::{SendSmsPayload, QUEUE_SMS};
use rn_rabbitmq::RNRabbitMQ;

use super::service::SmsService;

/// Starts a background Tokio task consuming SMS delivery messages from RabbitMQ.
pub fn start_sms_consumer(rabbitmq: RNRabbitMQ, sms_service: Arc<SmsService>) {
    tokio::spawn(async move {
        tracing::info!("Starting RabbitMQ SMS consumer on queue `{QUEUE_SMS}`");

        let mut consumer = match rabbitmq.create_consumer(QUEUE_SMS, "noti-sms-worker").await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to register RabbitMQ SMS consumer: {e}");
                return;
            }
        };

        while let Some(delivery_result) = consumer.next().await {
            match delivery_result {
                Ok(delivery) => match serde_json::from_slice::<SendSmsPayload>(&delivery.data) {
                    Ok(payload) => {
                        if let Err(e) = sms_service.send_sms(&payload).await {
                            tracing::error!(to = %payload.to, "Failed to deliver SMS: {e}");
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
                        tracing::error!("Invalid SendSmsPayload JSON on RabbitMQ: {e}");
                        let _ = delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                    }
                },
                Err(e) => {
                    tracing::error!("RabbitMQ SMS delivery stream error: {e}");
                }
            }
        }
    });
}
