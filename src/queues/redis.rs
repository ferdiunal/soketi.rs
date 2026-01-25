use crate::error::{PusherError, Result};
use crate::queues::{JobHandler, Queue, QueueManager, WebhookJob};
use crate::webhook_sender::WebhookSender;
use async_trait::async_trait;
use redis::{AsyncCommands, Client, RedisError};
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

/// Configuration for RedisQueueManager
#[derive(Debug, Clone)]
pub struct RedisQueueConfig {
    /// Redis connection URL
    pub redis_url: String,

    /// Number of concurrent worker tasks
    pub concurrency: usize,

    /// Maximum number of retry attempts for failed jobs
    pub max_retries: u32,

    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,

    /// Queue name prefix
    pub queue_prefix: String,
}

impl Default for RedisQueueConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            concurrency: 1,
            max_retries: 3,
            retry_delay_ms: 1000,
            queue_prefix: "pusher:queue".to_string(),
        }
    }
}

/// RedisQueueManager implements BullMQ-compatible queue operations using Redis
///
/// This implementation provides:
/// - Asynchronous job processing with configurable concurrency
/// - Automatic retry logic with exponential backoff
/// - BullMQ-compatible data structures for interoperability
///
/// Requirements: 9.2, 9.4, 9.5
pub struct RedisQueueManager {
    /// Redis client for queue operations
    client: Client,

    /// Configuration
    config: RedisQueueConfig,

    /// WebhookSender for processing jobs
    webhook_sender: Arc<WebhookSender>,

    /// Worker task handles
    workers: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>>,

    /// Flag to signal workers to stop
    shutdown: Arc<AtomicBool>,
}

impl RedisQueueManager {
    /// Create a new RedisQueueManager with the given configuration
    pub fn new(config: RedisQueueConfig, webhook_sender: Arc<WebhookSender>) -> Result<Self> {
        let client = Client::open(config.redis_url.as_str())
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

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
        config: RedisQueueConfig,
    ) {
        crate::log::Log::info(&format!("Redis queue worker {} started", worker_id));

        // Get a connection for this worker
        let mut conn = match client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                crate::log::Log::error(&format!(
                    "Worker {} failed to connect to Redis: {}",
                    worker_id, e
                ));
                return;
            }
        };

        let queue_name = format!("{}:jobs", config.queue_prefix);
        let processing_queue = format!("{}:processing", config.queue_prefix);

        while !shutdown.load(Ordering::Relaxed) {
            // Use BRPOPLPUSH to atomically move job from main queue to processing queue
            // This provides reliability - if worker crashes, job remains in processing queue
            let result: std::result::Result<Option<String>, RedisError> = redis::cmd("BRPOPLPUSH")
                .arg(&queue_name)
                .arg(&processing_queue)
                .arg(1) // 1 second timeout
                .query_async(&mut conn)
                .await;

            match result {
                Ok(Some(job_data)) => {
                    // Parse the job
                    match serde_json::from_str::<WebhookJob>(&job_data) {
                        Ok(job) => {
                            // Process the job with retry logic
                            let success = Self::process_job_with_retry(
                                &webhook_sender,
                                &job,
                                config.max_retries,
                                config.retry_delay_ms,
                            )
                            .await;

                            if success {
                                // Remove from processing queue on success
                                let _: std::result::Result<(), RedisError> =
                                    conn.lrem(&processing_queue, 1, &job_data).await;
                            } else {
                                // Move to failed queue
                                let failed_queue = format!("{}:failed", config.queue_prefix);
                                let _: std::result::Result<(), RedisError> =
                                    conn.rpush(&failed_queue, &job_data).await;
                                let _: std::result::Result<(), RedisError> =
                                    conn.lrem(&processing_queue, 1, &job_data).await;
                            }
                        }
                        Err(e) => {
                            crate::log::Log::error(&format!(
                                "Worker {} failed to parse job: {}",
                                worker_id, e
                            ));
                            // Remove invalid job from processing queue
                            let _: std::result::Result<(), RedisError> =
                                conn.lrem(&processing_queue, 1, &job_data).await;
                        }
                    }
                }
                Ok(None) => {
                    // Timeout, continue loop
                }
                Err(e) => {
                    crate::log::Log::error(&format!("Worker {} Redis error: {}", worker_id, e));
                    // Sleep before retrying to avoid tight loop on persistent errors
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        crate::log::Log::info(&format!("Redis queue worker {} stopped", worker_id));
    }

    /// Process a job with retry logic
    async fn process_job_with_retry(
        webhook_sender: &Arc<WebhookSender>,
        job: &WebhookJob,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> bool {
        let mut attempts = 0;

        while attempts <= max_retries {
            if attempts > 0 {
                // Exponential backoff: delay * 2^(attempts-1)
                let delay = retry_delay_ms * (1 << (attempts - 1));
                sleep(Duration::from_millis(delay)).await;

                crate::log::Log::info(&format!(
                    "Retrying job (attempt {}/{}) for event {} on channel {}",
                    attempts, max_retries, job.event_type, job.channel
                ));
            }

            // Process the job
            let success = Self::process_job(webhook_sender, job).await;

            if success {
                return true;
            }

            attempts += 1;
        }

        crate::log::Log::error(&format!(
            "Job failed after {} attempts: event {} on channel {}",
            max_retries + 1,
            job.event_type,
            job.channel
        ));

        false
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

    /// Get the number of jobs in the queue
    pub async fn get_queue_length(&self) -> Result<usize> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        let queue_name = format!("{}:jobs", self.config.queue_prefix);
        let length: usize = conn
            .llen(&queue_name)
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        Ok(length)
    }

    /// Get the number of jobs in the processing queue
    pub async fn get_processing_length(&self) -> Result<usize> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        let processing_queue = format!("{}:processing", self.config.queue_prefix);
        let length: usize = conn
            .llen(&processing_queue)
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        Ok(length)
    }

    /// Get the number of failed jobs
    pub async fn get_failed_length(&self) -> Result<usize> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        let failed_queue = format!("{}:failed", self.config.queue_prefix);
        let length: usize = conn
            .llen(&failed_queue)
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        Ok(length)
    }
}

#[async_trait]
impl QueueManager for RedisQueueManager {
    /// Enqueue a webhook job for asynchronous processing
    ///
    /// Jobs are added to a Redis list in BullMQ-compatible format.
    /// Workers will pick up jobs from the queue and process them with retry logic.
    async fn enqueue(&self, job: WebhookJob) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

        let queue_name = format!("{}:jobs", self.config.queue_prefix);
        let job_data = serde_json::to_string(&job)
            .map_err(|e| PusherError::SerializationError(e.to_string()))?;

        // Add job to the left of the queue (LPUSH) so workers can pop from the right (BRPOPLPUSH)
        let _: i32 = conn
            .lpush(&queue_name, job_data)
            .await
            .map_err(|e| PusherError::RedisError(e.to_string()))?;

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

        crate::log::Log::info("Redis queue manager disconnected");

        Ok(())
    }
}

// Legacy RedisQueueDriver for backward compatibility
pub struct RedisQueueDriver {
    client: Client,
}

impl RedisQueueDriver {
    pub fn new(url: &str) -> Self {
        let client = Client::open(url).expect("Invalid Redis URL");
        Self { client }
    }
}

#[async_trait]
impl Queue for RedisQueueDriver {
    async fn add_to_queue(&self, queue_name: &str, data: Value) {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            let _: std::result::Result<(), redis::RedisError> =
                conn.rpush(queue_name, data.to_string()).await;
        }
    }

    async fn process_queue(&self, queue_name: &str, handler: JobHandler) {
        let client = self.client.clone();
        let queue_name = queue_name.to_string();

        tokio::spawn(async move {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                loop {
                    // BLPOP returns (key, value)
                    let result: std::result::Result<(String, String), redis::RedisError> =
                        conn.blpop(&queue_name, 0.0).await;
                    if let Ok((_, data_str)) = result
                        && let Ok(data) = serde_json::from_str::<Value>(&data_str)
                    {
                        handler(data).await;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_queue_config_default() {
        let config = RedisQueueConfig::default();
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.concurrency, 1);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.queue_prefix, "pusher:queue");
    }

    #[test]
    fn test_redis_queue_config_custom() {
        let config = RedisQueueConfig {
            redis_url: "redis://localhost:6380".to_string(),
            concurrency: 4,
            max_retries: 5,
            retry_delay_ms: 2000,
            queue_prefix: "custom:queue".to_string(),
        };

        assert_eq!(config.redis_url, "redis://localhost:6380");
        assert_eq!(config.concurrency, 4);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay_ms, 2000);
        assert_eq!(config.queue_prefix, "custom:queue");
    }

    // Note: Integration tests with actual Redis are in tests/redis_queue_manager_test.rs
}
