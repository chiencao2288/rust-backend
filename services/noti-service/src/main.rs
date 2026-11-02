use noti_service::app::{AppState, Config};
use noti_service::modules::{register_routes, start_background_workers};
use rn_bootstrap::RNBootstrap;
use rn_postgres::RNPostgres;
use rn_rabbitmq::RNRabbitMQ;
use rn_redis::RNRedis;

fn main() {
    RNBootstrap::builder()
        .with_service_name("noti-service")
        .with_default_log_level("info,noti_service=debug")
        .build_config::<Config>()
        .build_state(|config| async move {
            let db = RNPostgres::builder()
                .with_master_url(&config.database_master_url)
                .add_slave_url(config.database_slave_url_01.as_deref())
                .build()
                .await?;

            let redis = RNRedis::builder()
                .add_node(Some(&config.redis_node_01))
                .with_password(config.redis_password.as_deref())
                .build()
                .await?;

            let rabbitmq = RNRabbitMQ::builder()
                .with_url(&config.rabbitmq_url)
                .build()
                .await?;

            Ok(AppState::new(db, redis, rabbitmq, config))
        })
        .build_worker(|state| async move { start_background_workers(&state).await })
        .with_http(|state| async move {
            let port = state.config.port;
            let router = register_routes(state);
            (router, port)
        })
        .run();
}
