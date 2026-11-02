use rn_postgres::RNPostgres;
use rn_rabbitmq::RNRabbitMQ;
use rn_redis::RNRedis;

use crate::app::Config;
use crate::common::repositories::NotificationRepository;

/// Shared application state containing common infrastructure clients, repositories, and config.
#[derive(Clone)]
pub struct AppState {
    pub db: RNPostgres,
    pub redis: RNRedis,
    pub rabbitmq: RNRabbitMQ,
    pub noti_repo: NotificationRepository,
    pub http_client: reqwest::Client,
    pub config: Config,
}

impl AppState {
    /// Initializes all repositories and returns a new `AppState`.
    pub fn new(db: RNPostgres, redis: RNRedis, rabbitmq: RNRabbitMQ, config: Config) -> Self {
        let noti_repo = NotificationRepository::new(db.clone());
        let http_client = reqwest::Client::new();

        Self {
            db,
            redis,
            rabbitmq,
            noti_repo,
            http_client,
            config,
        }
    }
}
