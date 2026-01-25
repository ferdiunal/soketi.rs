use serde_json::json;
use soketi_rs::app::{App, Webhook, WebhookFilter};
use soketi_rs::webhook_sender::{ClientEventData, WebhookSender};

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

#[test]
fn test_webhook_filter_prefix_match() {
    let filter = WebhookFilter {
        channel_name_starts_with: Some("private-".to_string()),
        channel_name_ends_with: None,
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Test matching prefix
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "private-user-123".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    // Test non-matching prefix
    let event2 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "public-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&webhook, &event2));
}

#[test]
fn test_webhook_filter_suffix_match() {
    let filter = WebhookFilter {
        channel_name_starts_with: None,
        channel_name_ends_with: Some("-notifications".to_string()),
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Test matching suffix
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "user-123-notifications".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    // Test non-matching suffix
    let event2 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "user-123-messages".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&webhook, &event2));
}

#[test]
fn test_webhook_filter_both_prefix_and_suffix() {
    let filter = WebhookFilter {
        channel_name_starts_with: Some("presence-".to_string()),
        channel_name_ends_with: Some("-room".to_string()),
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["member_added".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Test matching both
    let event1 = ClientEventData {
        name: "member_added".to_string(),
        channel: "presence-chat-room".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: Some("user-123".to_string()),
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    // Test matching prefix but not suffix
    let event2 = ClientEventData {
        name: "member_added".to_string(),
        channel: "presence-chat-lobby".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: Some("user-123".to_string()),
    };
    assert!(!sender.should_send_webhook(&webhook, &event2));

    // Test matching suffix but not prefix
    let event3 = ClientEventData {
        name: "member_added".to_string(),
        channel: "private-chat-room".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: Some("user-123".to_string()),
    };
    assert!(!sender.should_send_webhook(&webhook, &event3));

    // Test matching neither
    let event4 = ClientEventData {
        name: "member_added".to_string(),
        channel: "public-lobby".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: Some("user-123".to_string()),
    };
    assert!(!sender.should_send_webhook(&webhook, &event4));
}

#[test]
fn test_webhook_filter_no_filter() {
    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None, // No filter
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Should match any channel when no filter is present
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "any-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    let event2 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "private-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event2));
}

#[test]
fn test_webhook_filter_event_type_mismatch() {
    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Should not match different event type
    let event = ClientEventData {
        name: "channel_vacated".to_string(),
        channel: "test-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&webhook, &event));
}

#[test]
fn test_webhook_filter_multiple_event_types() {
    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
            "member_added".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Should match all configured event types
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "test-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    let event2 = ClientEventData {
        name: "channel_vacated".to_string(),
        channel: "test-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event2));

    let event3 = ClientEventData {
        name: "member_added".to_string(),
        channel: "test-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: Some("user-123".to_string()),
    };
    assert!(sender.should_send_webhook(&webhook, &event3));

    // Should not match unconfigured event type
    let event4 = ClientEventData {
        name: "client_event".to_string(),
        channel: "test-channel".to_string(),
        event: Some("custom-event".to_string()),
        data: Some(json!({"key": "value"})),
        socket_id: Some("socket-123".to_string()),
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&webhook, &event4));
}

#[test]
fn test_webhook_filter_empty_prefix() {
    let filter = WebhookFilter {
        channel_name_starts_with: Some("".to_string()),
        channel_name_ends_with: None,
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Empty prefix should match all channels (all strings start with "")
    let event = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "any-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event));
}

#[test]
fn test_webhook_filter_empty_suffix() {
    let filter = WebhookFilter {
        channel_name_starts_with: None,
        channel_name_ends_with: Some("".to_string()),
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Empty suffix should match all channels (all strings end with "")
    let event = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "any-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event));
}

#[test]
fn test_webhook_filter_exact_match() {
    let filter = WebhookFilter {
        channel_name_starts_with: Some("exact-channel".to_string()),
        channel_name_ends_with: Some("exact-channel".to_string()),
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Should match exact channel name
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "exact-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    // Should not match different channel
    let event2 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "exact-channel-2".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&webhook, &event2));
}

#[test]
fn test_webhook_filter_case_sensitive() {
    let filter = WebhookFilter {
        channel_name_starts_with: Some("Private-".to_string()),
        channel_name_ends_with: None,
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Should match with correct case
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "Private-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event1));

    // Should not match with different case
    let event2 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "private-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&webhook, &event2));
}

#[test]
fn test_webhook_filter_special_characters() {
    let filter = WebhookFilter {
        channel_name_starts_with: Some("user-".to_string()),
        channel_name_ends_with: Some("-notifications".to_string()),
    };

    let webhook = Webhook {
        url: Some("http://example.com".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(filter),
        lambda: None,
    };

    let sender = WebhookSender::new();

    // Should handle special characters in channel names
    let event = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "user-john.doe@example.com-notifications".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&webhook, &event));
}
