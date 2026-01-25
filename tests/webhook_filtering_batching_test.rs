use soketi_rs::app::{App, Webhook, WebhookFilter};
use soketi_rs::webhook_sender::{BatchingConfig, ClientEventData, WebhookSender};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper function to create a test app with webhooks
fn create_test_app_with_webhooks(webhooks: Vec<Webhook>) -> App {
    App {
        id: "test-app-id".to_string(),
        key: "test-app-key".to_string(),
        secret: "test-app-secret".to_string(),
        max_connections: None,
        enable_client_messages: true,
        enabled: true,
        max_backend_events_per_second: None,
        max_client_events_per_second: None,
        max_read_requests_per_second: None,
        webhooks,
        max_presence_members_per_channel: None,
        max_presence_member_size_in_kb: None,
        max_channel_name_length: None,
        max_event_channels_at_once: None,
        max_event_name_length: None,
        max_event_payload_in_kb: None,
        max_event_batch_size: None,
        enable_user_authentication: false,
    }
}

#[tokio::test]
async fn test_webhook_filtering_by_channel_prefix() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with channel prefix filter
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: None,
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should receive webhook for private channel
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook for private channel (should match filter)
    sender.send_channel_occupied(&app, "private-user-123").await;

    // Give it time to send
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify mock was called
    // (wiremock will panic if expectation not met)
}

#[tokio::test]
async fn test_webhook_filtering_by_channel_prefix_no_match() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with channel prefix filter
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: None,
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should NOT receive webhook for public channel
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0) // Expect NO calls
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook for public channel (should NOT match filter)
    sender.send_channel_occupied(&app, "public-channel").await;

    // Give it time to potentially send (it shouldn't)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify mock was NOT called
}

#[tokio::test]
async fn test_webhook_filtering_by_channel_suffix() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with channel suffix filter
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_vacated".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: None,
            channel_name_ends_with: Some("-notifications".to_string()),
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook for channel ending with -notifications (should match)
    sender
        .send_channel_vacated(&app, "user-123-notifications")
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_filtering_by_channel_suffix_no_match() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with channel suffix filter
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_vacated".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: None,
            channel_name_ends_with: Some("-notifications".to_string()),
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should NOT receive webhook
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook for channel NOT ending with -notifications
    sender.send_channel_vacated(&app, "user-123-messages").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_filtering_by_both_prefix_and_suffix() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with both prefix and suffix filters
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["member_added".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: Some("presence-".to_string()),
            channel_name_ends_with: Some("-room".to_string()),
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook for channel matching both filters
    sender
        .send_member_added(&app, "presence-chat-room", "user-123")
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_filtering_prefix_match_suffix_no_match() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with both prefix and suffix filters
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["member_added".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: Some("presence-".to_string()),
            channel_name_ends_with: Some("-room".to_string()),
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should NOT receive webhook
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook for channel matching prefix but not suffix
    sender
        .send_member_added(&app, "presence-chat-lobby", "user-123")
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_filtering_by_event_type() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook that only listens to channel_occupied events
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should only receive channel_occupied
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send channel_occupied (should be sent)
    sender.send_channel_occupied(&app, "test-channel").await;

    // Send channel_vacated (should NOT be sent)
    sender.send_channel_vacated(&app, "test-channel").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_batching_enabled() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should receive ONE batched request
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // Create sender with batching enabled
    let batching_config = BatchingConfig {
        enabled: true,
        duration_ms: 100,
    };
    let sender = WebhookSender::with_config(batching_config, false).await;

    // Send multiple webhooks quickly
    sender.send_channel_occupied(&app, "channel-1").await;
    sender.send_channel_occupied(&app, "channel-2").await;
    sender.send_channel_vacated(&app, "channel-3").await;

    // Wait for batch to be sent
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Should have received ONE request with multiple events
}

#[tokio::test]
async fn test_webhook_batching_disabled() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should receive THREE separate requests
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(3)
        .mount(&mock_server)
        .await;

    // Create sender with batching disabled
    let sender = WebhookSender::new();

    // Send multiple webhooks
    sender.send_channel_occupied(&app, "channel-1").await;
    sender.send_channel_occupied(&app, "channel-2").await;
    sender.send_channel_occupied(&app, "channel-3").await;

    // Wait for all to be sent
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Should have received THREE separate requests
}

#[tokio::test]
async fn test_webhook_custom_headers() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with custom headers
    let mut custom_headers = HashMap::new();
    custom_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    custom_headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: Some(custom_headers),
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation with header matchers
    Mock::given(method("POST"))
        .and(path("/"))
        .and(header("X-Custom-Header", "custom-value"))
        .and(header("Authorization", "Bearer token123"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    sender.send_channel_occupied(&app, "test-channel").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_signature_included() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - verify signature headers are present
    Mock::given(method("POST"))
        .and(path("/"))
        .and(header("X-Pusher-Key", "test-app-key"))
        .and(header_exists("X-Pusher-Signature"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    sender.send_channel_occupied(&app, "test-channel").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_multiple_webhooks_different_filters() {
    // Start two mock servers
    let mock_server1 = MockServer::start().await;
    let mock_server2 = MockServer::start().await;

    // Create two webhooks with different filters
    let webhook1 = Webhook {
        url: Some(mock_server1.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: None,
        }),
        lambda: None,
    };

    let webhook2 = Webhook {
        url: Some(mock_server2.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(WebhookFilter {
            channel_name_starts_with: Some("presence-".to_string()),
            channel_name_ends_with: None,
        }),
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook1, webhook2]);

    // Set up expectations - only webhook1 should receive
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server1)
        .await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&mock_server2)
        .await;

    let sender = WebhookSender::new();

    // Send for private channel - only webhook1 should match
    sender.send_channel_occupied(&app, "private-user-123").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_no_filter_receives_all() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook with NO filter
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None, // No filter
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock expectation - should receive all channel_occupied events
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(3)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send for different channel types
    sender.send_channel_occupied(&app, "public-channel").await;
    sender.send_channel_occupied(&app, "private-channel").await;
    sender.send_channel_occupied(&app, "presence-channel").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

// Helper function to check if header exists
fn header_exists(name: &str) -> impl wiremock::Match {
    let name = name.to_string();
    wiremock::matchers::header_exists(name)
}
