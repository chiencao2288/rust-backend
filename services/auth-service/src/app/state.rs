use super::Config;
use crate::common::repositories::{SessionStore, TokensRepository, UsersRepository};
use rn_postgres::RNPostgres;
use rn_rabbitmq::RNRabbitMQ;
use rn_redis::RNRedis;

/// Shared application state containing common repositories, infrastructure clients, and configuration.
#[derive(Clone)]
pub struct AppState {
    pub users_repo: UsersRepository,
    pub tokens_repo: TokensRepository,
    pub session_store: SessionStore,
    pub http_client: reqwest::Client,
    pub rabbitmq: RNRabbitMQ,
    pub config: Config,
}

impl AppState {
    /// Initializes all common repositories and returns a new `AppState`.
    pub fn new(db: RNPostgres, redis: RNRedis, rabbitmq: RNRabbitMQ, config: Config) -> Self {
        let users_repo = UsersRepository::new(db.clone());
        let tokens_repo = TokensRepository::new(db.clone());
        let session_store = SessionStore::new(redis);
        let http_client = reqwest::Client::new();

        Self {
            users_repo,
            tokens_repo,
            session_store,
            http_client,
            rabbitmq,
            config,
        }
    }
}
