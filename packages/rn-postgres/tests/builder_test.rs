use rn_postgres::{PostgresError, RNPostgres, RNPostgresBuilder};
use std::time::Duration;

#[test]
fn test_builder_fails_without_master_url() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = RNPostgresBuilder::new().build().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            PostgresError::Configuration(msg) => {
                assert!(msg.contains("Master database URL is required"));
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    });
}

#[test]
fn test_builder_fluent_configuration() {
    let builder = RNPostgres::builder()
        .with_master_url("postgres://user:pass@master:5432/db")
        .add_slave_url(Some("postgres://user:pass@slave1:5432/db"))
        .add_slave_url(Some("postgres://user:pass@slave2:5432/db"))
        .with_max_connections(30)
        .with_min_connections(5)
        .with_acquire_timeout(Duration::from_secs(10))
        .with_idle_timeout(Duration::from_secs(300));

    let debug_str = format!("{:?}", builder);
    assert!(debug_str.contains("master:5432"));
    assert!(debug_str.contains("slave1:5432"));
    assert!(debug_str.contains("slave2:5432"));
    assert!(debug_str.contains("max_connections: 30"));
}

#[test]
fn test_violation_checkers_return_false_for_row_not_found() {
    let not_found_err = sqlx::Error::RowNotFound;
    assert!(!RNPostgres::is_unique_violation(&not_found_err));
    assert!(!RNPostgres::is_foreign_key_violation(&not_found_err));
}
