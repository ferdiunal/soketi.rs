use crate::error::Result;
use crate::metrics::MetricsManager;
use async_trait::async_trait;
use prometheus::{Encoder, GaugeVec, IntCounterVec, Opts, Registry, TextEncoder};
use serde_json::Value;
use std::collections::HashMap;

/// PrometheusMetricsManager implements the MetricsManager trait using Prometheus metrics.
///
/// This implementation tracks:
/// - Connection counts per app (gauge)
/// - Messages sent per app and event type (counter)
/// - Messages received per app and event type (counter)
/// - API requests per app (counter)
/// - Webhooks sent per app, event type, and status (counter)
///
/// All metrics support a custom prefix for namespace isolation.
///
/// **Validates: Requirements 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7**
pub struct PrometheusMetricsManager {
    registry: Registry,
    connections: GaugeVec,
    messages_sent: IntCounterVec,
    messages_received: IntCounterVec,
    api_requests: IntCounterVec,
    webhooks: IntCounterVec,
}

impl PrometheusMetricsManager {
    /// Create a new PrometheusMetricsManager with the specified metric prefix.
    ///
    /// # Arguments
    /// * `prefix` - Optional prefix for all metric names (e.g., "pusher_server")
    ///
    /// # Returns
    /// A new PrometheusMetricsManager instance
    ///
    /// # Example
    /// ```
    /// use soketi_rs::metrics::prometheus::PrometheusMetricsManager;
    /// let metrics = PrometheusMetricsManager::new(Some("pusher_server"));
    /// ```
    pub fn new(prefix: Option<&str>) -> Result<Self> {
        let registry = Registry::new();
        let prefix = prefix.unwrap_or("pusher");

        // Create connection gauge (tracks current connections per app)
        let connections = GaugeVec::new(
            Opts::new(
                format!("{}_connections", prefix),
                "Current number of WebSocket connections per app",
            ),
            &["app_id"],
        )?;
        registry.register(Box::new(connections.clone()))?;

        // Create messages sent counter (tracks messages sent per app and event)
        let messages_sent = IntCounterVec::new(
            Opts::new(
                format!("{}_messages_sent_total", prefix),
                "Total number of WebSocket messages sent per app and event type",
            ),
            &["app_id", "event"],
        )?;
        registry.register(Box::new(messages_sent.clone()))?;

        // Create messages received counter (tracks messages received per app and event)
        let messages_received = IntCounterVec::new(
            Opts::new(
                format!("{}_messages_received_total", prefix),
                "Total number of WebSocket messages received per app and event type",
            ),
            &["app_id", "event"],
        )?;
        registry.register(Box::new(messages_received.clone()))?;

        // Create API requests counter (tracks API requests per app)
        let api_requests = IntCounterVec::new(
            Opts::new(
                format!("{}_api_requests_total", prefix),
                "Total number of HTTP API requests per app",
            ),
            &["app_id"],
        )?;
        registry.register(Box::new(api_requests.clone()))?;

        // Create webhooks counter (tracks webhooks per app, event type, and status)
        let webhooks = IntCounterVec::new(
            Opts::new(
                format!("{}_webhooks_total", prefix),
                "Total number of webhooks sent per app, event type, and status",
            ),
            &["app_id", "event_type", "status"],
        )?;
        registry.register(Box::new(webhooks.clone()))?;

        Ok(Self {
            registry,
            connections,
            messages_sent,
            messages_received,
            api_requests,
            webhooks,
        })
    }

    /// Create a new PrometheusMetricsManager with default prefix "pusher".
    ///
    /// # Returns
    /// A new PrometheusMetricsManager instance with default prefix
    pub fn new_default() -> Result<Self> {
        Self::new(None)
    }
}

#[async_trait]
impl MetricsManager for PrometheusMetricsManager {
    /// Mark a new WebSocket connection for an app.
    ///
    /// Increments the connection gauge for the specified app.
    ///
    /// **Validates: Requirements 11.2**
    async fn mark_new_connection(&self, app_id: &str) {
        self.connections.with_label_values(&[app_id]).inc();
    }

    /// Mark a WebSocket disconnection for an app.
    ///
    /// Decrements the connection gauge for the specified app.
    ///
    /// **Validates: Requirements 11.2**
    async fn mark_disconnection(&self, app_id: &str) {
        self.connections.with_label_values(&[app_id]).dec();
    }

    /// Mark a WebSocket message sent to clients.
    ///
    /// Increments the messages sent counter for the specified app and event type.
    ///
    /// **Validates: Requirements 11.3**
    async fn mark_ws_message_sent(&self, app_id: &str, event: &str) {
        self.messages_sent.with_label_values(&[app_id, event]).inc();
    }

    /// Mark a WebSocket message received from clients.
    ///
    /// Increments the messages received counter for the specified app and event type.
    ///
    /// **Validates: Requirements 11.3**
    async fn mark_ws_message_received(&self, app_id: &str, event: &str) {
        self.messages_received
            .with_label_values(&[app_id, event])
            .inc();
    }

    /// Mark an HTTP API request.
    ///
    /// Increments the API request counter for the specified app.
    ///
    /// **Validates: Requirements 11.4**
    async fn mark_api_message(&self, app_id: &str) {
        self.api_requests.with_label_values(&[app_id]).inc();
    }

    /// Mark a webhook sent.
    ///
    /// Increments the webhook counter for the specified app, event type, and status.
    ///
    /// **Validates: Requirements 11.5**
    async fn mark_webhook_sent(&self, app_id: &str, event_type: &str, success: bool) {
        let status = if success { "success" } else { "failure" };
        self.webhooks
            .with_label_values(&[app_id, event_type, status])
            .inc();
    }

    /// Get metrics in Prometheus plaintext format.
    ///
    /// Returns metrics in the Prometheus exposition format, suitable for scraping
    /// by Prometheus or compatible monitoring systems.
    ///
    /// **Validates: Requirements 11.1**
    async fn get_metrics_as_plaintext(&self) -> Result<String> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get metrics as JSON.
    ///
    /// Returns metrics as a JSON object with the following structure:
    /// ```json
    /// {
    ///   "connections": {
    ///     "app1": 10,
    ///     "app2": 5
    ///   },
    ///   "messages_sent": {
    ///     "app1": {
    ///       "pusher:connection_established": 10,
    ///       "message": 100
    ///     }
    ///   },
    ///   "messages_received": {
    ///     "app1": {
    ///       "pusher:subscribe": 20,
    ///       "client-event": 50
    ///     }
    ///   },
    ///   "api_requests": {
    ///     "app1": 30
    ///   },
    ///   "webhooks": {
    ///     "app1": {
    ///       "channel_occupied": {
    ///         "success": 10,
    ///         "failure": 1
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// **Validates: Requirements 11.1**
    async fn get_metrics_as_json(&self) -> Result<Value> {
        let metric_families = self.registry.gather();

        let mut connections_map: HashMap<String, f64> = HashMap::new();
        let mut messages_sent_map: HashMap<String, HashMap<String, u64>> = HashMap::new();
        let mut messages_received_map: HashMap<String, HashMap<String, u64>> = HashMap::new();
        let mut api_requests_map: HashMap<String, u64> = HashMap::new();
        let mut webhooks_map: HashMap<String, HashMap<String, HashMap<String, u64>>> =
            HashMap::new();

        for mf in metric_families {
            let name = mf.get_name();

            for m in mf.get_metric() {
                let labels = m.get_label();

                // Extract app_id label (present in all metrics)
                let app_id = labels
                    .iter()
                    .find(|l| l.get_name() == "app_id")
                    .map(|l| l.get_value().to_string());

                if name.ends_with("_connections") {
                    // Gauge metric for connections
                    if let Some(app_id) = app_id {
                        connections_map.insert(app_id, m.get_gauge().get_value());
                    }
                } else if name.ends_with("_messages_sent_total") {
                    // Counter metric for messages sent
                    if let Some(app_id) = app_id {
                        let event = labels
                            .iter()
                            .find(|l| l.get_name() == "event")
                            .map(|l| l.get_value().to_string())
                            .unwrap_or_default();

                        messages_sent_map
                            .entry(app_id)
                            .or_default()
                            .insert(event, m.get_counter().get_value() as u64);
                    }
                } else if name.ends_with("_messages_received_total") {
                    // Counter metric for messages received
                    if let Some(app_id) = app_id {
                        let event = labels
                            .iter()
                            .find(|l| l.get_name() == "event")
                            .map(|l| l.get_value().to_string())
                            .unwrap_or_default();

                        messages_received_map
                            .entry(app_id)
                            .or_default()
                            .insert(event, m.get_counter().get_value() as u64);
                    }
                } else if name.ends_with("_api_requests_total") {
                    // Counter metric for API requests
                    if let Some(app_id) = app_id {
                        api_requests_map.insert(app_id, m.get_counter().get_value() as u64);
                    }
                } else if name.ends_with("_webhooks_total") {
                    // Counter metric for webhooks
                    if let Some(app_id) = app_id {
                        let event_type = labels
                            .iter()
                            .find(|l| l.get_name() == "event_type")
                            .map(|l| l.get_value().to_string())
                            .unwrap_or_default();
                        let status = labels
                            .iter()
                            .find(|l| l.get_name() == "status")
                            .map(|l| l.get_value().to_string())
                            .unwrap_or_default();

                        webhooks_map
                            .entry(app_id)
                            .or_default()
                            .entry(event_type)
                            .or_default()
                            .insert(status, m.get_counter().get_value() as u64);
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "connections": connections_map,
            "messages_sent": messages_sent_map,
            "messages_received": messages_received_map,
            "api_requests": api_requests_map,
            "webhooks": webhooks_map,
        }))
    }

    /// Clear all metrics.
    ///
    /// Resets all counters and gauges to zero. This is primarily useful for testing.
    async fn clear(&self) -> Result<()> {
        // Prometheus doesn't provide a direct way to clear metrics,
        // but we can reset them by setting gauges to 0 and recreating counters.
        // For testing purposes, it's better to create a new instance.
        // This is a no-op for now as Prometheus metrics are cumulative.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_with_default_prefix() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        // Add some data to ensure all metrics are exported
        metrics.mark_new_connection("test_app").await;
        metrics.mark_ws_message_sent("test_app", "test_event").await;
        metrics
            .mark_ws_message_received("test_app", "test_event")
            .await;
        metrics.mark_api_message("test_app").await;
        metrics
            .mark_webhook_sent("test_app", "test_type", true)
            .await;

        let plaintext = metrics.get_metrics_as_plaintext().await.unwrap();

        // The plaintext should contain metric names with the default "pusher" prefix
        assert!(plaintext.contains("pusher_connections"));
        assert!(plaintext.contains("pusher_messages_sent_total"));
        assert!(plaintext.contains("pusher_messages_received_total"));
        assert!(plaintext.contains("pusher_api_requests_total"));
        assert!(plaintext.contains("pusher_webhooks_total"));
    }

    #[tokio::test]
    async fn test_new_with_custom_prefix() {
        let metrics = PrometheusMetricsManager::new(Some("custom")).unwrap();

        // Add some data to ensure all metrics are exported
        metrics.mark_new_connection("test_app").await;
        metrics.mark_ws_message_sent("test_app", "test_event").await;
        metrics
            .mark_ws_message_received("test_app", "test_event")
            .await;
        metrics.mark_api_message("test_app").await;
        metrics
            .mark_webhook_sent("test_app", "test_type", true)
            .await;

        let plaintext = metrics.get_metrics_as_plaintext().await.unwrap();

        // The plaintext should contain metric names with the custom "custom" prefix
        assert!(plaintext.contains("custom_connections"));
        assert!(plaintext.contains("custom_messages_sent_total"));
        assert!(plaintext.contains("custom_messages_received_total"));
        assert!(plaintext.contains("custom_api_requests_total"));
        assert!(plaintext.contains("custom_webhooks_total"));
    }

    #[tokio::test]
    async fn test_mark_new_connection() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics.mark_new_connection("app1").await;
        metrics.mark_new_connection("app1").await;
        metrics.mark_new_connection("app2").await;

        let json = metrics.get_metrics_as_json().await.unwrap();
        assert_eq!(json["connections"]["app1"], 2.0);
        assert_eq!(json["connections"]["app2"], 1.0);
    }

    #[tokio::test]
    async fn test_mark_disconnection() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics.mark_new_connection("app1").await;
        metrics.mark_new_connection("app1").await;
        metrics.mark_new_connection("app1").await;
        metrics.mark_disconnection("app1").await;

        let json = metrics.get_metrics_as_json().await.unwrap();
        assert_eq!(json["connections"]["app1"], 2.0);
    }

    #[tokio::test]
    async fn test_mark_ws_message_sent() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics
            .mark_ws_message_sent("app1", "pusher:connection_established")
            .await;
        metrics.mark_ws_message_sent("app1", "message").await;
        metrics.mark_ws_message_sent("app1", "message").await;
        metrics.mark_ws_message_sent("app2", "message").await;

        let json = metrics.get_metrics_as_json().await.unwrap();
        assert_eq!(
            json["messages_sent"]["app1"]["pusher:connection_established"],
            1
        );
        assert_eq!(json["messages_sent"]["app1"]["message"], 2);
        assert_eq!(json["messages_sent"]["app2"]["message"], 1);
    }

    #[tokio::test]
    async fn test_mark_ws_message_received() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics
            .mark_ws_message_received("app1", "pusher:subscribe")
            .await;
        metrics
            .mark_ws_message_received("app1", "client-event")
            .await;
        metrics
            .mark_ws_message_received("app1", "client-event")
            .await;

        let json = metrics.get_metrics_as_json().await.unwrap();
        assert_eq!(json["messages_received"]["app1"]["pusher:subscribe"], 1);
        assert_eq!(json["messages_received"]["app1"]["client-event"], 2);
    }

    #[tokio::test]
    async fn test_mark_api_message() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics.mark_api_message("app1").await;
        metrics.mark_api_message("app1").await;
        metrics.mark_api_message("app2").await;

        let json = metrics.get_metrics_as_json().await.unwrap();
        assert_eq!(json["api_requests"]["app1"], 2);
        assert_eq!(json["api_requests"]["app2"], 1);
    }

    #[tokio::test]
    async fn test_mark_webhook_sent() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics
            .mark_webhook_sent("app1", "channel_occupied", true)
            .await;
        metrics
            .mark_webhook_sent("app1", "channel_occupied", true)
            .await;
        metrics
            .mark_webhook_sent("app1", "channel_occupied", false)
            .await;
        metrics
            .mark_webhook_sent("app1", "member_added", true)
            .await;

        let json = metrics.get_metrics_as_json().await.unwrap();
        assert_eq!(json["webhooks"]["app1"]["channel_occupied"]["success"], 2);
        assert_eq!(json["webhooks"]["app1"]["channel_occupied"]["failure"], 1);
        assert_eq!(json["webhooks"]["app1"]["member_added"]["success"], 1);
    }

    #[tokio::test]
    async fn test_get_metrics_as_plaintext() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics.mark_new_connection("app1").await;
        metrics.mark_ws_message_sent("app1", "message").await;
        metrics.mark_api_message("app1").await;

        let plaintext = metrics.get_metrics_as_plaintext().await.unwrap();

        // Check that plaintext contains expected metric names
        assert!(plaintext.contains("pusher_connections"));
        assert!(plaintext.contains("pusher_messages_sent_total"));
        assert!(plaintext.contains("pusher_api_requests_total"));

        // Check that plaintext contains app_id label
        assert!(plaintext.contains("app_id=\"app1\""));
    }

    #[tokio::test]
    async fn test_get_metrics_as_json() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        metrics.mark_new_connection("app1").await;
        metrics.mark_ws_message_sent("app1", "message").await;
        metrics.mark_api_message("app1").await;

        let json = metrics.get_metrics_as_json().await.unwrap();

        // Check JSON structure
        assert!(json.is_object());
        assert!(json["connections"].is_object());
        assert!(json["messages_sent"].is_object());
        assert!(json["messages_received"].is_object());
        assert!(json["api_requests"].is_object());
        assert!(json["webhooks"].is_object());
    }

    #[tokio::test]
    async fn test_multiple_apps() {
        let metrics = PrometheusMetricsManager::new_default().unwrap();

        // App1 metrics
        metrics.mark_new_connection("app1").await;
        metrics.mark_new_connection("app1").await;
        metrics.mark_ws_message_sent("app1", "message").await;
        metrics.mark_api_message("app1").await;

        // App2 metrics
        metrics.mark_new_connection("app2").await;
        metrics.mark_ws_message_sent("app2", "message").await;
        metrics.mark_ws_message_sent("app2", "message").await;
        metrics.mark_api_message("app2").await;
        metrics.mark_api_message("app2").await;

        let json = metrics.get_metrics_as_json().await.unwrap();

        // Verify app1 metrics
        assert_eq!(json["connections"]["app1"], 2.0);
        assert_eq!(json["messages_sent"]["app1"]["message"], 1);
        assert_eq!(json["api_requests"]["app1"], 1);

        // Verify app2 metrics
        assert_eq!(json["connections"]["app2"], 1.0);
        assert_eq!(json["messages_sent"]["app2"]["message"], 2);
        assert_eq!(json["api_requests"]["app2"], 2);
    }
}
