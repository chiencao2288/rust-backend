use auth_service::app::Config;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_config_fail_fast_on_missing_required_vars() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Ensure required variables are absent.
    std::env::remove_var("DATABASE_MASTER_URL");
    std::env::remove_var("REDIS_NODE_01");
    std::env::remove_var("PORT");

    let result = std::panic::catch_unwind(Config::load);
    assert!(
        result.is_err(),
        "Config::load should panic on missing required variables"
    );
}

#[test]
fn test_config_loads_successfully_with_required_vars() {
    let _guard = ENV_LOCK.lock().unwrap();

    std::env::set_var(
        "DATABASE_MASTER_URL",
        "postgres://user:pass@127.0.0.1:5432/test_db",
    );
    std::env::set_var("REDIS_NODE_01", "redis://127.0.0.1:6379");
    std::env::set_var("PORT", "9000");

    let config = Config::load();
    assert_eq!(config.port, 9000);
    assert_eq!(
        config.database_master_url,
        "postgres://user:pass@127.0.0.1:5432/test_db"
    );
    assert_eq!(config.redis_node_01, "redis://127.0.0.1:6379");
    assert_eq!(
        config.google_oauth_url,
        "https://oauth2.googleapis.com/tokeninfo"
    );
}
