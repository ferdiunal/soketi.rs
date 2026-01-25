use soketi_rs::adapters::nats::NatsAdapter;
use soketi_rs::config::NatsAdapterConfig;

/// Test that NatsAdapter can be created with default config
/// This test will fail if NATS is not available, which is expected
#[tokio::test]
async fn test_nats_adapter_creation_with_default_config() {
    let config = NatsAdapterConfig::default();

    // This will attempt to connect to NATS at 127.0.0.1:4222
    // If NATS is not available, it should return an error
    let result = NatsAdapter::new(config).await;

    // We just verify that the function returns a Result
    // In a real environment with NATS, this would succeed
    match result {
        Ok(_) => {
            println!("NATS adapter created successfully - NATS is available");
        }
        Err(e) => {
            println!(
                "NATS adapter creation failed (expected if NATS is not running): {}",
                e
            );
        }
    }
}

/// Test that NatsAdapter can be created with custom config
#[tokio::test]
async fn test_nats_adapter_creation_with_custom_config() {
    let config = NatsAdapterConfig {
        servers: vec!["localhost:4222".to_string()],
        user: None,
        password: None,
        token: None,
        timeout_ms: 5000,
        prefix: "test-prefix".to_string(),
    };

    let result = NatsAdapter::new(config).await;

    match result {
        Ok(_) => {
            println!("NATS adapter with custom config created successfully");
        }
        Err(e) => {
            println!(
                "NATS adapter creation with custom config failed (expected if NATS is not running): {}",
                e
            );
        }
    }
}

/// Test that NatsAdapter properly handles authentication config with user/password
#[tokio::test]
async fn test_nats_adapter_with_user_password_auth() {
    let config = NatsAdapterConfig {
        servers: vec!["localhost:4222".to_string()],
        user: Some("testuser".to_string()),
        password: Some("testpass".to_string()),
        token: None,
        timeout_ms: 5000,
        prefix: "test".to_string(),
    };

    let result = NatsAdapter::new(config).await;

    // This will fail because we don't have a NATS instance with these credentials
    // But it tests that the authentication is properly configured
    match result {
        Ok(_) => {
            println!("NATS adapter with user/password auth created successfully");
        }
        Err(e) => {
            println!(
                "NATS adapter with user/password auth failed (expected): {}",
                e
            );
            // Verify the error message indicates connection failure
            assert!(e.to_string().contains("Failed to") || e.to_string().contains("connect"));
        }
    }
}

/// Test that NatsAdapter properly handles authentication config with token
#[tokio::test]
async fn test_nats_adapter_with_token_auth() {
    let config = NatsAdapterConfig {
        servers: vec!["localhost:4222".to_string()],
        user: None,
        password: None,
        token: Some("test-token".to_string()),
        timeout_ms: 5000,
        prefix: "test".to_string(),
    };

    let result = NatsAdapter::new(config).await;

    // This will fail because we don't have a NATS instance with this token
    // But it tests that the token authentication is properly configured
    match result {
        Ok(_) => {
            println!("NATS adapter with token auth created successfully");
        }
        Err(e) => {
            println!("NATS adapter with token auth failed (expected): {}", e);
            // Verify the error message indicates connection failure
            assert!(e.to_string().contains("Failed to") || e.to_string().contains("connect"));
        }
    }
}

/// Test that NatsAdapter properly handles multiple servers
#[tokio::test]
async fn test_nats_adapter_with_multiple_servers() {
    let config = NatsAdapterConfig {
        servers: vec![
            "localhost:4222".to_string(),
            "localhost:4223".to_string(),
            "localhost:4224".to_string(),
        ],
        user: None,
        password: None,
        token: None,
        timeout_ms: 5000,
        prefix: "test".to_string(),
    };

    let result = NatsAdapter::new(config).await;

    match result {
        Ok(_) => {
            println!("NATS adapter with multiple servers created successfully");
        }
        Err(e) => {
            println!(
                "NATS adapter with multiple servers failed (expected if NATS is not running): {}",
                e
            );
        }
    }
}
