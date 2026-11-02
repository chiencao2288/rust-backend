use rn_rabbitmq::RNRabbitMQ;
use std::time::Duration;

#[test]
fn test_builder_fluent_configuration() {
    let builder = RNRabbitMQ::builder()
        .with_url("amqp://guest:guest@127.0.0.1:5672/%2f")
        .with_timeout(Duration::from_secs(10))
        .with_prefetch_count(100);

    let debug_str = format!("{:?}", builder);
    assert!(debug_str.contains("5672"));
    assert!(debug_str.contains("100"));
}
