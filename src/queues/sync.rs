use crate::error::Result;
use crate::queues::{JobHandler, Queue, QueueManager, WebhookJob};
use crate::webhook_sender::WebhookSender;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedSender};

/// SyncQueueManager executes webhook jobs immediately without queuing
///
/// This is the simplest queue implementation that processes jobs synchronously
/// as they are enqueued. It's suitable for development and low-traffic scenarios
/// where immediate webhook delivery is acceptable.
///
/// Requirements: 9.1
#[derive(Clone)]
pub struct SyncQueueManager {
    webhook_sender: Arc<WebhookSender>,
}

impl SyncQueueManager {
    /// Create a new SyncQueueManager with the given WebhookSender
    pub fn new(webhook_sender: Arc<WebhookSender>) -> Self {
        Self { webhook_sender }
    }
}

#[async_trait]
impl QueueManager for SyncQueueManager {
    /// Enqueue a webhook job for immediate execution
    ///
    /// This implementation executes the webhook immediately without queuing,
    /// making it synchronous in nature despite the async interface.
    async fn enqueue(&self, job: WebhookJob) -> Result<()> {
        // Create a minimal App struct from the job data for webhook sending
        // We need to reconstruct the App to use the webhook sender
        let app = crate::app::App {
            id: job.app_id.clone(),
            key: job.app_key.clone(),
            secret: job.app_secret.clone(),
            max_connections: None,
            enable_client_messages: true,
            enabled: true,
            max_backend_events_per_second: None,
            max_client_events_per_second: None,
            max_read_requests_per_second: None,
            webhooks: vec![], // Webhooks are already configured in the original app
            max_presence_members_per_channel: None,
            max_presence_member_size_in_kb: None,
            max_channel_name_length: None,
            max_event_channels_at_once: None,
            max_event_name_length: None,
            max_event_payload_in_kb: None,
            max_event_batch_size: None,
            enable_user_authentication: false,
        };

        // Execute webhook immediately based on event type
        match job.event_type.as_str() {
            "channel_occupied" => {
                self.webhook_sender
                    .send_channel_occupied(&app, &job.channel)
                    .await;
            }
            "channel_vacated" => {
                self.webhook_sender
                    .send_channel_vacated(&app, &job.channel)
                    .await;
            }
            "member_added" => {
                if let Some(user_id) = &job.user_id {
                    self.webhook_sender
                        .send_member_added(&app, &job.channel, user_id)
                        .await;
                }
            }
            "member_removed" => {
                if let Some(user_id) = &job.user_id {
                    self.webhook_sender
                        .send_member_removed(&app, &job.channel, user_id)
                        .await;
                }
            }
            "client_event" => {
                if let (Some(event), Some(data)) = (&job.event, &job.data) {
                    self.webhook_sender
                        .send_client_event(
                            &app,
                            &job.channel,
                            event,
                            data.clone(),
                            job.socket_id.as_deref(),
                            job.user_id.as_deref(),
                        )
                        .await;
                }
            }
            "cache_miss" => {
                self.webhook_sender
                    .send_cache_missed(&app, &job.channel)
                    .await;
            }
            _ => {
                // Unknown event type, log and ignore
                crate::log::Log::warning(&format!(
                    "Unknown webhook event type: {}",
                    job.event_type
                ));
            }
        }

        Ok(())
    }

    /// Disconnect and clean up resources
    ///
    /// For SyncQueueManager, there are no background workers or connections to clean up,
    /// so this is a no-op.
    async fn disconnect(&self) -> Result<()> {
        // No resources to clean up for sync queue
        Ok(())
    }
}

// Legacy SyncQueueDriver for backward compatibility
#[derive(Clone)]
pub struct SyncQueueDriver {
    senders: Arc<Mutex<HashMap<String, UnboundedSender<Value>>>>,
}

impl Default for SyncQueueDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncQueueDriver {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Queue for SyncQueueDriver {
    async fn add_to_queue(&self, queue_name: &str, data: Value) {
        let senders = self.senders.lock().unwrap();
        if let Some(sender) = senders.get(queue_name) {
            let _ = sender.send(data);
        }
    }

    async fn process_queue(&self, queue_name: &str, handler: JobHandler) {
        let (tx, mut rx) = mpsc::unbounded_channel::<Value>();

        {
            let mut senders = self.senders.lock().unwrap();
            senders.insert(queue_name.to_string(), tx);
        }

        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                handler(job).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhook_sender::WebhookSender;
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::oneshot;

    // Tests for legacy SyncQueueDriver
    #[tokio::test]
    async fn test_queue_process() {
        let queue = SyncQueueDriver::new();
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        queue
            .process_queue(
                "test_queue",
                Box::new(move |val| {
                    let tx = tx.clone();
                    Box::pin(async move {
                        if let Some(sender) = tx.lock().unwrap().take() {
                            let _ = sender.send(val);
                        }
                    })
                }),
            )
            .await;

        queue
            .add_to_queue("test_queue", serde_json::json!({"foo": "bar"}))
            .await;

        let result = tokio::time::timeout(Duration::from_secs(1), rx).await;
        assert!(result.is_ok());
        let val = result.unwrap().unwrap();
        assert_eq!(val["foo"], "bar");
    }

    // Tests for SyncQueueManager
    #[tokio::test]
    async fn test_sync_queue_manager_creation() {
        let webhook_sender = Arc::new(WebhookSender::new());
        let queue_manager = SyncQueueManager::new(webhook_sender);

        // Test that disconnect works without errors
        let result = queue_manager.disconnect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_channel_occupied() {
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
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_channel_vacated() {
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
            event_type: "channel_vacated".to_string(),
            channel: "test-channel".to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
            timestamp,
        };

        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_member_added() {
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
            event_type: "member_added".to_string(),
            channel: "presence-test".to_string(),
            event: None,
            data: None,
            socket_id: Some("socket123".to_string()),
            user_id: Some("user456".to_string()),
            timestamp,
        };

        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_member_removed() {
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
            event_type: "member_removed".to_string(),
            channel: "presence-test".to_string(),
            event: None,
            data: None,
            socket_id: Some("socket123".to_string()),
            user_id: Some("user789".to_string()),
            timestamp,
        };

        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_client_event() {
        let webhook_sender = Arc::new(WebhookSender::new());
        let queue_manager = SyncQueueManager::new(webhook_sender);

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

        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_cache_miss() {
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
            event_type: "cache_miss".to_string(),
            channel: "cache-channel".to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
            timestamp,
        };

        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_enqueue_unknown_event_type() {
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
            event_type: "unknown_event".to_string(),
            channel: "test-channel".to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
            timestamp,
        };

        // Should not error, just log a warning
        let result = queue_manager.enqueue(job).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_queue_manager_multiple_jobs() {
        let webhook_sender = Arc::new(WebhookSender::new());
        let queue_manager = SyncQueueManager::new(webhook_sender);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Enqueue multiple jobs of different types
        let jobs = vec![
            WebhookJob {
                app_id: "app1".to_string(),
                app_key: "key1".to_string(),
                app_secret: "secret1".to_string(),
                event_type: "channel_occupied".to_string(),
                channel: "channel1".to_string(),
                event: None,
                data: None,
                socket_id: None,
                user_id: None,
                timestamp,
            },
            WebhookJob {
                app_id: "app2".to_string(),
                app_key: "key2".to_string(),
                app_secret: "secret2".to_string(),
                event_type: "channel_vacated".to_string(),
                channel: "channel2".to_string(),
                event: None,
                data: None,
                socket_id: None,
                user_id: None,
                timestamp,
            },
            WebhookJob {
                app_id: "app3".to_string(),
                app_key: "key3".to_string(),
                app_secret: "secret3".to_string(),
                event_type: "member_added".to_string(),
                channel: "presence-channel".to_string(),
                event: None,
                data: None,
                socket_id: Some("socket1".to_string()),
                user_id: Some("user1".to_string()),
                timestamp,
            },
        ];

        // All jobs should be processed successfully
        for job in jobs {
            let result = queue_manager.enqueue(job).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_sync_queue_manager_disconnect_idempotent() {
        let webhook_sender = Arc::new(WebhookSender::new());
        let queue_manager = SyncQueueManager::new(webhook_sender);

        // Should be able to call disconnect multiple times without error
        assert!(queue_manager.disconnect().await.is_ok());
        assert!(queue_manager.disconnect().await.is_ok());
        assert!(queue_manager.disconnect().await.is_ok());
    }
}
