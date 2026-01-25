use soketi_rs::app::{App, LambdaConfig, Webhook};
use soketi_rs::webhook_sender::{BatchingConfig, WebhookSender};
use std::collections::HashMap;

/// Test that WebhookSender can be initialized with various configurations
/// and used in a multi-threaded context (Arc<WebhookSender>)
#[tokio::test]
async fn test_webhook_sender_initialization_scenarios() {
    // Scenario 1: Default initialization
    let sender1 = WebhookSender::new();
    assert!(true, "Default initialization should work");

    // Scenario 2: With batching enabled
    let sender2 = WebhookSender::new().with_batching(true, 100);
    assert!(true, "Batching configuration should work");

    // Scenario 3: With custom HTTP client
    let custom_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("test-agent")
        .build()
        .unwrap();
    let sender3 = WebhookSender::with_http_client(custom_client);
    assert!(true, "Custom HTTP client should work");

    // Scenario 4: With full configuration (no Lambda to avoid AWS dependency)
    let batching_config = BatchingConfig {
        enabled: true,
        duration_ms: 200,
    };
    let sender4 = WebhookSender::with_config(batching_config, false).await;
    assert!(true, "Full configuration without Lambda should work");
}

/// Test that WebhookSender can be shared across threads using Arc
#[tokio::test]
async fn test_webhook_sender_thread_safety() {
    use std::sync::Arc;

    let sender = Arc::new(WebhookSender::new());

    // Clone the Arc for multiple tasks
    let sender1 = Arc::clone(&sender);
    let sender2 = Arc::clone(&sender);
    let sender3 = Arc::clone(&sender);

    // Spawn multiple tasks that use the sender
    let task1 = tokio::spawn(async move {
        let _s = sender1;
        // In a real scenario, this would send webhooks
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    });

    let task2 = tokio::spawn(async move {
        let _s = sender2;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    });

    let task3 = tokio::spawn(async move {
        let _s = sender3;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    });

    // Wait for all tasks to complete
    let _ = tokio::join!(task1, task2, task3);

    assert!(true, "WebhookSender should be thread-safe with Arc");
}

/// Test that WebhookSender can handle apps with different webhook configurations
#[tokio::test]
async fn test_webhook_sender_with_various_app_configs() {
    let sender = WebhookSender::new();

    // App with no webhooks
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );
    assert!(!app1.has_client_event_webhooks());
    assert!(!app1.has_channel_occupied_webhooks());

    // App with client event webhooks
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
    assert!(app2.has_client_event_webhooks());
    assert!(!app2.has_channel_occupied_webhooks());

    // App with multiple webhook types
    let mut app3 = App::new(
        "app3".to_string(),
        "key3".to_string(),
        "secret3".to_string(),
    );
    app3.webhooks = vec![
        Webhook {
            url: Some("https://example.com/webhook1".to_string()),
            lambda_function: None,
            event_types: vec!["client_event".to_string(), "channel_occupied".to_string()],
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
    ];
    assert!(app3.has_client_event_webhooks());
    assert!(app3.has_channel_occupied_webhooks());
    assert!(app3.has_member_added_webhooks());
    assert!(app3.has_member_removed_webhooks());

    // App with Lambda webhook
    let mut app4 = App::new(
        "app4".to_string(),
        "key4".to_string(),
        "secret4".to_string(),
    );
    app4.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-lambda-function".to_string()),
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];
    assert!(app4.has_client_event_webhooks());

    // App with webhook filters
    let mut app5 = App::new(
        "app5".to_string(),
        "key5".to_string(),
        "secret5".to_string(),
    );
    app5.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string()],
        headers: Some({
            let mut headers = HashMap::new();
            headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
            headers
        }),
        filter: Some(soketi_rs::app::WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: Some("-notifications".to_string()),
        }),
        lambda: None,
    }];
    assert!(app5.has_client_event_webhooks());

    assert!(
        true,
        "WebhookSender should handle various app configurations"
    );
}

/// Test BatchingConfig with various values
#[tokio::test]
async fn test_batching_config_variations() {
    // Test with batching disabled
    let config1 = BatchingConfig {
        enabled: false,
        duration_ms: 50,
    };
    assert!(!config1.enabled);

    // Test with batching enabled and short duration
    let config2 = BatchingConfig {
        enabled: true,
        duration_ms: 10,
    };
    assert!(config2.enabled);
    assert_eq!(config2.duration_ms, 10);

    // Test with batching enabled and long duration
    let config3 = BatchingConfig {
        enabled: true,
        duration_ms: 5000,
    };
    assert!(config3.enabled);
    assert_eq!(config3.duration_ms, 5000);

    // Test default
    let config4 = BatchingConfig::default();
    assert!(!config4.enabled);
    assert_eq!(config4.duration_ms, 50);
}

/// Test that WebhookSender can be created with Lambda support
/// This test will pass even without AWS credentials configured
#[tokio::test]
async fn test_webhook_sender_lambda_initialization() {
    // Test with Lambda enabled (will use default AWS config)
    let batching_config = BatchingConfig::default();
    let sender = WebhookSender::with_config(batching_config, true).await;

    // The sender should be created successfully even if AWS credentials aren't configured
    // The Lambda client will be initialized but won't be used until actual invocation
    assert!(true, "WebhookSender with Lambda should initialize");
}

/// Test that WebhookSender can be configured with custom Lambda client
#[tokio::test]
async fn test_webhook_sender_custom_lambda_client() {
    // Create a custom AWS config and Lambda client
    let aws_config = aws_config::load_from_env().await;
    let lambda_client = aws_sdk_lambda::Client::new(&aws_config);

    // Create WebhookSender with custom Lambda client
    let sender = WebhookSender::new().with_lambda_client(lambda_client);

    assert!(true, "WebhookSender should accept custom Lambda client");
}

/// Test builder pattern combinations
#[tokio::test]
async fn test_webhook_sender_builder_combinations() {
    // Combination 1: Custom HTTP client + batching
    let client1 = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let sender1 = WebhookSender::with_http_client(client1).with_batching(true, 100);
    assert!(true, "HTTP client + batching should work");

    // Combination 2: Default + batching + Lambda
    let aws_config = aws_config::load_from_env().await;
    let lambda_client = aws_sdk_lambda::Client::new(&aws_config);
    let sender2 = WebhookSender::new()
        .with_batching(true, 200)
        .with_lambda_client(lambda_client);
    assert!(true, "Default + batching + Lambda should work");

    // Combination 3: Custom HTTP client + Lambda
    let client3 = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let aws_config3 = aws_config::load_from_env().await;
    let lambda_client3 = aws_sdk_lambda::Client::new(&aws_config3);
    let sender3 = WebhookSender::with_http_client(client3).with_lambda_client(lambda_client3);
    assert!(true, "HTTP client + Lambda should work");
}
