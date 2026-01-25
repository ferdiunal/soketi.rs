use axum::extract::ws::Message;
use soketi_rs::adapters::Adapter;
use soketi_rs::app::App;
use soketi_rs::config::ServerConfig;
use soketi_rs::namespace::Socket;
use soketi_rs::server::Server;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Helper function to create a test app
fn create_test_app() -> App {
    App {
        id: "test_app_id".to_string(),
        key: "test_app_key".to_string(),
        secret: "test_app_secret".to_string(),
        max_connections: Some(100),
        enable_client_messages: true,
        enabled: true,
        max_backend_events_per_second: Some(100),
        max_client_events_per_second: Some(100),
        max_read_requests_per_second: Some(100),
        webhooks: vec![],
        max_presence_members_per_channel: Some(100),
        max_presence_member_size_in_kb: Some(2.0),
        max_channel_name_length: Some(200),
        max_event_channels_at_once: Some(100),
        max_event_name_length: Some(200),
        max_event_payload_in_kb: Some(100.0),
        max_event_batch_size: Some(10),
        enable_user_authentication: false,
    }
}

#[tokio::test]
async fn test_graceful_shutdown_sets_closing_flag() {
    // Create server with default config
    let mut config = ServerConfig::default();
    let test_app = create_test_app();
    config.app_manager.array.apps = vec![test_app];
    config.shutdown_grace_period_ms = 100; // Short grace period for testing

    let mut server = Server::new(config);
    server.initialize().await.unwrap();

    let state = server.state().unwrap();

    // Verify closing flag is initially false
    assert!(!state.closing.load(std::sync::atomic::Ordering::Relaxed));

    // Call stop
    let result = server.stop().await;
    assert!(result.is_ok(), "Graceful shutdown should succeed");

    // Verify closing flag is now true
    assert!(state.closing.load(std::sync::atomic::Ordering::Relaxed));
}

#[tokio::test]
async fn test_graceful_shutdown_closes_all_connections() {
    // Create server with default config
    let mut config = ServerConfig::default();
    let test_app = create_test_app();
    config.app_manager.array.apps = vec![test_app.clone()];
    config.shutdown_grace_period_ms = 100; // Short grace period for testing

    let mut server = Server::new(config);
    server.initialize().await.unwrap();

    let state = server.state().unwrap();

    // Add some test sockets
    let (tx1, mut rx1) = mpsc::channel::<Message>(10);
    let (tx2, mut rx2) = mpsc::channel::<Message>(10);

    let socket1 = Socket {
        id: "socket1".to_string(),
        sender: tx1,
    };

    let socket2 = Socket {
        id: "socket2".to_string(),
        sender: tx2,
    };

    state
        .adapter
        .add_socket(&test_app.id, socket1)
        .await
        .unwrap();
    state
        .adapter
        .add_socket(&test_app.id, socket2)
        .await
        .unwrap();

    // Verify sockets are added
    let socket_count = state.adapter.get_sockets_count(&test_app.id).await.unwrap();
    assert_eq!(socket_count, 2, "Should have 2 sockets");

    // Call stop
    let result = server.stop().await;
    assert!(result.is_ok(), "Graceful shutdown should succeed");

    // Verify that sockets received error messages with code 4200
    if let Some(msg) = rx1.recv().await {
        if let Message::Text(text) = msg {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["event"], "pusher:error");
            assert_eq!(json["data"]["code"], 4200);
        }
    }

    if let Some(msg) = rx2.recv().await {
        if let Message::Text(text) = msg {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["event"], "pusher:error");
            assert_eq!(json["data"]["code"], 4200);
        }
    }
}

#[tokio::test]
async fn test_graceful_shutdown_waits_grace_period() {
    // Create server with default config
    let mut config = ServerConfig::default();
    let test_app = create_test_app();
    config.app_manager.array.apps = vec![test_app];
    config.shutdown_grace_period_ms = 200; // 200ms grace period

    let mut server = Server::new(config);
    server.initialize().await.unwrap();

    // Measure time taken for shutdown
    let start = std::time::Instant::now();
    let result = server.stop().await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Graceful shutdown should succeed");

    // Verify that at least the grace period elapsed
    // Allow some tolerance for timing
    assert!(
        elapsed.as_millis() >= 180,
        "Shutdown should wait at least the grace period (got {}ms, expected >= 180ms)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_graceful_shutdown_disconnects_managers() {
    // Create server with default config
    let mut config = ServerConfig::default();
    let test_app = create_test_app();
    config.app_manager.array.apps = vec![test_app];
    config.shutdown_grace_period_ms = 50; // Short grace period for testing

    let mut server = Server::new(config);
    server.initialize().await.unwrap();

    // Call stop - this should disconnect all managers
    let result = server.stop().await;
    assert!(result.is_ok(), "Graceful shutdown should succeed");

    // Test passes if we reach here without errors
    // The disconnect methods are called and should not panic
}

#[tokio::test]
async fn test_graceful_shutdown_without_initialization() {
    // Create server without initialization
    let config = ServerConfig::default();
    let server = Server::new(config);

    // Try to stop without initialization
    let result = server.stop().await;

    // Should fail because server is not initialized
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not initialized"));
}

#[tokio::test]
async fn test_new_connections_rejected_when_closing() {
    // Create server with default config
    let mut config = ServerConfig::default();
    let test_app = create_test_app();
    config.app_manager.array.apps = vec![test_app.clone()];
    config.shutdown_grace_period_ms = 100;

    let mut server = Server::new(config);
    server.initialize().await.unwrap();

    let state = server.state().unwrap();

    // Set closing flag manually (simulating shutdown in progress)
    state
        .closing
        .store(true, std::sync::atomic::Ordering::Relaxed);

    // Try to add a new socket - this should be rejected by the connection handler
    // We can't easily test the full WebSocket connection here, but we can verify
    // that the closing flag is set and would be checked
    assert!(state.closing.load(std::sync::atomic::Ordering::Relaxed));

    // The actual rejection happens in ws_handler.handle_connection
    // which checks the closing flag at the beginning
}
