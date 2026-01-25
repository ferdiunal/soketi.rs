use soketi_rs::queues::{QueueManager, SqsQueueConfig, SqsQueueManager, WebhookJob};
use soketi_rs::webhook_sender::WebhookSender;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Helper function to create a test job
fn create_test_job(event_type: &str, channel: &str) -> WebhookJob {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    WebhookJob {
        app_id: "test_app".to_string(),
        app_key: "test_key".to_string(),
        app_secret: "test_secret".to_string(),
        event_type: event_type.to_string(),
        channel: channel.to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
        timestamp,
    }
}

#[test]
fn test_sqs_queue_config_default() {
    let config = SqsQueueConfig::default();
    assert_eq!(config.queue_url, "");
    assert_eq!(config.concurrency, 1);
    assert_eq!(config.batch_size, 10);
    assert_eq!(config.wait_time_seconds, 20);
    assert_eq!(config.visibility_timeout, 30);
    assert_eq!(config.max_retries, 3);
    assert!(config.region.is_none());
}

#[test]
fn test_sqs_queue_config_custom() {
    let config = SqsQueueConfig {
        queue_url: "https://sqs.us-east-1.amazonaws.com/123456789012/my-queue".to_string(),
        concurrency: 4,
        batch_size: 5,
        wait_time_seconds: 10,
        visibility_timeout: 60,
        max_retries: 5,
        region: Some("us-west-2".to_string()),
    };

    assert_eq!(
        config.queue_url,
        "https://sqs.us-east-1.amazonaws.com/123456789012/my-queue"
    );
    assert_eq!(config.concurrency, 4);
    assert_eq!(config.batch_size, 5);
    assert_eq!(config.wait_time_seconds, 10);
    assert_eq!(config.visibility_timeout, 60);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.region, Some("us-west-2".to_string()));
}

#[test]
fn test_webhook_job_creation_for_sqs() {
    let job = create_test_job("channel_occupied", "test-channel");

    assert_eq!(job.app_id, "test_app");
    assert_eq!(job.event_type, "channel_occupied");
    assert_eq!(job.channel, "test-channel");
    assert!(job.event.is_none());
    assert!(job.user_id.is_none());
}

#[test]
fn test_webhook_job_serialization_for_sqs() {
    let job = create_test_job("member_added", "presence-test");

    // Test serialization
    let json = serde_json::to_string(&job).unwrap();
    assert!(json.contains("test_app"));
    assert!(json.contains("member_added"));
    assert!(json.contains("presence-test"));

    // Test deserialization
    let deserialized: WebhookJob = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.app_id, job.app_id);
    assert_eq!(deserialized.event_type, job.event_type);
    assert_eq!(deserialized.channel, job.channel);
}

#[test]
fn test_webhook_job_with_client_event_data_for_sqs() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let event_data = serde_json::json!({
        "message": "Hello, world!",
        "count": 42
    });

    let job = WebhookJob {
        app_id: "test_app".to_string(),
        app_key: "test_key".to_string(),
        app_secret: "test_secret".to_string(),
        event_type: "client_event".to_string(),
        channel: "private-chat".to_string(),
        event: Some("client-message".to_string()),
        data: Some(event_data.clone()),
        socket_id: Some("socket789".to_string()),
        user_id: Some("user123".to_string()),
        timestamp,
    };

    assert_eq!(job.event_type, "client_event");
    assert_eq!(job.event, Some("client-message".to_string()));
    assert!(job.data.is_some());

    let data = job.data.unwrap();
    assert_eq!(data["message"], "Hello, world!");
    assert_eq!(data["count"], 42);
}

// Note: The following tests require AWS credentials and a real SQS queue
// They are marked with #[ignore] by default and can be run with:
// cargo test --test sqs_queue_manager_test -- --ignored
//
// To run these tests, you need to:
// 1. Set up AWS credentials (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
// 2. Create an SQS queue and set the QUEUE_URL environment variable
// 3. Run: cargo test --test sqs_queue_manager_test -- --ignored

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_creation() {
    let queue_url = std::env::var("TEST_SQS_QUEUE_URL").unwrap_or_else(|_| {
        "https://sqs.us-east-1.amazonaws.com/123456789012/test-queue".to_string()
    });

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 1,
        batch_size: 10,
        wait_time_seconds: 20,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let result = SqsQueueManager::new(config, webhook_sender).await;

    assert!(result.is_ok());

    let queue_manager = result.unwrap();
    assert!(queue_manager.disconnect().await.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_enqueue() {
    let queue_url =
        std::env::var("TEST_SQS_QUEUE_URL").expect("TEST_SQS_QUEUE_URL must be set for this test");

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 1,
        batch_size: 10,
        wait_time_seconds: 20,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SqsQueueManager::new(config, webhook_sender).await.unwrap();

    let job = create_test_job("channel_occupied", "test-channel");

    // Enqueue the job
    let result = queue_manager.enqueue(job).await;
    assert!(result.is_ok());

    queue_manager.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_multiple_jobs() {
    let queue_url =
        std::env::var("TEST_SQS_QUEUE_URL").expect("TEST_SQS_QUEUE_URL must be set for this test");

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 1,
        batch_size: 10,
        wait_time_seconds: 20,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SqsQueueManager::new(config, webhook_sender).await.unwrap();

    // Enqueue multiple jobs
    for i in 0..5 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    queue_manager.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_worker_processing() {
    use tokio::time::{Duration, sleep};

    let queue_url =
        std::env::var("TEST_SQS_QUEUE_URL").expect("TEST_SQS_QUEUE_URL must be set for this test");

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 2,
        batch_size: 10,
        wait_time_seconds: 5,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SqsQueueManager::new(config, webhook_sender).await.unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

    // Enqueue jobs
    for i in 0..3 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Wait for workers to process jobs
    sleep(Duration::from_secs(10)).await;

    queue_manager.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_batch_processing() {
    use tokio::time::{Duration, sleep};

    let queue_url =
        std::env::var("TEST_SQS_QUEUE_URL").expect("TEST_SQS_QUEUE_URL must be set for this test");

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 1,
        batch_size: 10, // Process up to 10 messages at once
        wait_time_seconds: 5,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SqsQueueManager::new(config, webhook_sender).await.unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

    // Enqueue many jobs to test batch processing
    for i in 0..20 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Wait for workers to process jobs in batches
    sleep(Duration::from_secs(15)).await;

    queue_manager.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_different_event_types() {
    use tokio::time::{Duration, sleep};

    let queue_url =
        std::env::var("TEST_SQS_QUEUE_URL").expect("TEST_SQS_QUEUE_URL must be set for this test");

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 1,
        batch_size: 10,
        wait_time_seconds: 5,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SqsQueueManager::new(config, webhook_sender).await.unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

    // Enqueue different event types
    let event_types = vec![
        "channel_occupied",
        "channel_vacated",
        "member_added",
        "member_removed",
        "cache_miss",
    ];

    for event_type in event_types {
        let mut job = create_test_job(event_type, "test-channel");
        if event_type == "member_added" || event_type == "member_removed" {
            job.user_id = Some("user123".to_string());
        }
        queue_manager.enqueue(job).await.unwrap();
    }

    // Wait for workers to process jobs
    sleep(Duration::from_secs(10)).await;

    queue_manager.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_sqs_queue_manager_get_queue_length() {
    let queue_url =
        std::env::var("TEST_SQS_QUEUE_URL").expect("TEST_SQS_QUEUE_URL must be set for this test");

    let config = SqsQueueConfig {
        queue_url,
        concurrency: 1,
        batch_size: 10,
        wait_time_seconds: 20,
        visibility_timeout: 30,
        max_retries: 3,
        region: Some("us-east-1".to_string()),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SqsQueueManager::new(config, webhook_sender).await.unwrap();

    // Get initial queue length (should be 0 or close to it)
    let initial_length = queue_manager.get_queue_length().await.unwrap();
    println!("Initial queue length: {}", initial_length);

    // Enqueue some jobs
    for i in 0..3 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Note: SQS queue length is approximate and may take time to update
    // So we just verify the call succeeds
    let length = queue_manager.get_queue_length().await;
    assert!(length.is_ok());

    queue_manager.disconnect().await.unwrap();
}
