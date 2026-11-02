use std::sync::Arc;

use futures_lite::StreamExt;
use lapin::options::{BasicAckOptions, BasicNackOptions};
use rn_contracts::noti::{SendInAppPayload, QUEUE_IN_APP};
use rn_rabbitmq::RNRabbitMQ;

use super::service::InAppService;

/// Starts a background Tokio task consuming in-app notification messages from RabbitMQ.
pub fn start_in_app_consumer(rabbitmq: RNRabbitMQ, in_app_service: Arc<InAppService>) {
    tokio::spawn(async move {
        tracing::info!("Starting RabbitMQ in-app consumer on queue `{QUEUE_IN_APP}`");

        let mut consumer = match rabbitmq
            .create_consumer(QUEUE_IN_APP, "noti-in-app-worker")
            .await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to register RabbitMQ in-app consumer: {e}");
                return;
            }
        };

        while let Some(delivery_result) = consumer.next().await {
            match delivery_result {
                Ok(delivery) => match serde_json::from_slice::<SendInAppPayload>(&delivery.data) {
                    Ok(payload) => {
                        if let Err(e) = in_app_service.create_notification(&payload).await {
                            tracing::error!(
                                user_id = %payload.user_id,
                                "Failed to persist in-app notification: {e}"
                            );
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
                        tracing::error!("Invalid SendInAppPayload JSON on RabbitMQ: {e}");
                        let _ = delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                    }
                },
                Err(e) => {
                    tracing::error!("RabbitMQ in-app delivery stream error: {e}");
                }
            }
        }
    });
}
