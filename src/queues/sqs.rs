use crate::error::{PusherError, Result};
use crate::queues::{JobHandler, Queue, QueueManager, WebhookJob};
use crate::webhook_sender::WebhookSender;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_sqs::Client;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

/// Configuration for SqsQueueManager
#[derive(Debug, Clone)]
pub struct SqsQueueConfig {
    /// SQS queue URL
    pub queue_url: String,

    /// Number of concurrent worker tasks
    pub concurrency: usize,

    /// Maximum number of messages to receive in a single batch (1-10)
    pub batch_size: i32,

    /// Wait time for long polling in seconds (0-20)
    pub wait_time_seconds: i32,

    /// Visibility timeout in seconds (how long a message is hidden after being received)
    pub visibility_timeout: i32,

    /// Maximum number of retry attempts for failed jobs
    pub max_retries: u32,

    /// AWS region (optional, defaults to environment/config)
    pub region: Option<String>,
}

impl Default for SqsQueueConfig {
    fn default() -> Self {
        Self {
            queue_url: String::new(),
            concurrency: 1,
            batch_size: 10,
            wait_time_seconds: 20,
            visibility_timeout: 30,
            max_retries: 3,
            region: None,
        }
    }
}

/// SqsQueueManager implements queue operations using AWS SQS
///
/// This implementation provides:
/// - Asynchronous job processing with configurable concurrency
/// - Batch processing for efficient message retrieval
/// - Long polling to reduce API calls
/// - Automatic retry logic using SQS visibility timeout
///
/// Requirements: 9.3, 9.4, 9.6
pub struct SqsQueueManager {
    /// SQS client for queue operations
    client: Client,

    /// Configuration
    config: SqsQueueConfig,

    /// WebhookSender for processing jobs
    webhook_sender: Arc<WebhookSender>,

    /// Worker task handles
    workers: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>>,

    /// Flag to signal workers to stop
    shutdown: Arc<AtomicBool>,
}

impl SqsQueueManager {
    /// Create a new SqsQueueManager with the given configuration
    pub async fn new(config: SqsQueueConfig, webhook_sender: Arc<WebhookSender>) -> Result<Self> {
        // Load AWS configuration
        let aws_config = if let Some(region) = &config.region {
            aws_config::defaults(BehaviorVersion::latest())
                .region(aws_sdk_sqs::config::Region::new(region.clone()))
                .load()
                .await
        } else {
            aws_config::load_defaults(BehaviorVersion::latest()).await
        };

        let client = Client::new(&aws_config);

        let manager = Self {
            client,
            config,
            webhook_sender,
            workers: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            shutdown: Arc::new(AtomicBool::new(false)),
        };

        Ok(manager)
    }

    /// Start worker tasks to process jobs from the queue
    pub async fn start_workers(&self) -> Result<()> {
        let mut workers = self.workers.lock().await;

        // Spawn worker tasks based on configured concurrency
        for worker_id in 0..self.config.concurrency {
            let client = self.client.clone();
            let webhook_sender = self.webhook_sender.clone();
            let shutdown = self.shutdown.clone();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                Self::worker_loop(worker_id, client, webhook_sender, shutdown, config).await;
            });

            workers.push(handle);
        }

        Ok(())
    }

    /// Worker loop that processes jobs from the queue
    async fn worker_loop(
        worker_id: usize,
        client: Client,
        webhook_sender: Arc<WebhookSender>,
        shutdown: Arc<AtomicBool>,
        config: SqsQueueConfig,
    ) {
        crate::log::Log::info(&format!("SQS queue worker {} started", worker_id));

        while !shutdown.load(Ordering::Relaxed) {
            // Receive messages from SQS with long polling
            let result = client
                .receive_message()
                .queue_url(&config.queue_url)
                .max_number_of_messages(config.batch_size)
                .wait_time_seconds(config.wait_time_seconds)
                .visibility_timeout(config.visibility_timeout)
                .send()
                .await;

            match result {
                Ok(output) => {
                    if let Some(messages) = output.messages {
                        // Process messages in batch
                        for message in messages {
                            if let Some(body) = &message.body {
                                // Parse the job
                                match serde_json::from_str::<WebhookJob>(body) {
                                    Ok(job) => {
                                        // SQS automatically handles retries via visibility timeout
                                        // We don't need to track receive count manually for basic retry logic
                                        // The message will become visible again if not deleted

                                        // Process the job
                                        let success =
                                            Self::process_job(&webhook_sender, &job).await;

                                        if success {
                                            // Delete message on success
                                            if let Some(receipt_handle) = &message.receipt_handle {
                                                let delete_result = client
                                                    .delete_message()
                                                    .queue_url(&config.queue_url)
                                                    .receipt_handle(receipt_handle)
                                                    .send()
                                                    .await;

                                                if let Err(e) = delete_result {
                                                    crate::log::Log::error(&format!(
                                                        "Worker {} failed to delete message: {}",
                                                        worker_id, e
                                                    ));
                                                }
                                            }
                                        } else {
                                            // On failure, don't delete the message
                                            // Let visibility timeout expire so message becomes available again
                                            // SQS will automatically retry based on the queue's redrive policy
                                            crate::log::Log::warning(&format!(
                                                "Job failed, will retry via visibility timeout: event {} on channel {}",
                                                job.event_type, job.channel
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        crate::log::Log::error(&format!(
                                            "Worker {} failed to parse job: {}",
                                            worker_id, e
                                        ));

                                        // Delete invalid message
                                        if let Some(receipt_handle) = &message.receipt_handle {
                                            let _ = client
                                                .delete_message()
                                                .queue_url(&config.queue_url)
                                                .receipt_handle(receipt_handle)
                                                .send()
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    crate::log::Log::error(&format!("Worker {} SQS error: {}", worker_id, e));
                    // Sleep before retrying to avoid tight loop on persistent errors
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }

        crate::log::Log::info(&format!("SQS queue worker {} stopped", worker_id));
    }

    /// Process a single job by calling the appropriate webhook sender method
    async fn process_job(webhook_sender: &Arc<WebhookSender>, job: &WebhookJob) -> bool {
        // Create a minimal App struct from the job data
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
            webhooks: vec![],
            max_presence_members_per_channel: None,
            max_presence_member_size_in_kb: None,
            max_channel_name_length: None,
            max_event_channels_at_once: None,
            max_event_name_length: None,
            max_event_payload_in_kb: None,
            max_event_batch_size: None,
            enable_user_authentication: false,
        };

        // Execute webhook based on event type
        match job.event_type.as_str() {
            "channel_occupied" => {
                webhook_sender
                    .send_channel_occupied(&app, &job.channel)
                    .await;
                true
            }
            "channel_vacated" => {
                webhook_sender
                    .send_channel_vacated(&app, &job.channel)
                    .await;
                true
            }
            "member_added" => {
                if let Some(user_id) = &job.user_id {
                    webhook_sender
                        .send_member_added(&app, &job.channel, user_id)
                        .await;
                    true
                } else {
                    crate::log::Log::warning(&format!(
                        "member_added job missing user_id for channel {}",
                        job.channel
                    ));
                    false
                }
            }
            "member_removed" => {
                if let Some(user_id) = &job.user_id {
                    webhook_sender
                        .send_member_removed(&app, &job.channel, user_id)
                        .await;
                    true
                } else {
                    crate::log::Log::warning(&format!(
                        "member_removed job missing user_id for channel {}",
                        job.channel
                    ));
                    false
                }
            }
            "client_event" => {
                if let (Some(event), Some(data)) = (&job.event, &job.data) {
                    webhook_sender
                        .send_client_event(
                            &app,
                            &job.channel,
                            event,
                            data.clone(),
                            job.socket_id.as_deref(),
                            job.user_id.as_deref(),
                        )
                        .await;
                    true
                } else {
                    crate::log::Log::warning(&format!(
                        "client_event job missing event or data for channel {}",
                        job.channel
                    ));
                    false
                }
            }
            "cache_miss" => {
                webhook_sender.send_cache_missed(&app, &job.channel).await;
                true
            }
            _ => {
                crate::log::Log::warning(&format!(
                    "Unknown webhook event type: {}",
                    job.event_type
                ));
                false
            }
        }
    }

    /// Get approximate number of messages in the queue
    pub async fn get_queue_length(&self) -> Result<i32> {
        let result = self
            .client
            .get_queue_attributes()
            .queue_url(&self.config.queue_url)
            .attribute_names(aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessages)
            .send()
            .await
            .map_err(|e| {
                PusherError::QueueError(format!("Failed to get queue attributes: {}", e))
            })?;

        let count = result
            .attributes
            .and_then(|attrs| {
                attrs
                    .get(&aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessages)
                    .cloned()
            })
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        Ok(count)
    }

    /// Get approximate number of messages not visible (being processed)
    pub async fn get_processing_length(&self) -> Result<i32> {
        let result = self
            .client
            .get_queue_attributes()
            .queue_url(&self.config.queue_url)
            .attribute_names(
                aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessagesNotVisible,
            )
            .send()
            .await
            .map_err(|e| {
                PusherError::QueueError(format!("Failed to get queue attributes: {}", e))
            })?;

        let count = result
            .attributes
            .and_then(|attrs| attrs.get(&aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessagesNotVisible).cloned())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        Ok(count)
    }
}

#[async_trait]
impl QueueManager for SqsQueueManager {
    /// Enqueue a webhook job for asynchronous processing
    ///
    /// Jobs are sent to the SQS queue and will be processed by worker tasks.
    async fn enqueue(&self, job: WebhookJob) -> Result<()> {
        let job_data = serde_json::to_string(&job)
            .map_err(|e| PusherError::SerializationError(e.to_string()))?;

        self.client
            .send_message()
            .queue_url(&self.config.queue_url)
            .message_body(job_data)
            .send()
            .await
            .map_err(|e| {
                PusherError::QueueError(format!("Failed to send message to SQS: {}", e))
            })?;

        Ok(())
    }

    /// Disconnect and clean up queue resources
    ///
    /// This signals all worker tasks to stop and waits for them to complete.
    async fn disconnect(&self) -> Result<()> {
        // Signal workers to stop
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for all workers to finish
        let mut workers = self.workers.lock().await;
        while let Some(handle) = workers.pop() {
            let _ = handle.await;
        }

        crate::log::Log::info("SQS queue manager disconnected");

        Ok(())
    }
}

// Legacy SqsQueueDriver for backward compatibility
pub struct SqsQueueDriver {
    client: Client,
    queue_url: String,
}

impl SqsQueueDriver {
    pub async fn new(queue_url: String) -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        Self { client, queue_url }
    }
}

#[async_trait]
impl Queue for SqsQueueDriver {
    async fn add_to_queue(&self, _queue_name: &str, data: Value) {
        // SQS usually has one URL per queue. queue_name might correspond to different URLs
        // or we ignore queue_name if the driver is bound to a specific URL.
        // Soketi SQS driver config has `queueUrl`.
        let _ = self
            .client
            .send_message()
            .queue_url(&self.queue_url)
            .message_body(data.to_string())
            .send()
            .await;
    }

    async fn process_queue(&self, _queue_name: &str, handler: JobHandler) {
        let client = self.client.clone();
        let queue_url = self.queue_url.clone();

        tokio::spawn(async move {
            loop {
                let resp = client
                    .receive_message()
                    .queue_url(&queue_url)
                    .max_number_of_messages(10) // Batch size
                    .wait_time_seconds(20) // Long polling
                    .send()
                    .await;

                if let Ok(output) = resp
                    && let Some(messages) = output.messages
                {
                    for message in messages {
                        if let Some(body) = message.body
                            && let Ok(data) = serde_json::from_str::<Value>(&body)
                        {
                            handler(data).await;

                            // Delete message
                            if let Some(receipt_handle) = message.receipt_handle {
                                let _ = client
                                    .delete_message()
                                    .queue_url(&queue_url)
                                    .receipt_handle(receipt_handle)
                                    .send()
                                    .await;
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // Note: Integration tests with actual SQS are in tests/sqs_queue_manager_test.rs
}
