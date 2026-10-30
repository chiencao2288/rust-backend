use auth_service::config::Config;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_config_fail_fast_and_success_flow() {
    let _guard = ENV_LOCK.lock().unwrap();

    // 1. Test fail-fast when required variables are absent
    std::env::remove_var("DATABASE_MASTER_URL");
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("REDIS_NODES");
    std::env::remove_var("REDIS_URL");

    let result = Config::load();
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Missing required environment variable"));

    // 2. Test successful loading when required variables are present
    std::env::set_var(
        "DATABASE_MASTER_URL",
        "postgres://user:pass@127.0.0.1:5432/test_db",
    );
    std::env::set_var("REDIS_NODES", "redis://127.0.0.1:6379");
    std::env::set_var("PORT", "9000");

    let config = Config::load().expect("Config should load successfully");
    assert_eq!(config.port, 9000);
    assert_eq!(
        config.database_master_url,
        "postgres://user:pass@127.0.0.1:5432/test_db"
    );
    assert_eq!(config.redis_nodes, vec!["redis://127.0.0.1:6379"]);
}
