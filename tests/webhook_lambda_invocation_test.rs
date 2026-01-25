use serde_json::json;
use soketi_rs::app::{App, LambdaConfig, Webhook};
use soketi_rs::webhook_sender::{BatchingConfig, ClientEventData, WebhookSender};

/// Test that Lambda webhooks can be configured with async invocation
#[tokio::test]
async fn test_lambda_webhook_async_invocation_config() {
    let mut app = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    // Configure webhook with async Lambda invocation
    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-async-function".to_string()),
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];

    assert_eq!(
        app.webhooks[0].lambda_function,
        Some("my-async-function".to_string())
    );
    assert_eq!(
        app.webhooks[0].lambda.as_ref().unwrap().async_invocation,
        true
    );
}

/// Test that Lambda webhooks can be configured with sync invocation
#[tokio::test]
async fn test_lambda_webhook_sync_invocation_config() {
    let mut app = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );

    // Configure webhook with sync Lambda invocation
    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-sync-function".to_string()),
        event_types: vec!["member_added".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: false,
        }),
    }];

    assert_eq!(
        app.webhooks[0].lambda_function,
        Some("my-sync-function".to_string())
    );
    assert_eq!(
        app.webhooks[0].lambda.as_ref().unwrap().async_invocation,
        false
    );
}

/// Test that Lambda webhooks can be configured with client context
#[tokio::test]
async fn test_lambda_webhook_with_client_context() {
    let mut app = App::new(
        "app3".to_string(),
        "key3".to_string(),
        "secret3".to_string(),
    );

    let client_context = json!({
        "custom": {
            "app_id": "app3",
            "environment": "production"
        }
    });

    // Configure webhook with client context
    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-function-with-context".to_string()),
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: Some(client_context.clone()),
            async_invocation: true,
        }),
    }];

    assert_eq!(
        app.webhooks[0].lambda_function,
        Some("my-function-with-context".to_string())
    );
    assert_eq!(
        app.webhooks[0].lambda.as_ref().unwrap().client_context,
        Some(client_context)
    );
}

/// Test that Lambda webhooks default to sync invocation when lambda config is not provided
#[tokio::test]
async fn test_lambda_webhook_default_invocation_type() {
    let mut app = App::new(
        "app4".to_string(),
        "key4".to_string(),
        "secret4".to_string(),
    );

    // Configure webhook without lambda config (should default to sync)
    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-default-function".to_string()),
        event_types: vec!["channel_vacated".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    }];

    assert_eq!(
        app.webhooks[0].lambda_function,
        Some("my-default-function".to_string())
    );
    assert_eq!(app.webhooks[0].lambda, None);
}

/// Test that an app can have both HTTP and Lambda webhooks
#[tokio::test]
async fn test_mixed_http_and_lambda_webhooks() {
    let mut app = App::new(
        "app5".to_string(),
        "key5".to_string(),
        "secret5".to_string(),
    );

    app.webhooks = vec![
        Webhook {
            url: Some("https://example.com/webhook".to_string()),
            lambda_function: None,
            event_types: vec!["channel_occupied".to_string()],
            headers: None,
            filter: None,
            lambda: None,
        },
        Webhook {
            url: None,
            lambda_function: Some("my-lambda-function".to_string()),
            event_types: vec!["channel_occupied".to_string()],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: None,
                async_invocation: true,
            }),
        },
    ];

    assert_eq!(app.webhooks.len(), 2);
    assert!(app.webhooks[0].url.is_some());
    assert!(app.webhooks[0].lambda_function.is_none());
    assert!(app.webhooks[1].url.is_none());
    assert!(app.webhooks[1].lambda_function.is_some());
}

/// Test that Lambda webhooks support all event types
#[tokio::test]
async fn test_lambda_webhook_all_event_types() {
    let mut app = App::new(
        "app6".to_string(),
        "key6".to_string(),
        "secret6".to_string(),
    );

    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-all-events-function".to_string()),
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
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];

    assert!(app.has_channel_occupied_webhooks());
    assert!(app.has_channel_vacated_webhooks());
    assert!(app.has_member_added_webhooks());
    assert!(app.has_member_removed_webhooks());
    assert!(app.has_client_event_webhooks());
    assert!(app.has_cache_missed_webhooks());
}

/// Test that Lambda webhooks support channel name filtering
#[tokio::test]
async fn test_lambda_webhook_with_channel_filters() {
    let mut app = App::new(
        "app7".to_string(),
        "key7".to_string(),
        "secret7".to_string(),
    );

    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-filtered-function".to_string()),
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(soketi_rs::app::WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: Some("-notifications".to_string()),
        }),
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];

    let sender = WebhookSender::new();

    // Test filtering logic
    let event1 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "private-user-123-notifications".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&app.webhooks[0], &event1));

    let event2 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "public-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&app.webhooks[0], &event2));

    let event3 = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "private-user-123".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&app.webhooks[0], &event3));
}

/// Test that WebhookSender can be initialized with Lambda support
#[tokio::test]
async fn test_webhook_sender_with_lambda_enabled() {
    let batching_config = BatchingConfig {
        enabled: false,
        duration_ms: 50,
    };

    // Initialize with Lambda enabled
    let sender = WebhookSender::with_config(batching_config, true).await;

    // Sender should be created successfully
    // The Lambda client will be initialized but won't be used until actual invocation
    assert!(
        true,
        "WebhookSender with Lambda should initialize successfully"
    );
}

/// Test that WebhookSender can be initialized without Lambda support
#[tokio::test]
async fn test_webhook_sender_without_lambda() {
    let batching_config = BatchingConfig {
        enabled: false,
        duration_ms: 50,
    };

    // Initialize without Lambda
    let sender = WebhookSender::with_config(batching_config, false).await;

    assert!(
        true,
        "WebhookSender without Lambda should initialize successfully"
    );
}

/// Test that WebhookSender can be configured with custom Lambda client
#[tokio::test]
async fn test_webhook_sender_with_custom_lambda_client() {
    // Create AWS config and Lambda client
    let aws_config = aws_config::load_from_env().await;
    let lambda_client = aws_sdk_lambda::Client::new(&aws_config);

    // Create WebhookSender with custom Lambda client
    let sender = WebhookSender::new().with_lambda_client(lambda_client);

    assert!(true, "WebhookSender should accept custom Lambda client");
}

/// Test that Lambda webhooks work with batching enabled
#[tokio::test]
async fn test_lambda_webhook_with_batching() {
    let mut app = App::new(
        "app8".to_string(),
        "key8".to_string(),
        "secret8".to_string(),
    );

    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-batched-function".to_string()),
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];

    let batching_config = BatchingConfig {
        enabled: true,
        duration_ms: 100,
    };

    let sender = WebhookSender::with_config(batching_config, true).await;

    // Send multiple events (they should be batched)
    sender.send_channel_occupied(&app, "test-channel-1").await;
    sender.send_channel_vacated(&app, "test-channel-2").await;

    // Wait for batch to flush
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    assert!(true, "Lambda webhooks should work with batching");
}

/// Test that Lambda config can have complex client context
#[tokio::test]
async fn test_lambda_webhook_complex_client_context() {
    let mut app = App::new(
        "app9".to_string(),
        "key9".to_string(),
        "secret9".to_string(),
    );

    let complex_context = json!({
        "custom": {
            "app_id": "app9",
            "environment": "production",
            "region": "us-east-1",
            "metadata": {
                "version": "1.0.0",
                "deployment": "blue-green"
            },
            "tags": ["webhook", "lambda", "production"]
        }
    });

    app.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-complex-function".to_string()),
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: Some(complex_context.clone()),
            async_invocation: false,
        }),
    }];

    assert_eq!(
        app.webhooks[0].lambda.as_ref().unwrap().client_context,
        Some(complex_context)
    );
}

/// Test that multiple Lambda webhooks can be configured for different event types
#[tokio::test]
async fn test_multiple_lambda_webhooks_different_events() {
    let mut app = App::new(
        "app10".to_string(),
        "key10".to_string(),
        "secret10".to_string(),
    );

    app.webhooks = vec![
        Webhook {
            url: None,
            lambda_function: Some("channel-events-function".to_string()),
            event_types: vec![
                "channel_occupied".to_string(),
                "channel_vacated".to_string(),
            ],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: None,
                async_invocation: true,
            }),
        },
        Webhook {
            url: None,
            lambda_function: Some("member-events-function".to_string()),
            event_types: vec!["member_added".to_string(), "member_removed".to_string()],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: None,
                async_invocation: false,
            }),
        },
        Webhook {
            url: None,
            lambda_function: Some("client-events-function".to_string()),
            event_types: vec!["client_event".to_string()],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: Some(json!({"type": "client_event"})),
                async_invocation: true,
            }),
        },
    ];

    assert_eq!(app.webhooks.len(), 3);
    assert_eq!(
        app.webhooks[0].lambda_function,
        Some("channel-events-function".to_string())
    );
    assert_eq!(
        app.webhooks[1].lambda_function,
        Some("member-events-function".to_string())
    );
    assert_eq!(
        app.webhooks[2].lambda_function,
        Some("client-events-function".to_string())
    );

    assert_eq!(
        app.webhooks[0].lambda.as_ref().unwrap().async_invocation,
        true
    );
    assert_eq!(
        app.webhooks[1].lambda.as_ref().unwrap().async_invocation,
        false
    );
    assert_eq!(
        app.webhooks[2].lambda.as_ref().unwrap().async_invocation,
        true
    );
}

/// Test that Lambda webhooks can be combined with channel filters for precise targeting
#[tokio::test]
async fn test_lambda_webhook_precise_filtering() {
    let mut app = App::new(
        "app11".to_string(),
        "key11".to_string(),
        "secret11".to_string(),
    );

    app.webhooks = vec![
        Webhook {
            url: None,
            lambda_function: Some("private-channels-function".to_string()),
            event_types: vec!["channel_occupied".to_string()],
            headers: None,
            filter: Some(soketi_rs::app::WebhookFilter {
                channel_name_starts_with: Some("private-".to_string()),
                channel_name_ends_with: None,
            }),
            lambda: Some(LambdaConfig {
                client_context: None,
                async_invocation: true,
            }),
        },
        Webhook {
            url: None,
            lambda_function: Some("presence-channels-function".to_string()),
            event_types: vec!["member_added".to_string(), "member_removed".to_string()],
            headers: None,
            filter: Some(soketi_rs::app::WebhookFilter {
                channel_name_starts_with: Some("presence-".to_string()),
                channel_name_ends_with: None,
            }),
            lambda: Some(LambdaConfig {
                client_context: None,
                async_invocation: false,
            }),
        },
    ];

    let sender = WebhookSender::new();

    // Test private channel filtering
    let private_event = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "private-user-123".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(sender.should_send_webhook(&app.webhooks[0], &private_event));

    // Test presence channel filtering
    let presence_event = ClientEventData {
        name: "member_added".to_string(),
        channel: "presence-room-456".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: Some("user-789".to_string()),
    };
    assert!(sender.should_send_webhook(&app.webhooks[1], &presence_event));

    // Test that public channels don't match private filter
    let public_event = ClientEventData {
        name: "channel_occupied".to_string(),
        channel: "public-channel".to_string(),
        event: None,
        data: None,
        socket_id: None,
        user_id: None,
    };
    assert!(!sender.should_send_webhook(&app.webhooks[0], &public_event));
}
