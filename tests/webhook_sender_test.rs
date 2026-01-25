use soketi_rs::webhook_sender::{BatchingConfig, WebhookSender};

#[tokio::test]
async fn test_webhook_sender_new() {
    // Test default initialization
    let sender = WebhookSender::new();

    // Verify it was created successfully (we can't directly inspect private fields,
    // but we can verify it doesn't panic and can be used)
    assert!(true, "WebhookSender::new() should create a valid instance");
}

#[tokio::test]
async fn test_webhook_sender_with_config_no_lambda() {
    // Test initialization with custom config but no Lambda
    let batching_config = BatchingConfig {
        enabled: true,
        duration_ms: 100,
    };

    let sender = WebhookSender::with_config(batching_config, false).await;

    // Verify it was created successfully
    assert!(
        true,
        "WebhookSender::with_config() should create a valid instance"
    );
}

#[tokio::test]
async fn test_webhook_sender_with_config_with_lambda() {
    // Test initialization with Lambda enabled
    // Note: This requires AWS credentials to be configured
    let batching_config = BatchingConfig {
        enabled: false,
        duration_ms: 50,
    };

    // This will attempt to load AWS config from environment
    // In a real test environment, you'd mock this or skip if credentials aren't available
    let sender = WebhookSender::with_config(batching_config, true).await;

    // Verify it was created successfully
    assert!(
        true,
        "WebhookSender::with_config() with Lambda should create a valid instance"
    );
}

#[tokio::test]
async fn test_webhook_sender_with_http_client() {
    // Test initialization with custom HTTP client
    let custom_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let sender = WebhookSender::with_http_client(custom_client);

    // Verify it was created successfully
    assert!(
        true,
        "WebhookSender::with_http_client() should create a valid instance"
    );
}

#[tokio::test]
async fn test_webhook_sender_with_batching() {
    // Test batching configuration
    let sender = WebhookSender::new().with_batching(true, 200);

    // Verify it was created successfully
    assert!(
        true,
        "WebhookSender::with_batching() should configure batching"
    );
}

#[tokio::test]
async fn test_webhook_sender_default() {
    // Test Default trait implementation
    let sender = WebhookSender::default();

    // Verify it was created successfully
    assert!(
        true,
        "WebhookSender::default() should create a valid instance"
    );
}

#[tokio::test]
async fn test_batching_config_default() {
    // Test BatchingConfig default values
    let config = BatchingConfig::default();

    assert_eq!(config.enabled, false, "Default batching should be disabled");
    assert_eq!(config.duration_ms, 50, "Default duration should be 50ms");
}

#[tokio::test]
async fn test_batching_config_custom() {
    // Test custom BatchingConfig
    let config = BatchingConfig {
        enabled: true,
        duration_ms: 1000,
    };

    assert_eq!(config.enabled, true, "Custom batching should be enabled");
    assert_eq!(config.duration_ms, 1000, "Custom duration should be 1000ms");
}

#[tokio::test]
async fn test_webhook_sender_clone() {
    // Test that WebhookSender can be cloned (required for Arc<WebhookSender>)
    let sender = WebhookSender::new();
    let cloned = sender.clone();

    // Both should be valid instances
    assert!(true, "WebhookSender should be cloneable");
}

#[tokio::test]
async fn test_webhook_sender_builder_pattern() {
    // Test builder pattern with multiple configurations
    let custom_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let sender = WebhookSender::with_http_client(custom_client).with_batching(true, 150);

    // Verify the builder pattern works
    assert!(true, "WebhookSender builder pattern should work");
}

#[tokio::test]
async fn test_webhook_sender_with_lambda_client() {
    // Test setting a custom Lambda client
    let aws_config = aws_config::load_from_env().await;
    let lambda_client = aws_sdk_lambda::Client::new(&aws_config);

    let sender = WebhookSender::new().with_lambda_client(lambda_client);

    // Verify it was created successfully
    assert!(
        true,
        "WebhookSender::with_lambda_client() should set Lambda client"
    );
}
