use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub mod redis;
pub mod sqs;
pub mod sync;

// Re-export the queue manager implementations
pub use redis::{RedisQueueConfig, RedisQueueManager};
pub use sqs::{SqsQueueConfig, SqsQueueManager};
pub use sync::SyncQueueManager;

pub type JobHandler = Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Webhook job data structure for queue processing
///
/// This struct represents a webhook job that will be processed asynchronously.
/// Jobs are enqueued when webhook events occur and processed by queue workers.
///
/// # Examples
///
/// ## Creating a channel occupied webhook job
/// ```
/// use soketi_rs::queues::WebhookJob;
/// use std::time::{SystemTime, UNIX_EPOCH};
///
/// let timestamp = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
///
/// let job = WebhookJob {
///     app_id: "app-123".to_string(),
///     app_key: "key-123".to_string(),
///     app_secret: "secret-123".to_string(),
///     event_type: "channel_occupied".to_string(),
///     channel: "private-chat".to_string(),
///     event: None,
///     data: None,
///     socket_id: None,
///     user_id: None,
///     timestamp,
/// };
/// ```
///
/// ## Creating a client event webhook job
/// ```
/// use soketi_rs::queues::WebhookJob;
/// use std::time::{SystemTime, UNIX_EPOCH};
/// use serde_json::json;
///
/// let timestamp = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_secs();
///
/// let job = WebhookJob {
///     app_id: "app-123".to_string(),
///     app_key: "key-123".to_string(),
///     app_secret: "secret-123".to_string(),
///     event_type: "client_event".to_string(),
///     channel: "private-chat".to_string(),
///     event: Some("client-message".to_string()),
///     data: Some(json!({"text": "Hello!"})),
///     socket_id: Some("socket-456".to_string()),
///     user_id: Some("user-789".to_string()),
///     timestamp,
/// };
/// ```
///
/// **Validates: Requirements 9.1-9.6, 10.1-10.6**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookJob {
    /// The app ID this webhook belongs to
    pub app_id: String,

    /// The app key for authentication
    pub app_key: String,

    /// The app secret for signing
    pub app_secret: String,

    /// The webhook event type (channel_occupied, channel_vacated, member_added, etc.)
    pub event_type: String,

    /// The channel name
    pub channel: String,

    /// Optional event name (for client events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,

    /// Optional event data (for client events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    /// Optional socket ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_id: Option<String>,

    /// Optional user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Timestamp when the job was created
    pub timestamp: u64,
}

/// QueueManager trait for managing asynchronous webhook job processing
///
/// The QueueManager is responsible for enqueueing webhook jobs and managing
/// the lifecycle of queue connections. Different implementations support
/// different backends:
/// - **SyncQueueManager**: Processes webhooks immediately (no queue)
/// - **RedisQueueManager**: Uses Redis with BullMQ-compatible operations
/// - **SqsQueueManager**: Uses AWS SQS for queue processing
///
/// # Examples
///
/// ## Using SyncQueueManager
/// ```ignore
/// use soketi_rs::queues::{SyncQueueManager, QueueManager, WebhookJob};
/// use soketi_rs::webhook_sender::WebhookSender;
/// use std::sync::Arc;
///
/// let webhook_sender = Arc::new(WebhookSender::new());
/// let queue = SyncQueueManager::new(webhook_sender);
///
/// // Enqueue a job (will be processed immediately)
/// queue.enqueue(job).await?;
/// ```
///
/// ## Using RedisQueueManager
/// ```ignore
/// use soketi_rs::queues::{RedisQueueManager, RedisQueueConfig, QueueManager};
/// use soketi_rs::webhook_sender::WebhookSender;
/// use std::sync::Arc;
///
/// let config = RedisQueueConfig {
///     redis_url: "redis://localhost:6379".to_string(),
///     concurrency: 4,
///     max_retries: 3,
///     retry_delay_ms: 1000,
///     queue_prefix: "pusher:queue".to_string(),
/// };
///
/// let webhook_sender = Arc::new(WebhookSender::new());
/// let queue = RedisQueueManager::new(config, webhook_sender)?;
///
/// // Enqueue a job (will be processed by workers)
/// queue.enqueue(job).await?;
/// ```
///
/// **Validates: Requirements 9.1-9.6**
#[async_trait]
pub trait QueueManager: Send + Sync {
    /// Enqueue a webhook job for asynchronous processing
    ///
    /// # Arguments
    /// * `job` - The webhook job to enqueue
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the job was successfully enqueued, Err otherwise
    async fn enqueue(&self, job: WebhookJob) -> Result<()>;

    /// Disconnect and clean up queue resources
    ///
    /// This method should be called during graceful shutdown to ensure
    /// all queue connections are properly closed and resources are released.
    ///
    /// # Returns
    /// * `Result<()>` - Ok if disconnection was successful, Err otherwise
    async fn disconnect(&self) -> Result<()>;
}

// Legacy Queue trait for backward compatibility
#[async_trait]
pub trait Queue: Send + Sync {
    async fn add_to_queue(&self, queue_name: &str, data: Value);
    async fn process_queue(&self, queue_name: &str, handler: JobHandler);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_webhook_job_creation() {
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

        assert_eq!(job.app_id, "test_app");
        assert_eq!(job.event_type, "channel_occupied");
        assert_eq!(job.channel, "test-channel");
        assert!(job.event.is_none());
        assert!(job.user_id.is_none());
    }

    #[test]
    fn test_webhook_job_serialization() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let job = WebhookJob {
            app_id: "test_app".to_string(),
            app_key: "test_key".to_string(),
            app_secret: "test_secret".to_string(),
            event_type: "member_added".to_string(),
            channel: "presence-test".to_string(),
            event: None,
            data: None,
            socket_id: Some("socket123".to_string()),
            user_id: Some("user456".to_string()),
            timestamp,
        };

        // Test serialization
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("test_app"));
        assert!(json.contains("member_added"));
        assert!(json.contains("socket123"));
        assert!(json.contains("user456"));

        // Test deserialization
        let deserialized: WebhookJob = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.app_id, job.app_id);
        assert_eq!(deserialized.event_type, job.event_type);
        assert_eq!(deserialized.socket_id, job.socket_id);
        assert_eq!(deserialized.user_id, job.user_id);
    }

    #[test]
    fn test_webhook_job_with_client_event_data() {
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

    // Mock implementation of QueueManager for testing
    struct MockQueueManager {
        jobs: Arc<tokio::sync::Mutex<Vec<WebhookJob>>>,
    }

    impl MockQueueManager {
        fn new() -> Self {
            Self {
                jobs: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        async fn get_jobs(&self) -> Vec<WebhookJob> {
            self.jobs.lock().await.clone()
        }
    }

    #[async_trait]
    impl QueueManager for MockQueueManager {
        async fn enqueue(&self, job: WebhookJob) -> Result<()> {
            self.jobs.lock().await.push(job);
            Ok(())
        }

        async fn disconnect(&self) -> Result<()> {
            self.jobs.lock().await.clear();
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_queue_manager_enqueue() {
        let queue = MockQueueManager::new();
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

        // Enqueue the job
        let result = queue.enqueue(job.clone()).await;
        assert!(result.is_ok());

        // Verify the job was enqueued
        let jobs = queue.get_jobs().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].app_id, "test_app");
        assert_eq!(jobs[0].event_type, "channel_occupied");
    }

    #[tokio::test]
    async fn test_queue_manager_disconnect() {
        let queue = MockQueueManager::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Enqueue multiple jobs
        for i in 0..3 {
            let job = WebhookJob {
                app_id: format!("app_{}", i),
                app_key: "test_key".to_string(),
                app_secret: "test_secret".to_string(),
                event_type: "channel_occupied".to_string(),
                channel: format!("channel-{}", i),
                event: None,
                data: None,
                socket_id: None,
                user_id: None,
                timestamp,
            };
            queue.enqueue(job).await.unwrap();
        }

        // Verify jobs were enqueued
        assert_eq!(queue.get_jobs().await.len(), 3);

        // Disconnect and verify cleanup
        let result = queue.disconnect().await;
        assert!(result.is_ok());
        assert_eq!(queue.get_jobs().await.len(), 0);
    }

    #[tokio::test]
    async fn test_queue_manager_multiple_event_types() {
        let queue = MockQueueManager::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event_types = vec![
            "channel_occupied",
            "channel_vacated",
            "member_added",
            "member_removed",
            "client_event",
            "cache_miss",
        ];

        // Enqueue jobs for different event types
        for event_type in event_types {
            let job = WebhookJob {
                app_id: "test_app".to_string(),
                app_key: "test_key".to_string(),
                app_secret: "test_secret".to_string(),
                event_type: event_type.to_string(),
                channel: "test-channel".to_string(),
                event: None,
                data: None,
                socket_id: None,
                user_id: None,
                timestamp,
            };
            queue.enqueue(job).await.unwrap();
        }

        // Verify all jobs were enqueued
        let jobs = queue.get_jobs().await;
        assert_eq!(jobs.len(), 6);

        // Verify each event type is present
        let event_types_in_queue: Vec<String> = jobs.iter().map(|j| j.event_type.clone()).collect();

        assert!(event_types_in_queue.contains(&"channel_occupied".to_string()));
        assert!(event_types_in_queue.contains(&"channel_vacated".to_string()));
        assert!(event_types_in_queue.contains(&"member_added".to_string()));
        assert!(event_types_in_queue.contains(&"member_removed".to_string()));
        assert!(event_types_in_queue.contains(&"client_event".to_string()));
        assert!(event_types_in_queue.contains(&"cache_miss".to_string()));
    }
}
