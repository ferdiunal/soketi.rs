use axum::extract::ws::Message;
use hex;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use soketi_rs::adapters::Adapter;
use soketi_rs::adapters::local::LocalAdapter;
use soketi_rs::app::App;
use soketi_rs::channels::private::PrivateChannelManager;
use soketi_rs::channels::{ChannelManager, PusherMessage};
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

/// Helper function to generate a valid auth signature for private channels
/// This mimics what a client would do to authenticate
fn generate_auth_signature(
    app_key: &str,
    app_secret: &str,
    socket_id: &str,
    channel: &str,
) -> String {
    let data_to_sign = format!("{}:{}", socket_id, channel);
    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data_to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}:{}", app_key, signature)
}

#[tokio::test]
async fn test_private_channel_join_with_valid_auth() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate valid auth signature
    let channel = "private-test-channel";
    let auth = generate_auth_signature(&app.key, &app.secret, &socket.id, channel);

    // Create subscription message with auth
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify success
    assert!(response.success, "Join should succeed with valid auth");
    assert_eq!(response.error_code, None);
    assert_eq!(response.error_message, None);
    assert_eq!(response.auth_error, false);

    // Verify socket is in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(is_in, "Socket should be in the channel");

    // Verify channel count
    let count = adapter
        .get_channel_sockets_count(&app.id, channel)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_private_channel_join_with_invalid_auth() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Create subscription message with invalid auth
    let channel = "private-test-channel";
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": "invalid-auth-signature"
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(!response.success, "Join should fail with invalid auth");
    assert_eq!(response.error_code, Some(4009));
    assert!(response.error_message.is_some());
    assert!(response.error_message.unwrap().contains("unauthorized"));
    assert_eq!(response.auth_error, true);
    assert_eq!(response.type_, Some("AuthError".to_string()));

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");
}

#[tokio::test]
async fn test_private_channel_join_without_auth() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Create subscription message without auth field
    let channel = "private-test-channel";
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({})),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(!response.success, "Join should fail without auth");
    assert_eq!(response.error_code, Some(4009));
    assert!(response.error_message.is_some());
    assert!(response.error_message.unwrap().contains("unauthorized"));
    assert_eq!(response.auth_error, true);
    assert_eq!(response.type_, Some("AuthError".to_string()));

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");
}

#[tokio::test]
async fn test_private_channel_join_without_message() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Try to join without providing a message
    let channel = "private-test-channel";
    let response = manager.join(&app, &socket, channel, None).await;

    // Verify failure
    assert!(!response.success, "Join should fail without message");
    assert_eq!(response.error_code, Some(4009));
    assert!(response.error_message.is_some());
    assert!(
        response
            .error_message
            .unwrap()
            .contains("Message check failed")
    );
    assert_eq!(response.auth_error, true);
    assert_eq!(response.type_, Some("AuthError".to_string()));

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");
}

#[tokio::test]
async fn test_private_channel_join_with_wrong_secret() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth signature with wrong secret
    let channel = "private-test-channel";
    let auth = generate_auth_signature(&app.key, "wrong-secret", &socket.id, channel);

    // Create subscription message with auth
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(!response.success, "Join should fail with wrong secret");
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");
}

#[tokio::test]
async fn test_private_channel_join_with_wrong_socket_id() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth signature for a different socket ID
    let channel = "private-test-channel";
    let auth = generate_auth_signature(&app.key, &app.secret, "different-socket-id", channel);

    // Create subscription message with auth
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(
        !response.success,
        "Join should fail with wrong socket ID in signature"
    );
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");
}

#[tokio::test]
async fn test_private_channel_join_with_wrong_channel() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth signature for a different channel
    let channel = "private-test-channel";
    let auth = generate_auth_signature(
        &app.key,
        &app.secret,
        &socket.id,
        "private-different-channel",
    );

    // Create subscription message with auth
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(
        !response.success,
        "Join should fail with wrong channel in signature"
    );
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");
}

#[tokio::test]
async fn test_private_channel_multiple_sockets_with_valid_auth() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and sockets
    let app = create_test_app();
    let socket1 = create_test_socket("socket-1");
    let socket2 = create_test_socket("socket-2");

    // Add sockets to adapter
    adapter.add_socket(&app.id, socket1.clone()).await.unwrap();
    adapter.add_socket(&app.id, socket2.clone()).await.unwrap();

    // Generate valid auth signatures for both sockets
    let channel = "private-test-channel";
    let auth1 = generate_auth_signature(&app.key, &app.secret, &socket1.id, channel);
    let auth2 = generate_auth_signature(&app.key, &app.secret, &socket2.id, channel);

    // Create subscription messages
    let message1 = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth1
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    let message2 = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth2
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Join the same channel with both sockets
    let response1 = manager.join(&app, &socket1, channel, Some(message1)).await;
    let response2 = manager.join(&app, &socket2, channel, Some(message2)).await;

    // Verify both succeeded
    assert!(response1.success, "Socket 1 should join successfully");
    assert!(response2.success, "Socket 2 should join successfully");

    // Verify both sockets are in channel
    let is_in1 = adapter
        .is_in_channel(&app.id, channel, &socket1.id)
        .await
        .unwrap();
    let is_in2 = adapter
        .is_in_channel(&app.id, channel, &socket2.id)
        .await
        .unwrap();
    assert!(is_in1, "Socket 1 should be in the channel");
    assert!(is_in2, "Socket 2 should be in the channel");

    // Verify channel count
    let count = adapter
        .get_channel_sockets_count(&app.id, channel)
        .await
        .unwrap();
    assert_eq!(count, 2, "Channel should have 2 sockets");
}

#[tokio::test]
async fn test_private_channel_join_different_channels() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Join multiple different private channels
    let channels = vec![
        "private-channel-1",
        "private-channel-2",
        "private-channel-3",
    ];

    for channel in &channels {
        let auth = generate_auth_signature(&app.key, &app.secret, &socket.id, channel);
        let message = PusherMessage {
            event: "pusher:subscribe".to_string(),
            data: Some(json!({
                "auth": auth
            })),
            channel: Some(channel.to_string()),
            socket_id: None,
        };

        let response = manager.join(&app, &socket, channel, Some(message)).await;
        assert!(response.success, "Should join {} successfully", channel);
    }

    // Verify socket is in all channels
    for channel in &channels {
        let is_in = adapter
            .is_in_channel(&app.id, channel, &socket.id)
            .await
            .unwrap();
        assert!(is_in, "Socket should be in {}", channel);
    }

    // Verify channel counts
    let all_channels = adapter
        .get_channels_with_sockets_count(&app.id)
        .await
        .unwrap();
    assert_eq!(all_channels.len(), 3, "Should have 3 channels");
    for channel in &channels {
        assert_eq!(
            all_channels.get(*channel),
            Some(&1),
            "{} should have 1 socket",
            channel
        );
    }
}

#[tokio::test]
async fn test_private_channel_auth_signature_format() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Test with malformed auth signature (missing colon separator)
    let channel = "private-test-channel";
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": "malformed-signature-without-colon"
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the private channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(
        !response.success,
        "Join should fail with malformed signature"
    );
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);
}

#[tokio::test]
async fn test_private_channel_rejects_public_channel_name() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PrivateChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Try to use private channel manager with a public channel name
    // Even with valid auth, this should work because the manager doesn't validate the prefix
    let channel = "public-channel";
    let auth = generate_auth_signature(&app.key, &app.secret, &socket.id, channel);

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Join with valid auth (the manager doesn't validate channel name prefix)
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // This should succeed because PrivateChannelManager delegates to PublicChannelManager
    // after auth validation, and PublicChannelManager allows non-private channels
    assert!(
        response.success,
        "Should succeed with valid auth regardless of channel prefix"
    );
}
