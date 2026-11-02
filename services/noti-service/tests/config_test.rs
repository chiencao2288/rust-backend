use noti_service::app::Config;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_noti_config_fail_fast_on_missing_required_vars() {
    let _guard = ENV_LOCK.lock().unwrap();

    std::env::remove_var("DATABASE_MASTER_URL");
    std::env::remove_var("REDIS_NODE_01");
    std::env::remove_var("RABBITMQ_URL");
    std::env::remove_var("PORT");

    let result = std::panic::catch_unwind(Config::load);
    assert!(
        result.is_err(),
        "Config::load should panic when required vars are missing"
    );
}

#[test]
fn test_noti_config_loads_successfully_with_required_vars() {
    let _guard = ENV_LOCK.lock().unwrap();

    std::env::set_var(
        "DATABASE_MASTER_URL",
        "postgres://user:pass@127.0.0.1:5432/test_db",
    );
    std::env::set_var("REDIS_NODE_01", "redis://127.0.0.1:6379");
    std::env::set_var("RABBITMQ_URL", "amqp://guest:guest@127.0.0.1:5672/%2f");
    std::env::set_var("PORT", "9090");

    let config = Config::load();
    assert_eq!(config.port, 9090);
    assert_eq!(
        config.database_master_url,
        "postgres://user:pass@127.0.0.1:5432/test_db"
    );
    assert_eq!(config.redis_node_01, "redis://127.0.0.1:6379");
    assert_eq!(config.rabbitmq_url, "amqp://guest:guest@127.0.0.1:5672/%2f");
    assert_eq!(config.auth_service_url, "http://127.0.0.1:8080");
    assert_eq!(config.email_provider, "smtp");
    assert_eq!(config.mail_port, 587);
    assert_eq!(config.mail_from, "chiencao19920808@gmail.com");
    assert_eq!(config.mail_from_name, "Podkeen");
    assert_eq!(config.sms_provider, "mock");
}
