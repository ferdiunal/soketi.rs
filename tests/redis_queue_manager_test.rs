use soketi_rs::queues::{QueueManager, RedisQueueConfig, RedisQueueManager, WebhookJob};
use soketi_rs::webhook_sender::WebhookSender;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, sleep};

// Helper function to check if Redis is available
async fn is_redis_available() -> bool {
    match redis::Client::open("redis://127.0.0.1:6379") {
        Ok(client) => match client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                use redis::AsyncCommands;
                conn.set::<&str, &str, ()>("test_key", "test_value")
                    .await
                    .is_ok()
            }
            Err(_) => false,
        },
        Err(_) => false,
    }
}

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

// Helper function to clean up Redis queues
async fn cleanup_redis_queues(prefix: &str) {
    if let Ok(client) = redis::Client::open("redis://127.0.0.1:6379") {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            use redis::AsyncCommands;
            let _: Result<(), redis::RedisError> = conn.del(format!("{}:jobs", prefix)).await;
            let _: Result<(), redis::RedisError> = conn.del(format!("{}:processing", prefix)).await;
            let _: Result<(), redis::RedisError> = conn.del(format!("{}:failed", prefix)).await;
        }
    }
}

#[tokio::test]
async fn test_redis_queue_manager_creation() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: "test:queue:creation".to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let result = RedisQueueManager::new(config, webhook_sender);

    assert!(result.is_ok());

    let queue_manager = result.unwrap();
    assert!(queue_manager.disconnect().await.is_ok());
}

#[tokio::test]
async fn test_redis_queue_manager_enqueue() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:enqueue";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    let job = create_test_job("channel_occupied", "test-channel");

    // Enqueue the job
    let result = queue_manager.enqueue(job).await;
    assert!(result.is_ok());

    // Check queue length
    let length = queue_manager.get_queue_length().await.unwrap();
    assert_eq!(length, 1);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_multiple_jobs() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:multiple";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    // Enqueue multiple jobs
    for i in 0..5 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Check queue length
    let length = queue_manager.get_queue_length().await.unwrap();
    assert_eq!(length, 5);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_worker_processing() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:worker";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 2,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

    // Enqueue jobs
    for i in 0..3 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Wait for workers to process jobs
    sleep(Duration::from_secs(2)).await;

    // Check that queue is empty (jobs processed)
    let length = queue_manager.get_queue_length().await.unwrap();
    assert_eq!(length, 0);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_concurrency() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:concurrency";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 4, // 4 concurrent workers
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

    // Enqueue many jobs
    for i in 0..20 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Wait for workers to process jobs
    sleep(Duration::from_secs(3)).await;

    // Check that queue is empty (jobs processed)
    let length = queue_manager.get_queue_length().await.unwrap();
    assert_eq!(length, 0);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_different_event_types() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:events";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

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
    sleep(Duration::from_secs(2)).await;

    // Check that queue is empty (jobs processed)
    let length = queue_manager.get_queue_length().await.unwrap();
    assert_eq!(length, 0);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_client_event() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:client_event";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

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
        data: Some(event_data),
        socket_id: Some("socket789".to_string()),
        user_id: Some("user123".to_string()),
        timestamp,
    };

    queue_manager.enqueue(job).await.unwrap();

    // Wait for worker to process job
    sleep(Duration::from_secs(2)).await;

    // Check that queue is empty (job processed)
    let length = queue_manager.get_queue_length().await.unwrap();
    assert_eq!(length, 0);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_queue_lengths() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:lengths";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    // Initially all queues should be empty
    assert_eq!(queue_manager.get_queue_length().await.unwrap(), 0);
    assert_eq!(queue_manager.get_processing_length().await.unwrap(), 0);
    assert_eq!(queue_manager.get_failed_length().await.unwrap(), 0);

    // Enqueue some jobs
    for i in 0..3 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Queue should have 3 jobs
    assert_eq!(queue_manager.get_queue_length().await.unwrap(), 3);

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_disconnect() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let queue_prefix = "test:queue:disconnect";
    cleanup_redis_queues(queue_prefix).await;

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 2,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: queue_prefix.to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = RedisQueueManager::new(config, webhook_sender).unwrap();

    // Start workers
    queue_manager.start_workers().await.unwrap();

    // Enqueue some jobs
    for i in 0..5 {
        let job = create_test_job("channel_occupied", &format!("channel-{}", i));
        queue_manager.enqueue(job).await.unwrap();
    }

    // Disconnect should stop workers gracefully
    let result = queue_manager.disconnect().await;
    assert!(result.is_ok());

    cleanup_redis_queues(queue_prefix).await;
}

#[tokio::test]
async fn test_redis_queue_manager_invalid_redis_url() {
    let config = RedisQueueConfig {
        redis_url: "redis://invalid-host:9999".to_string(),
        concurrency: 1,
        max_retries: 3,
        retry_delay_ms: 100,
        queue_prefix: "test:queue".to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let result = RedisQueueManager::new(config, webhook_sender);

    // Should succeed in creating the manager (connection is lazy)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_redis_queue_manager_custom_config() {
    if !is_redis_available().await {
        println!("Skipping test: Redis not available");
        return;
    }

    let config = RedisQueueConfig {
        redis_url: "redis://127.0.0.1:6379".to_string(),
        concurrency: 8,
        max_retries: 5,
        retry_delay_ms: 2000,
        queue_prefix: "custom:prefix".to_string(),
    };

    let webhook_sender = Arc::new(WebhookSender::new());
    let result = RedisQueueManager::new(config.clone(), webhook_sender);

    assert!(result.is_ok());

    let queue_manager = result.unwrap();

    // Verify we can enqueue with custom prefix
    let job = create_test_job("channel_occupied", "test-channel");
    assert!(queue_manager.enqueue(job).await.is_ok());

    queue_manager.disconnect().await.unwrap();
    cleanup_redis_queues(&config.queue_prefix).await;
}
