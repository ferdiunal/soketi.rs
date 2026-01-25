/// Test to verify the MetricsManager trait is properly defined
///
/// This test ensures that the MetricsManager trait has all required methods
/// as specified in the design document and requirements.
use async_trait::async_trait;
use serde_json::Value;
use soketi_rs::error::Result;
use soketi_rs::metrics::MetricsManager;

/// Mock implementation of MetricsManager for testing trait definition
struct MockMetricsManager {
    connections: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, i64>>>,
    messages_sent: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
    messages_received: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
    api_requests: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
    webhooks: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
}

impl MockMetricsManager {
    fn new() -> Self {
        Self {
            connections: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            messages_sent: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            messages_received: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            api_requests: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            webhooks: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl MetricsManager for MockMetricsManager {
    async fn mark_new_connection(&self, app_id: &str) {
        let mut connections = self.connections.lock().unwrap();
        *connections.entry(app_id.to_string()).or_insert(0) += 1;
    }

    async fn mark_disconnection(&self, app_id: &str) {
        let mut connections = self.connections.lock().unwrap();
        *connections.entry(app_id.to_string()).or_insert(0) -= 1;
    }

    async fn mark_ws_message_sent(&self, app_id: &str, _event: &str) {
        let mut messages = self.messages_sent.lock().unwrap();
        *messages.entry(app_id.to_string()).or_insert(0) += 1;
    }

    async fn mark_ws_message_received(&self, app_id: &str, _event: &str) {
        let mut messages = self.messages_received.lock().unwrap();
        *messages.entry(app_id.to_string()).or_insert(0) += 1;
    }

    async fn mark_api_message(&self, app_id: &str) {
        let mut requests = self.api_requests.lock().unwrap();
        *requests.entry(app_id.to_string()).or_insert(0) += 1;
    }

    async fn mark_webhook_sent(&self, app_id: &str, event_type: &str, success: bool) {
        let mut webhooks = self.webhooks.lock().unwrap();
        let key = format!("{}:{}:{}", app_id, event_type, success);
        *webhooks.entry(key).or_insert(0) += 1;
    }

    async fn get_metrics_as_plaintext(&self) -> Result<String> {
        Ok("# Mock metrics\n".to_string())
    }

    async fn get_metrics_as_json(&self) -> Result<Value> {
        Ok(serde_json::json!({"mock": "metrics"}))
    }

    async fn clear(&self) -> Result<()> {
        self.connections.lock().unwrap().clear();
        self.messages_sent.lock().unwrap().clear();
        self.messages_received.lock().unwrap().clear();
        self.api_requests.lock().unwrap().clear();
        self.webhooks.lock().unwrap().clear();
        Ok(())
    }
}

#[tokio::test]
async fn test_metrics_manager_trait_has_all_required_methods() {
    // This test verifies that the MetricsManager trait can be implemented
    // with all required methods as specified in Requirements 11.2, 11.3, 11.4, 11.5

    let metrics = MockMetricsManager::new();

    // Test connection tracking (Requirement 11.2)
    metrics.mark_new_connection("app1").await;
    metrics.mark_new_connection("app1").await;
    metrics.mark_disconnection("app1").await;

    let connections = metrics.connections.lock().unwrap();
    assert_eq!(*connections.get("app1").unwrap(), 1);
    drop(connections);

    // Test message tracking (Requirement 11.3)
    metrics
        .mark_ws_message_sent("app1", "pusher:connection_established")
        .await;
    metrics.mark_ws_message_sent("app1", "message").await;
    metrics
        .mark_ws_message_received("app1", "pusher:subscribe")
        .await;

    let sent = metrics.messages_sent.lock().unwrap();
    assert_eq!(*sent.get("app1").unwrap(), 2);
    drop(sent);

    let received = metrics.messages_received.lock().unwrap();
    assert_eq!(*received.get("app1").unwrap(), 1);
    drop(received);

    // Test API request tracking (Requirement 11.4)
    metrics.mark_api_message("app1").await;
    metrics.mark_api_message("app1").await;

    let api = metrics.api_requests.lock().unwrap();
    assert_eq!(*api.get("app1").unwrap(), 2);
    drop(api);

    // Test webhook tracking (Requirement 11.5)
    metrics
        .mark_webhook_sent("app1", "channel_occupied", true)
        .await;
    metrics
        .mark_webhook_sent("app1", "channel_occupied", false)
        .await;

    let webhooks = metrics.webhooks.lock().unwrap();
    assert_eq!(*webhooks.get("app1:channel_occupied:true").unwrap(), 1);
    assert_eq!(*webhooks.get("app1:channel_occupied:false").unwrap(), 1);
    drop(webhooks);

    // Test metrics export (Requirement 11.1)
    let plaintext = metrics.get_metrics_as_plaintext().await.unwrap();
    assert!(!plaintext.is_empty());

    let json = metrics.get_metrics_as_json().await.unwrap();
    assert!(json.is_object());

    // Test clear functionality
    metrics.clear().await.unwrap();
    let connections = metrics.connections.lock().unwrap();
    assert_eq!(connections.len(), 0);
}

#[tokio::test]
async fn test_metrics_manager_trait_can_be_used_as_trait_object() {
    // Verify that MetricsManager can be used as a trait object (Arc<dyn MetricsManager>)
    // This is important for the AppState design

    let metrics: std::sync::Arc<dyn MetricsManager> =
        std::sync::Arc::new(MockMetricsManager::new());

    // Should be able to call all methods through the trait object
    metrics.mark_new_connection("app1").await;
    metrics.mark_ws_message_sent("app1", "test").await;
    metrics.mark_api_message("app1").await;
    metrics.mark_webhook_sent("app1", "test", true).await;

    let plaintext = metrics.get_metrics_as_plaintext().await.unwrap();
    assert!(!plaintext.is_empty());

    let json = metrics.get_metrics_as_json().await.unwrap();
    assert!(json.is_object());
}

#[tokio::test]
async fn test_metrics_manager_connection_tracking() {
    // Test that connection tracking works correctly with multiple apps
    let metrics = MockMetricsManager::new();

    // App1: 3 connections
    metrics.mark_new_connection("app1").await;
    metrics.mark_new_connection("app1").await;
    metrics.mark_new_connection("app1").await;

    // App2: 2 connections
    metrics.mark_new_connection("app2").await;
    metrics.mark_new_connection("app2").await;

    // App1: 1 disconnection
    metrics.mark_disconnection("app1").await;

    let connections = metrics.connections.lock().unwrap();
    assert_eq!(*connections.get("app1").unwrap(), 2);
    assert_eq!(*connections.get("app2").unwrap(), 2);
}

#[tokio::test]
async fn test_metrics_manager_message_tracking_per_app() {
    // Test that messages are tracked separately per app
    let metrics = MockMetricsManager::new();

    // App1 messages
    metrics.mark_ws_message_sent("app1", "event1").await;
    metrics.mark_ws_message_sent("app1", "event2").await;
    metrics.mark_ws_message_received("app1", "event3").await;

    // App2 messages
    metrics.mark_ws_message_sent("app2", "event1").await;
    metrics.mark_ws_message_received("app2", "event2").await;
    metrics.mark_ws_message_received("app2", "event3").await;

    let sent = metrics.messages_sent.lock().unwrap();
    assert_eq!(*sent.get("app1").unwrap(), 2);
    assert_eq!(*sent.get("app2").unwrap(), 1);
    drop(sent);

    let received = metrics.messages_received.lock().unwrap();
    assert_eq!(*received.get("app1").unwrap(), 1);
    assert_eq!(*received.get("app2").unwrap(), 2);
}

#[tokio::test]
async fn test_metrics_manager_webhook_tracking_with_success_failure() {
    // Test that webhooks track both success and failure separately
    let metrics = MockMetricsManager::new();

    // Successful webhooks
    metrics
        .mark_webhook_sent("app1", "channel_occupied", true)
        .await;
    metrics
        .mark_webhook_sent("app1", "channel_occupied", true)
        .await;
    metrics
        .mark_webhook_sent("app1", "member_added", true)
        .await;

    // Failed webhooks
    metrics
        .mark_webhook_sent("app1", "channel_occupied", false)
        .await;
    metrics
        .mark_webhook_sent("app1", "member_added", false)
        .await;
    metrics
        .mark_webhook_sent("app1", "member_added", false)
        .await;

    let webhooks = metrics.webhooks.lock().unwrap();
    assert_eq!(*webhooks.get("app1:channel_occupied:true").unwrap(), 2);
    assert_eq!(*webhooks.get("app1:channel_occupied:false").unwrap(), 1);
    assert_eq!(*webhooks.get("app1:member_added:true").unwrap(), 1);
    assert_eq!(*webhooks.get("app1:member_added:false").unwrap(), 2);
}
