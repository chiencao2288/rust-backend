use rn_redis::{RNRedis, RNRedisBuilder, RedisError};
use std::time::Duration;

#[test]
fn test_builder_fails_without_nodes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = RNRedisBuilder::new().build().await;
        assert!(result.is_err());
        match result.err().unwrap() {
            RedisError::Configuration(msg) => {
                assert!(msg.contains("At least one Redis node URL"));
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    });
}

#[test]
fn test_builder_fluent_chaining() {
    let builder = RNRedis::builder()
        .add_node(Some("redis://127.0.0.1:6379"))
        .with_timeout(Duration::from_secs(10))
        .with_password(Some("secret_pass"));

    // Validates builder state
    assert!(format!("{:?}", builder).contains("127.0.0.1:6379"));
}
