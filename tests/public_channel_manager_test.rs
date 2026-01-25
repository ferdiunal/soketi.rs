use axum::extract::ws::Message;
use soketi_rs::adapters::Adapter;
use soketi_rs::adapters::local::LocalAdapter;
use soketi_rs::app::App;
use soketi_rs::channels::public::PublicChannelManager;
use soketi_rs::namespace::Socket;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Helper function to create a test app
fn create_test_app() -> App {
    App::new(
        "test-app-id".to_string(),
        "test-app-key".to_string(),
        "test-app-secret".to_string(),
    )
}

/// Helper function to create a test socket
fn create_test_socket(id: &str) -> Socket {
    let (tx, _rx) = mpsc::channel::<Message>(100);
    Socket {
        id: id.to_string(),
        sender: tx,
    }
}

#[tokio::test]
async fn test_public_channel_join_success() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Join a public channel
    let response = manager.join(&app, &socket, "test-channel", None).await;

    // Verify success
    assert!(response.success);
    assert_eq!(response.error_code, None);
    assert_eq!(response.error_message, None);
    assert_eq!(response.auth_error, false);

    // Verify socket is in channel
    let is_in = adapter
        .is_in_channel(&app.id, "test-channel", &socket.id)
        .await
        .unwrap();
    assert!(is_in);

    // Verify channel count
    let count = adapter
        .get_channel_sockets_count(&app.id, "test-channel")
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_public_channel_join_multiple_sockets() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and sockets
    let app = create_test_app();
    let socket1 = create_test_socket("socket-1");
    let socket2 = create_test_socket("socket-2");

    // Add sockets to adapter
    adapter.add_socket(&app.id, socket1.clone()).await.unwrap();
    adapter.add_socket(&app.id, socket2.clone()).await.unwrap();

    // Join the same channel with both sockets
    let response1 = manager.join(&app, &socket1, "test-channel", None).await;
    let response2 = manager.join(&app, &socket2, "test-channel", None).await;

    // Verify both succeeded
    assert!(response1.success);
    assert!(response2.success);

    // Verify both sockets are in channel
    let is_in1 = adapter
        .is_in_channel(&app.id, "test-channel", &socket1.id)
        .await
        .unwrap();
    let is_in2 = adapter
        .is_in_channel(&app.id, "test-channel", &socket2.id)
        .await
        .unwrap();
    assert!(is_in1);
    assert!(is_in2);

    // Verify channel count
    let count = adapter
        .get_channel_sockets_count(&app.id, "test-channel")
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_public_channel_reject_private_channel() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Try to join a private channel (should fail)
    let response = manager.join(&app, &socket, "private-test", None).await;

    // Verify failure
    assert!(!response.success);
    assert_eq!(response.error_code, Some(4009));
    assert!(response.error_message.is_some());
    assert!(
        response
            .error_message
            .unwrap()
            .contains("requires authentication")
    );
    assert_eq!(response.auth_error, true);
    assert_eq!(response.type_, Some("AuthError".to_string()));

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, "private-test", &socket.id)
        .await
        .unwrap();
    assert!(!is_in);
}

#[tokio::test]
async fn test_public_channel_reject_presence_channel() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Try to join a presence channel (should fail)
    let response = manager.join(&app, &socket, "presence-test", None).await;

    // Verify failure
    assert!(!response.success);
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);
}

#[tokio::test]
async fn test_public_channel_reject_encrypted_private_channel() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Try to join an encrypted private channel (should fail)
    let response = manager
        .join(&app, &socket, "private-encrypted-test", None)
        .await;

    // Verify failure
    assert!(!response.success);
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);
}

#[tokio::test]
async fn test_public_channel_leave() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Join a channel
    let response = manager.join(&app, &socket, "test-channel", None).await;
    assert!(response.success);

    // Verify socket is in channel
    let is_in = adapter
        .is_in_channel(&app.id, "test-channel", &socket.id)
        .await
        .unwrap();
    assert!(is_in);

    // Leave the channel
    let result = manager.leave(&app.id, &socket.id, "test-channel").await;
    assert!(result.is_ok());

    // Verify socket is no longer in channel
    let is_in = adapter
        .is_in_channel(&app.id, "test-channel", &socket.id)
        .await
        .unwrap();
    assert!(!is_in);

    // Verify channel count is 0
    let count = adapter
        .get_channel_sockets_count(&app.id, "test-channel")
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_public_channel_leave_multiple_sockets() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and sockets
    let app = create_test_app();
    let socket1 = create_test_socket("socket-1");
    let socket2 = create_test_socket("socket-2");

    // Add sockets to adapter
    adapter.add_socket(&app.id, socket1.clone()).await.unwrap();
    adapter.add_socket(&app.id, socket2.clone()).await.unwrap();

    // Join the same channel with both sockets
    manager.join(&app, &socket1, "test-channel", None).await;
    manager.join(&app, &socket2, "test-channel", None).await;

    // Verify both are in channel
    let count = adapter
        .get_channel_sockets_count(&app.id, "test-channel")
        .await
        .unwrap();
    assert_eq!(count, 2);

    // Leave with socket1
    let result = manager.leave(&app.id, &socket1.id, "test-channel").await;
    assert!(result.is_ok());

    // Verify socket1 is no longer in channel but socket2 still is
    let is_in1 = adapter
        .is_in_channel(&app.id, "test-channel", &socket1.id)
        .await
        .unwrap();
    let is_in2 = adapter
        .is_in_channel(&app.id, "test-channel", &socket2.id)
        .await
        .unwrap();
    assert!(!is_in1);
    assert!(is_in2);

    // Verify channel count is 1
    let count = adapter
        .get_channel_sockets_count(&app.id, "test-channel")
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_public_channel_leave_nonexistent_socket() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Try to leave a channel with a socket that was never added
    let result = manager
        .leave("test-app-id", "nonexistent-socket", "test-channel")
        .await;

    // Should succeed (idempotent operation)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_public_channel_join_different_channels() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PublicChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Join multiple different channels
    let response1 = manager.join(&app, &socket, "channel-1", None).await;
    let response2 = manager.join(&app, &socket, "channel-2", None).await;
    let response3 = manager.join(&app, &socket, "channel-3", None).await;

    // Verify all succeeded
    assert!(response1.success);
    assert!(response2.success);
    assert!(response3.success);

    // Verify socket is in all channels
    let is_in1 = adapter
        .is_in_channel(&app.id, "channel-1", &socket.id)
        .await
        .unwrap();
    let is_in2 = adapter
        .is_in_channel(&app.id, "channel-2", &socket.id)
        .await
        .unwrap();
    let is_in3 = adapter
        .is_in_channel(&app.id, "channel-3", &socket.id)
        .await
        .unwrap();
    assert!(is_in1);
    assert!(is_in2);
    assert!(is_in3);

    // Verify channel counts
    let channels = adapter
        .get_channels_with_sockets_count(&app.id)
        .await
        .unwrap();
    assert_eq!(channels.len(), 3);
    assert_eq!(channels.get("channel-1"), Some(&1));
    assert_eq!(channels.get("channel-2"), Some(&1));
    assert_eq!(channels.get("channel-3"), Some(&1));
}
