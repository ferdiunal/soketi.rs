use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

pub mod prometheus;

/// MetricsManager trait for tracking server metrics
///
/// This trait defines the interface for collecting and exposing metrics about
/// server performance and usage. Implementations should track:
/// - Connection counts per app
/// - Message counts (sent and received) per app
/// - API request counts per app
/// - Webhook delivery counts and failures
///
/// Metrics can be exported in Prometheus plaintext format or as JSON.
#[async_trait]
pub trait MetricsManager: Send + Sync {
    /// Mark a new WebSocket connection for an app
    ///
    /// This should increment the connection count gauge for the specified app.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    ///
    /// **Validates: Requirements 11.2**
    async fn mark_new_connection(&self, app_id: &str);

    /// Mark a WebSocket disconnection for an app
    ///
    /// This should decrement the connection count gauge for the specified app.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    ///
    /// **Validates: Requirements 11.2**
    async fn mark_disconnection(&self, app_id: &str);

    /// Mark a WebSocket message sent to clients
    ///
    /// This should increment the messages sent counter for the specified app and event type.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    /// * `event` - The event name (e.g., "pusher:connection_established", "message")
    ///
    /// **Validates: Requirements 11.3**
    async fn mark_ws_message_sent(&self, app_id: &str, event: &str);

    /// Mark a WebSocket message received from clients
    ///
    /// This should increment the messages received counter for the specified app and event type.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    /// * `event` - The event name (e.g., "pusher:subscribe", "client-event")
    ///
    /// **Validates: Requirements 11.3**
    async fn mark_ws_message_received(&self, app_id: &str, event: &str);

    /// Mark an HTTP API request
    ///
    /// This should increment the API request counter for the specified app.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    ///
    /// **Validates: Requirements 11.4**
    async fn mark_api_message(&self, app_id: &str);

    /// Mark a webhook sent
    ///
    /// This should increment the webhook counter for the specified app, event type,
    /// and success/failure status.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    /// * `event_type` - The webhook event type (e.g., "channel_occupied", "member_added")
    /// * `success` - Whether the webhook was delivered successfully
    ///
    /// **Validates: Requirements 11.5**
    async fn mark_webhook_sent(&self, app_id: &str, event_type: &str, success: bool);

    /// Get metrics in Prometheus plaintext format
    ///
    /// Returns metrics in the Prometheus exposition format, suitable for scraping
    /// by Prometheus or compatible monitoring systems.
    ///
    /// # Returns
    /// A string containing metrics in Prometheus plaintext format
    ///
    /// **Validates: Requirements 11.1**
    async fn get_metrics_as_plaintext(&self) -> Result<String>;

    /// Get metrics as JSON
    ///
    /// Returns metrics as a JSON object, useful for programmatic access or
    /// custom monitoring dashboards.
    ///
    /// # Returns
    /// A JSON value containing all metrics
    ///
    /// **Validates: Requirements 11.1**
    async fn get_metrics_as_json(&self) -> Result<Value>;

    /// Clear all metrics
    ///
    /// Resets all counters and gauges to zero. This is primarily useful for testing.
    ///
    /// # Returns
    /// Ok(()) on success, or an error if clearing fails
    async fn clear(&self) -> Result<()>;
}

// Legacy trait for backward compatibility
// TODO: Remove this once all code is migrated to MetricsManager
#[async_trait]
pub trait Metrics: Send + Sync {
    async fn mark_api_message(&self, app_id: &str, payload: Value, result: Value);
    async fn get_metrics_as_plaintext(&self) -> String;
    async fn get_metrics_as_json(&self) -> Value;
}
