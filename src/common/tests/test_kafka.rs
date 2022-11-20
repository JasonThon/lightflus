use common::{
    kafka::{run_consumer, run_producer},
    utils,
};

#[tokio::test]
async fn test_kafka_pub_sub() {
    let kafka_host = utils::get_env("KAFKA_HOST").unwrap_or("localhost".to_string());
    println!("kafka host: {}", kafka_host);
    let consumer_result = run_consumer(format!("{}:9092", kafka_host).as_str(), "ci_group", "ci");
    if consumer_result.is_err() {
        println!("{}", consumer_result.err().unwrap());
        return;
    }
    assert!(consumer_result.is_ok());

    let consumer = consumer_result.unwrap();

    let producer = run_producer(format!("{}:9092", kafka_host).as_str(), "ci", "ci_group", 0);
    assert!(producer.is_ok());
    let producer = producer.unwrap();
    let send_result = producer.send("key".as_bytes(), "value".as_bytes());
    if send_result.is_err() {
        let send_result = producer.send("key".as_bytes(), "value".as_bytes());
        assert!(send_result.is_ok());
    } else {
        assert!(send_result.is_ok());
    }

    let opt = consumer.fetch(|msg| {
        let key = String::from_utf8(msg.key);
        assert!(key.is_ok());
        let key = key.unwrap();

        let value = String::from_utf8(msg.payload);

        assert!(value.is_ok());

        let value = value.unwrap();

        assert_eq!(key.as_str(), "key");
        assert_eq!(value.as_str(), "value");
    });

    assert!(opt.is_some());
}
