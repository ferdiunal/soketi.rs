use soketi_rs::queues::{QueueManager, SyncQueueManager, WebhookJob};
use soketi_rs::webhook_sender::WebhookSender;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Test that SyncQueueManager can be created and used
#[tokio::test]
async fn test_sync_queue_manager_basic_usage() {
    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SyncQueueManager::new(webhook_sender);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let job = WebhookJob {
        app_id: "test_app".to_string(),
        app_key: "test_key".to_string(),
        app_secret: "test_secret".to_string(),
        event_type: "channel_occupied".to_string(),
        channel: "test-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
        timestamp,
    };

    // Should execute immediately without error
    let result = queue_manager.enqueue(job).await;
    assert!(result.is_ok());

    // Disconnect should work without error
    let result = queue_manager.disconnect().await;
    assert!(result.is_ok());
}

/// Test that SyncQueueManager handles all webhook event types
#[tokio::test]
async fn test_sync_queue_manager_all_event_types() {
    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SyncQueueManager::new(webhook_sender);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let event_types = vec![
        ("channel_occupied", None, None, None, None),
        ("channel_vacated", None, None, None, None),
        ("member_added", None, None, None, Some("user123")),
        ("member_removed", None, None, None, Some("user456")),
        (
            "client_event",
            Some("client-message"),
            Some(serde_json::json!({"msg": "hello"})),
            Some("socket789"),
            Some("user789"),
        ),
        ("cache_miss", None, None, None, None),
    ];

    for (event_type, event, data, socket_id, user_id) in event_types {
        let job = WebhookJob {
            app_id: "test_app".to_string(),
            app_key: "test_key".to_string(),
            app_secret: "test_secret".to_string(),
            event_type: event_type.to_string(),
            channel: "test-channel".to_string(),
            event: event.map(|s| s.to_string()),
            data,
            socket_id: socket_id.map(|s| s.to_string()),
            user_id: user_id.map(|s| s.to_string()),
            timestamp,
        };

        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok(), "Failed to enqueue {} event", event_type);
    }
}

/// Test that SyncQueueManager executes jobs immediately (synchronously)
#[tokio::test]
async fn test_sync_queue_manager_immediate_execution() {
    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = SyncQueueManager::new(webhook_sender);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Enqueue multiple jobs rapidly
    for i in 0..10 {
        let job = WebhookJob {
            app_id: format!("app_{}", i),
            app_key: format!("key_{}", i),
            app_secret: format!("secret_{}", i),
            event_type: "channel_occupied".to_string(),
            channel: format!("channel-{}", i),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
            timestamp,
        };

        // Each job should complete immediately
        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    // All jobs should have been processed by now since they execute immediately
    // No need to wait or poll for completion
}

/// Test that SyncQueueManager can be cloned and used from multiple tasks
#[tokio::test]
async fn test_sync_queue_manager_concurrent_usage() {
    let webhook_sender = Arc::new(WebhookSender::new());
    let queue_manager = Arc::new(SyncQueueManager::new(webhook_sender));

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Spawn multiple tasks that enqueue jobs concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let qm = queue_manager.clone();
        let handle = tokio::spawn(async move {
            let job = WebhookJob {
                app_id: format!("app_{}", i),
                app_key: format!("key_{}", i),
                app_secret: format!("secret_{}", i),
                event_type: "channel_occupied".to_string(),
                channel: format!("channel-{}", i),
                event: None,
                data: None,
                socket_id: None,
                user_id: None,
                timestamp,
            };

            qm.enqueue(job).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
