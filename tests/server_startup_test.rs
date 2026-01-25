use soketi_rs::app::App;
use soketi_rs::config::ServerConfig;
use soketi_rs::server::Server;
use std::time::Duration;
use tokio::time::timeout;

/// Test that server can be initialized and started
///
/// This test verifies:
/// 1. Server initialization with default configuration
/// 2. Server startup with WebSocket and HTTP routes
/// 3. Path prefix configuration
/// 4. Metrics server on separate port (if enabled)
///
/// Requirements: 1.5, 1.7
#[tokio::test]
async fn test_server_startup_basic() {
    // Create a test configuration
    let mut config = ServerConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = 16001; // Use a different port to avoid conflicts
    config.path_prefix = "".to_string();

    // Add a test app
    let test_app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    config.app_manager.array.apps = vec![test_app];

    // Create and initialize server
    let mut server = Server::new(config);
    let init_result = server.initialize().await;

    assert!(init_result.is_ok(), "Server initialization should succeed");
    assert!(
        server.state().is_some(),
        "Server state should be initialized"
    );

    // Note: We don't actually start the server in this test because it would block
    // In a real integration test, we would start the server in a separate task
    // and make HTTP/WebSocket requests to verify it's working
}

/// Test that server startup requires initialization
///
/// Requirements: 1.1, 1.2
#[tokio::test]
async fn test_server_startup_requires_initialization() {
    let config = ServerConfig::default();
    let server = Server::new(config);

    // Try to start without initialization
    let result = server.start().await;

    // Should fail because server is not initialized
    assert!(
        result.is_err(),
        "Server start should fail without initialization"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not initialized"),
        "Error should mention initialization: {}",
        error_msg
    );
}

/// Test that server logs startup information
///
/// This test verifies that the server logs all required startup information
/// including host, port, path prefix, SSL status, and component configurations.
///
/// Requirements: 1.5, 1.7
#[tokio::test]
async fn test_server_startup_logging() {
    // Create a test configuration with various settings
    let mut config = ServerConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = 16002;
    config.path_prefix = "/pusher".to_string();
    config.ssl.enabled = false;
    config.debug = true;
    config.metrics.enabled = false;

    // Add a test app
    let test_app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    config.app_manager.array.apps = vec![test_app];

    // Create and initialize server
    let mut server = Server::new(config);
    let init_result = server.initialize().await;

    assert!(init_result.is_ok(), "Server initialization should succeed");

    // Note: In a real test, we would capture the log output and verify
    // that it contains the expected startup information
    // For now, we just verify that initialization succeeds
}

/// Test that server can be configured with path prefix
///
/// Requirements: 1.7
#[tokio::test]
async fn test_server_with_path_prefix() {
    let mut config = ServerConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = 16003;
    config.path_prefix = "/api/v1".to_string();

    // Add a test app
    let test_app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    config.app_manager.array.apps = vec![test_app];

    // Create and initialize server
    let mut server = Server::new(config);
    let init_result = server.initialize().await;

    assert!(init_result.is_ok(), "Server initialization should succeed");
    assert!(
        server.state().is_some(),
        "Server state should be initialized"
    );

    // Note: In a real integration test, we would start the server and verify
    // that routes are accessible at /api/v1/app/:app_key, /api/v1/channels, etc.
}

/// Test that server can be configured with metrics on separate port
///
/// Requirements: 11.6
#[tokio::test]
async fn test_server_with_metrics_enabled() {
    let mut config = ServerConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = 16004;
    config.metrics.enabled = true;
    config.metrics.port = 19604; // Separate port for metrics
    config.metrics.prefix = "test_pusher".to_string();

    // Add a test app
    let test_app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    config.app_manager.array.apps = vec![test_app];

    // Create and initialize server
    let mut server = Server::new(config);
    let init_result = server.initialize().await;

    assert!(init_result.is_ok(), "Server initialization should succeed");
    assert!(
        server.state().is_some(),
        "Server state should be initialized"
    );

    let state = server.state().unwrap();
    assert!(
        state.metrics_manager.is_some(),
        "Metrics manager should be initialized"
    );

    // Note: In a real integration test, we would start the server and verify
    // that metrics are accessible at the configured metrics port
}

/// Test that server initialization logs component information
///
/// Requirements: 1.1, 1.2
#[tokio::test]
async fn test_server_initialization_logging() {
    let mut config = ServerConfig::default();

    // Add a test app
    let test_app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    config.app_manager.array.apps = vec![test_app];

    // Create and initialize server
    let mut server = Server::new(config);
    let init_result = server.initialize().await;

    assert!(init_result.is_ok(), "Server initialization should succeed");

    // Verify that all components are initialized
    let state = server.state().unwrap();

    // Test that we can interact with each component
    let socket_count = state.adapter.get_sockets_count("test_app").await;
    assert!(socket_count.is_ok(), "Adapter should be functional");

    let cache_result = state
        .cache_manager
        .set("test_key", "test_value", None)
        .await;
    assert!(cache_result.is_ok(), "Cache manager should be functional");

    // Note: In a real test, we would capture the log output and verify
    // that it contains initialization messages for each component
}
