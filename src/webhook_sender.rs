use crate::app::App;
use crate::log::Log;
use hex;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Event data for webhooks
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientEventData {
    pub name: String,
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Webhook payload structure
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookPayload {
    pub time_ms: u64,
    pub events: Vec<ClientEventData>,
}

/// Configuration for webhook batching
#[derive(Debug, Clone)]
pub struct BatchingConfig {
    pub enabled: bool,
    pub duration_ms: u64,
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            duration_ms: 50,
        }
    }
}

/// WebhookSender handles sending webhook notifications to configured endpoints
/// Supports both HTTP webhooks and AWS Lambda function invocation
/// Can batch multiple events together if batching is enabled
#[derive(Clone)]
pub struct WebhookSender {
    /// HTTP client for sending webhook requests
    http_client: Client,

    /// Optional AWS Lambda client for invoking Lambda functions
    lambda_client: Option<Arc<aws_sdk_lambda::Client>>,

    /// Batching configuration
    batching_config: BatchingConfig,

    /// Pending batches per app key
    /// Maps `app_key -> Vec<ClientEventData>`
    pending_batches: Arc<Mutex<HashMap<String, Vec<ClientEventData>>>>,

    /// Flag to track if batch leader is active
    batch_has_leader: Arc<Mutex<bool>>,
}

impl WebhookSender {
    /// Create a new WebhookSender with default configuration (no batching, no Lambda)
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            lambda_client: None,
            batching_config: BatchingConfig::default(),
            pending_batches: Arc::new(Mutex::new(HashMap::new())),
            batch_has_leader: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new WebhookSender with custom configuration
    pub async fn with_config(batching_config: BatchingConfig, enable_lambda: bool) -> Self {
        let lambda_client = if enable_lambda {
            // Initialize AWS Lambda client
            let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            Some(Arc::new(aws_sdk_lambda::Client::new(&aws_config)))
        } else {
            None
        };

        Self {
            http_client: Client::new(),
            lambda_client,
            batching_config,
            pending_batches: Arc::new(Mutex::new(HashMap::new())),
            batch_has_leader: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new WebhookSender with a custom HTTP client
    pub fn with_http_client(http_client: Client) -> Self {
        Self {
            http_client,
            lambda_client: None,
            batching_config: BatchingConfig::default(),
            pending_batches: Arc::new(Mutex::new(HashMap::new())),
            batch_has_leader: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the Lambda client (useful for testing or custom configuration)
    pub fn with_lambda_client(mut self, lambda_client: aws_sdk_lambda::Client) -> Self {
        self.lambda_client = Some(Arc::new(lambda_client));
        self
    }

    /// Enable or disable batching
    pub fn with_batching(mut self, enabled: bool, duration_ms: u64) -> Self {
        self.batching_config = BatchingConfig {
            enabled,
            duration_ms,
        };
        self
    }

    /// Send a channel_occupied webhook when a channel becomes occupied (0 -> 1+ subscribers)
    pub async fn send_channel_occupied(&self, app: &App, channel: &str) {
        if !app.has_channel_occupied_webhooks() {
            return;
        }

        let event_data = ClientEventData {
            name: "channel_occupied".to_string(),
            channel: channel.to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
        };

        self.send(app, event_data, "channel_occupied_webhooks")
            .await;
    }

    /// Send a channel_vacated webhook when a channel becomes vacated (1+ -> 0 subscribers)
    pub async fn send_channel_vacated(&self, app: &App, channel: &str) {
        if !app.has_channel_vacated_webhooks() {
            return;
        }

        let event_data = ClientEventData {
            name: "channel_vacated".to_string(),
            channel: channel.to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
        };

        self.send(app, event_data, "channel_vacated_webhooks").await;
    }

    /// Send a member_added webhook when a member joins a presence channel
    pub async fn send_member_added(&self, app: &App, channel: &str, user_id: &str) {
        if !app.has_member_added_webhooks() {
            return;
        }

        let event_data = ClientEventData {
            name: "member_added".to_string(),
            channel: channel.to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: Some(user_id.to_string()),
        };

        self.send(app, event_data, "member_added_webhooks").await;
    }

    /// Send a member_removed webhook when a member leaves a presence channel
    pub async fn send_member_removed(&self, app: &App, channel: &str, user_id: &str) {
        if !app.has_member_removed_webhooks() {
            return;
        }

        let event_data = ClientEventData {
            name: "member_removed".to_string(),
            channel: channel.to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: Some(user_id.to_string()),
        };

        self.send(app, event_data, "member_removed_webhooks").await;
    }

    /// Send a client_event webhook when a client sends an event
    pub async fn send_client_event(
        &self,
        app: &App,
        channel: &str,
        event: &str,
        data: Value,
        socket_id: Option<&str>,
        user_id: Option<&str>,
    ) {
        if !app.has_client_event_webhooks() {
            return;
        }

        let event_data = ClientEventData {
            name: "client_event".to_string(),
            channel: channel.to_string(),
            event: Some(event.to_string()),
            data: Some(data),
            socket_id: socket_id.map(|s| s.to_string()),
            user_id: user_id.map(|s| s.to_string()),
        };

        self.send(app, event_data, "client_event_webhooks").await;
    }

    /// Send a cache_miss webhook when a cache miss occurs on a cache-enabled channel
    pub async fn send_cache_missed(&self, app: &App, channel: &str) {
        if !app.has_cache_missed_webhooks() {
            return;
        }

        let event_data = ClientEventData {
            name: "cache_miss".to_string(),
            channel: channel.to_string(),
            event: None,
            data: None,
            socket_id: None,
            user_id: None,
        };

        self.send(app, event_data, "cache_miss_webhooks").await;
    }

    async fn send(&self, app: &App, data: ClientEventData, _queue_name: &str) {
        // If batching is enabled, add to pending batch and start batch leader if needed
        if self.batching_config.enabled {
            self.add_to_batch(app, data).await;
        } else {
            // Send immediately without batching
            self.send_immediate(app, vec![data]).await;
        }
    }

    /// Add event to pending batch and start batch leader if needed
    async fn add_to_batch(&self, app: &App, data: ClientEventData) {
        let app_key = app.key.clone();

        // Add event to pending batch
        {
            let mut batches = self.pending_batches.lock().await;
            batches
                .entry(app_key.clone())
                .or_insert_with(Vec::new)
                .push(data);
        }

        // Start batch leader if not already running
        let mut has_leader = self.batch_has_leader.lock().await;
        if !*has_leader {
            *has_leader = true;
            drop(has_leader); // Release lock before spawning

            let sender = self.clone();
            let app_clone = app.clone();
            let duration = self.batching_config.duration_ms;

            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
                sender.flush_batch(&app_clone).await;
            });
        }
    }

    /// Flush pending batch for an app
    async fn flush_batch(&self, app: &App) {
        let app_key = app.key.clone();

        // Get and clear pending events
        let events = {
            let mut batches = self.pending_batches.lock().await;
            batches.remove(&app_key).unwrap_or_default()
        };

        // Reset leader flag
        {
            let mut has_leader = self.batch_has_leader.lock().await;
            *has_leader = false;
        }

        // Send batched events if any
        if !events.is_empty() {
            self.send_immediate(app, events).await;
        }
    }

    /// Send webhook immediately (with or without batching)
    async fn send_immediate(&self, app: &App, events: Vec<ClientEventData>) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let payload = WebhookPayload {
            time_ms: time,
            events: events.clone(),
        };

        let payload_json = serde_json::to_string(&payload).unwrap();
        let _signature = self.create_webhook_hmac(&payload_json, &app.secret);

        for webhook in &app.webhooks {
            // Filter events for this webhook
            let filtered_events: Vec<ClientEventData> = events
                .iter()
                .filter(|event| self.should_send_webhook(webhook, event))
                .cloned()
                .collect();

            if filtered_events.is_empty() {
                continue;
            }

            // Create payload with filtered events
            let filtered_payload = WebhookPayload {
                time_ms: time,
                events: filtered_events,
            };

            let filtered_payload_json = serde_json::to_string(&filtered_payload).unwrap();
            let filtered_signature = self.create_webhook_hmac(&filtered_payload_json, &app.secret);

            // Send to HTTP endpoint or Lambda
            if let Some(url) = &webhook.url {
                self.send_http_webhook(
                    app,
                    url,
                    webhook,
                    &filtered_payload_json,
                    &filtered_signature,
                )
                .await;
            } else if let Some(lambda_function) = &webhook.lambda_function {
                self.send_lambda_webhook(lambda_function, webhook, &filtered_payload_json)
                    .await;
            }
        }
    }

    /// Check if webhook should be sent based on event type and channel filters
    pub fn should_send_webhook(
        &self,
        webhook: &crate::app::Webhook,
        event: &ClientEventData,
    ) -> bool {
        // Check event type
        if !webhook.event_types.contains(&event.name) {
            return false;
        }

        // Check channel name filters if present
        if let Some(filter) = &webhook.filter {
            if let Some(prefix) = &filter.channel_name_starts_with
                && !event.channel.starts_with(prefix)
            {
                return false;
            }

            if let Some(suffix) = &filter.channel_name_ends_with
                && !event.channel.ends_with(suffix)
            {
                return false;
            }
        }

        true
    }

    /// Send webhook to HTTP endpoint
    async fn send_http_webhook(
        &self,
        app: &App,
        url: &str,
        webhook: &crate::app::Webhook,
        payload_json: &str,
        signature: &str,
    ) {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-Pusher-Key", app.key.parse().unwrap());
        headers.insert("X-Pusher-Signature", signature.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        // Add custom headers from webhook config
        if let Some(custom_headers) = &webhook.headers {
            for (key, value) in custom_headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(value),
                ) {
                    headers.insert(header_name, header_value);
                }
            }
        }

        let res = self
            .http_client
            .post(url)
            .headers(headers)
            .body(payload_json.to_string())
            .send()
            .await;

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    Log::info(&format!("Webhook sent successfully to {}", url));
                } else {
                    Log::error(&format!(
                        "Webhook failed with status {}: {}",
                        response.status(),
                        url
                    ));
                }
            }
            Err(e) => Log::error(&format!("Webhook request failed: {}", e)),
        }
    }

    /// Send webhook to AWS Lambda function
    async fn send_lambda_webhook(
        &self,
        function_name: &str,
        webhook: &crate::app::Webhook,
        payload_json: &str,
    ) {
        if let Some(lambda_client) = &self.lambda_client {
            let invocation_type = if webhook
                .lambda
                .as_ref()
                .map(|l| l.async_invocation)
                .unwrap_or(false)
            {
                aws_sdk_lambda::types::InvocationType::Event
            } else {
                aws_sdk_lambda::types::InvocationType::RequestResponse
            };

            let mut request = lambda_client
                .invoke()
                .function_name(function_name)
                .invocation_type(invocation_type)
                .payload(aws_sdk_lambda::primitives::Blob::new(
                    payload_json.as_bytes(),
                ));

            // Add client context if provided
            if let Some(lambda_config) = &webhook.lambda
                && let Some(client_context) = &lambda_config.client_context
                && let Ok(context_json) = serde_json::to_string(client_context)
            {
                request = request.client_context(context_json);
            }

            match request.send().await {
                Ok(_) => Log::info(&format!(
                    "Lambda webhook invoked successfully: {}",
                    function_name
                )),
                Err(e) => Log::error(&format!("Lambda webhook failed: {}", e)),
            }
        } else {
            Log::error("Lambda client not initialized but Lambda webhook configured");
        }
    }

    fn create_webhook_hmac(&self, data: &str, secret: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

impl Default for WebhookSender {
    fn default() -> Self {
        Self::new()
    }
}
