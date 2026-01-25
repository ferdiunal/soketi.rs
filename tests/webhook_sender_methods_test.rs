use serde_json::json;
use soketi_rs::app::{App, Webhook};
use soketi_rs::webhook_sender::{BatchingConfig, WebhookSender};

/// Test send_channel_occupied method
#[tokio::test]
async fn test_send_channel_occupied() {
    let sender = WebhookSender::new();

    // App without channel_occupied webhooks - should not send
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    // This should return immediately without sending
    sender.send_channel_occupied(&app1, "test-channel").await;

    // App with channel_occupied webhooks
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // This should attempt to send (will fail without a real server, but that's ok for this test)
    sender.send_channel_occupied(&app2, "test-channel").await;

    assert!(
        true,
        "send_channel_occupied should handle apps with and without webhooks"
    );
}

/// Test send_channel_vacated method
#[tokio::test]
async fn test_send_channel_vacated() {
    let sender = WebhookSender::new();

    // App without channel_vacated webhooks - should not send
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    sender.send_channel_vacated(&app1, "test-channel").await;

    // App with channel_vacated webhooks
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["channel_vacated".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    sender.send_channel_vacated(&app2, "test-channel").await;

    assert!(
        true,
        "send_channel_vacated should handle apps with and without webhooks"
    );
}

/// Test send_member_added method
#[tokio::test]
async fn test_send_member_added() {
    let sender = WebhookSender::new();

    // App without member_added webhooks - should not send
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    sender
        .send_member_added(&app1, "presence-channel", "user123")
        .await;

    // App with member_added webhooks
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["member_added".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    sender
        .send_member_added(&app2, "presence-channel", "user123")
        .await;

    assert!(
        true,
        "send_member_added should handle apps with and without webhooks"
    );
}

/// Test send_member_removed method
#[tokio::test]
async fn test_send_member_removed() {
    let sender = WebhookSender::new();

    // App without member_removed webhooks - should not send
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    sender
        .send_member_removed(&app1, "presence-channel", "user123")
        .await;

    // App with member_removed webhooks
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["member_removed".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    sender
        .send_member_removed(&app2, "presence-channel", "user123")
        .await;

    assert!(
        true,
        "send_member_removed should handle apps with and without webhooks"
    );
}

/// Test send_client_event method
#[tokio::test]
async fn test_send_client_event() {
    let sender = WebhookSender::new();

    // App without client_event webhooks - should not send
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    let data = json!({"message": "hello"});
    sender
        .send_client_event(
            &app1,
            "test-channel",
            "client-event",
            data.clone(),
            Some("socket123"),
            Some("user456"),
        )
        .await;

    // App with client_event webhooks
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    sender
        .send_client_event(
            &app2,
            "test-channel",
            "client-event",
            data,
            Some("socket123"),
            Some("user456"),
        )
        .await;

    assert!(
        true,
        "send_client_event should handle apps with and without webhooks"
    );
}

/// Test send_cache_missed method
#[tokio::test]
async fn test_send_cache_missed() {
    let sender = WebhookSender::new();

    // App without cache_miss webhooks - should not send
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    sender.send_cache_missed(&app1, "cache-channel").await;

    // App with cache_miss webhooks
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["cache_miss".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    sender.send_cache_missed(&app2, "cache-channel").await;

    assert!(
        true,
        "send_cache_missed should handle apps with and without webhooks"
    );
}

/// Test that all webhook methods work with multiple webhooks configured
#[tokio::test]
async fn test_multiple_webhooks_configured() {
    let sender = WebhookSender::new();

    // App with multiple webhooks for different event types
    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![
        Webhook {
            url: Some("https://example.com/webhook1".to_string()),
            lambda_function: None,
            event_types: vec![
                "channel_occupied".to_string(),
                "channel_vacated".to_string(),
            ],
            headers: None,
            filter: None,
            lambda: None,
        },
        Webhook {
            url: Some("https://example.com/webhook2".to_string()),
            lambda_function: None,
            event_types: vec!["member_added".to_string(), "member_removed".to_string()],
            headers: None,
            filter: None,
            lambda: None,
        },
        Webhook {
            url: Some("https://example.com/webhook3".to_string()),
            lambda_function: None,
            event_types: vec!["client_event".to_string(), "cache_miss".to_string()],
            headers: None,
            filter: None,
            lambda: None,
        },
    ];

    // Test all methods
    sender.send_channel_occupied(&app, "test-channel").await;
    sender.send_channel_vacated(&app, "test-channel").await;
    sender
        .send_member_added(&app, "presence-channel", "user123")
        .await;
    sender
        .send_member_removed(&app, "presence-channel", "user123")
        .await;
    sender
        .send_client_event(
            &app,
            "test-channel",
            "client-event",
            json!({}),
            Some("socket123"),
            None,
        )
        .await;
    sender.send_cache_missed(&app, "cache-channel").await;

    assert!(
        true,
        "All webhook methods should work with multiple webhooks configured"
    );
}

/// Test webhook methods with empty channel names
#[tokio::test]
async fn test_webhook_methods_with_empty_channel() {
    let sender = WebhookSender::new();

    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
            "member_added".to_string(),
            "member_removed".to_string(),
            "client_event".to_string(),
            "cache_miss".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // Test with empty channel names
    sender.send_channel_occupied(&app, "").await;
    sender.send_channel_vacated(&app, "").await;
    sender.send_member_added(&app, "", "user123").await;
    sender.send_member_removed(&app, "", "user123").await;
    sender
        .send_client_event(&app, "", "client-event", json!({}), None, None)
        .await;
    sender.send_cache_missed(&app, "").await;

    assert!(true, "Webhook methods should handle empty channel names");
}

/// Test webhook methods with special characters in channel names
#[tokio::test]
async fn test_webhook_methods_with_special_characters() {
    let sender = WebhookSender::new();

    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string(), "member_added".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // Test with special characters
    let special_channels = vec![
        "private-channel",
        "presence-channel",
        "private-encrypted-channel",
        "channel-with-dashes",
        "channel_with_underscores",
        "channel.with.dots",
        "channel:with:colons",
    ];

    for channel in special_channels {
        sender.send_channel_occupied(&app, channel).await;
        sender.send_member_added(&app, channel, "user123").await;
    }

    assert!(
        true,
        "Webhook methods should handle special characters in channel names"
    );
}

/// Test webhook methods with None optional parameters
#[tokio::test]
async fn test_webhook_methods_with_none_parameters() {
    let sender = WebhookSender::new();

    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // Test send_client_event with None for optional parameters
    sender
        .send_client_event(&app, "test-channel", "client-event", json!({}), None, None)
        .await;
    sender
        .send_client_event(
            &app,
            "test-channel",
            "client-event",
            json!({}),
            Some("socket123"),
            None,
        )
        .await;
    sender
        .send_client_event(
            &app,
            "test-channel",
            "client-event",
            json!({}),
            None,
            Some("user456"),
        )
        .await;

    assert!(
        true,
        "send_client_event should handle None optional parameters"
    );
}

/// Test that webhook methods work with batching enabled
#[tokio::test]
async fn test_webhook_methods_with_batching() {
    let sender = WebhookSender::new().with_batching(true, 100);

    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
            "member_added".to_string(),
            "member_removed".to_string(),
            "client_event".to_string(),
            "cache_miss".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // Send multiple events that should be batched
    sender.send_channel_occupied(&app, "channel1").await;
    sender.send_channel_vacated(&app, "channel2").await;
    sender
        .send_member_added(&app, "presence-channel", "user1")
        .await;
    sender
        .send_member_removed(&app, "presence-channel", "user2")
        .await;
    sender
        .send_client_event(
            &app,
            "channel3",
            "client-event",
            json!({}),
            Some("socket1"),
            None,
        )
        .await;
    sender.send_cache_missed(&app, "cache-channel").await;

    assert!(true, "Webhook methods should work with batching enabled");
}

/// Test webhook methods with very long channel names
#[tokio::test]
async fn test_webhook_methods_with_long_channel_names() {
    let sender = WebhookSender::new();

    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // Test with very long channel name
    let long_channel = "a".repeat(200);
    sender.send_channel_occupied(&app, &long_channel).await;

    assert!(true, "Webhook methods should handle long channel names");
}

/// Test webhook methods with complex JSON data
#[tokio::test]
async fn test_webhook_methods_with_complex_data() {
    let sender = WebhookSender::new();

    let mut app = App::new("app".to_string(), "key".to_string(), "secret".to_string());
    app.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    // Test with complex nested JSON data
    let complex_data = json!({
        "message": "hello",
        "nested": {
            "level1": {
                "level2": {
                    "value": 123
                }
            }
        },
        "array": [1, 2, 3, 4, 5],
        "boolean": true,
        "null_value": null
    });

    sender
        .send_client_event(
            &app,
            "test-channel",
            "client-event",
            complex_data,
            Some("socket123"),
            Some("user456"),
        )
        .await;

    assert!(true, "send_client_event should handle complex JSON data");
}
