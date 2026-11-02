use auth_service::app::{AppState, Config};
use auth_service::modules::register_routes;
use rn_bootstrap::RNBootstrap;
use rn_postgres::RNPostgres;
use rn_redis::RNRedis;

fn main() {
    RNBootstrap::builder()
        .with_service_name("auth-service")
        .with_default_log_level("info,auth_service=debug")
        .run(|app| async move {
            let config = Config::load();

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

            let port = config.port;
            let state = AppState::new(db, redis, config);
            let router = register_routes(state);

            app.serve_http(router, port).await
        });
}
