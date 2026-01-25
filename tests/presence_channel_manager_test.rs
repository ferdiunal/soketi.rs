use axum::extract::ws::Message;
use hex;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use soketi_rs::adapters::Adapter;
use soketi_rs::adapters::local::LocalAdapter;
use soketi_rs::app::App;
use soketi_rs::channels::presence::PresenceChannelManager;
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

/// Helper function to create a test app with limits
fn create_test_app_with_limits(max_members: Option<u64>, max_size_kb: Option<f64>) -> App {
    let mut app = create_test_app();
    app.max_presence_members_per_channel = max_members;
    app.max_presence_member_size_in_kb = max_size_kb;
    app
}

/// Helper function to create a test socket
fn create_test_socket(id: &str) -> Socket {
    let (tx, _rx) = mpsc::channel::<Message>(100);
    Socket {
        id: id.to_string(),
        sender: tx,
    }
}

/// Helper function to generate a valid auth signature for presence channels
/// This mimics what a client would do to authenticate
fn generate_presence_auth_signature(
    app_key: &str,
    app_secret: &str,
    socket_id: &str,
    channel: &str,
    channel_data: &str,
) -> String {
    let data_to_sign = format!("{}:{}:{}", socket_id, channel, channel_data);
    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data_to_sign.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}:{}", app_key, signature)
}

#[tokio::test]
async fn test_presence_channel_join_with_valid_auth() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate valid auth signature with channel_data
    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": "user-123",
        "user_info": {
            "name": "Test User"
        }
    })
    .to_string();
    let auth =
        generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, &channel_data);

    // Create subscription message with auth and channel_data
    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Join the presence channel
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

    // Verify member is tracked
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 1, "Should have 1 member");
    assert!(members.contains_key("user-123"), "Should contain user-123");

    let member = members.get("user-123").unwrap();
    assert_eq!(member.user_id, "user-123");
    assert_eq!(member.user_info["name"], "Test User");
}

#[tokio::test]
async fn test_presence_channel_join_with_invalid_auth() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Create subscription message with invalid auth
    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": "user-123"
    })
    .to_string();

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": "invalid-auth-signature",
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the presence channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(!response.success, "Join should fail with invalid auth");
    assert_eq!(response.error_code, Some(4009));
    assert!(response.error_message.is_some());
    assert!(response.error_message.unwrap().contains("unauthorized"));
    assert_eq!(response.auth_error, true);

    // Verify socket is NOT in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");

    // Verify no members
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 0, "Should have no members");
}

#[tokio::test]
async fn test_presence_channel_join_without_channel_data() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth without channel_data (should fail)
    let channel = "presence-test-channel";
    let auth = generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, "{}");

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the presence channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure - should fail because channel_data doesn't have user_id
    assert!(
        !response.success,
        "Join should fail without user_id in channel_data"
    );
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);
}

#[tokio::test]
async fn test_presence_channel_join_with_empty_user_id() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth with empty user_id
    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": ""
    })
    .to_string();
    let auth =
        generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, &channel_data);

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the presence channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(!response.success, "Join should fail with empty user_id");
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);
}

#[tokio::test]
async fn test_presence_channel_join_with_invalid_json_channel_data() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth with invalid JSON channel_data
    let channel = "presence-test-channel";
    let channel_data = "not-valid-json";
    let auth =
        generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, channel_data);

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the presence channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(!response.success, "Join should fail with invalid JSON");
    assert_eq!(response.error_code, Some(4009));
    assert_eq!(response.auth_error, true);
}

#[tokio::test]
async fn test_presence_channel_join_exceeds_member_limit() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app with member limit of 2
    let app = create_test_app_with_limits(Some(2), None);

    // Add 2 members first
    for i in 1..=2 {
        let socket = create_test_socket(&format!("socket-{}", i));
        adapter.add_socket(&app.id, socket.clone()).await.unwrap();

        let channel = "presence-test-channel";
        let channel_data = json!({
            "user_id": format!("user-{}", i)
        })
        .to_string();
        let auth = generate_presence_auth_signature(
            &app.key,
            &app.secret,
            &socket.id,
            channel,
            &channel_data,
        );

        let message = PusherMessage {
            event: "pusher:subscribe".to_string(),
            data: Some(json!({
                "auth": auth,
                "channel_data": channel_data
            })),
            channel: Some(channel.to_string()),
            socket_id: None,
        };

        let response = manager.join(&app, &socket, channel, Some(message)).await;
        assert!(response.success, "First 2 members should join successfully");
    }

    // Try to add a 3rd member (should fail)
    let socket3 = create_test_socket("socket-3");
    adapter.add_socket(&app.id, socket3.clone()).await.unwrap();

    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": "user-3"
    })
    .to_string();
    let auth = generate_presence_auth_signature(
        &app.key,
        &app.secret,
        &socket3.id,
        channel,
        &channel_data,
    );

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    let response = manager.join(&app, &socket3, channel, Some(message)).await;

    // Verify failure
    assert!(
        !response.success,
        "Join should fail when member limit is reached"
    );
    assert_eq!(response.error_code, Some(4100));
    assert!(response.error_message.unwrap().contains("maximum members"));
    assert_eq!(response.auth_error, false);
}

#[tokio::test]
async fn test_presence_channel_join_exceeds_member_size_limit() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app with member size limit of 1 KB
    let app = create_test_app_with_limits(None, Some(1.0));
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Create channel_data larger than 1 KB
    let large_data = "x".repeat(2000); // 2 KB of data
    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": "user-123",
        "user_info": {
            "data": large_data
        }
    })
    .to_string();
    let auth =
        generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, &channel_data);

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Try to join the presence channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify failure
    assert!(
        !response.success,
        "Join should fail when member size exceeds limit"
    );
    assert_eq!(response.error_code, Some(4301));
    assert!(
        response
            .error_message
            .unwrap()
            .contains("exceeds maximum size")
    );
}

#[tokio::test]
async fn test_presence_channel_multiple_members() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app
    let app = create_test_app();

    // Add 3 members
    for i in 1..=3 {
        let socket = create_test_socket(&format!("socket-{}", i));
        adapter.add_socket(&app.id, socket.clone()).await.unwrap();

        let channel = "presence-test-channel";
        let channel_data = json!({
            "user_id": format!("user-{}", i),
            "user_info": {
                "name": format!("User {}", i)
            }
        })
        .to_string();
        let auth = generate_presence_auth_signature(
            &app.key,
            &app.secret,
            &socket.id,
            channel,
            &channel_data,
        );

        let message = PusherMessage {
            event: "pusher:subscribe".to_string(),
            data: Some(json!({
                "auth": auth,
                "channel_data": channel_data
            })),
            channel: Some(channel.to_string()),
            socket_id: None,
        };

        let response = manager.join(&app, &socket, channel, Some(message)).await;
        assert!(response.success, "Member {} should join successfully", i);
    }

    // Verify all members are tracked
    let members = adapter
        .get_channel_members(&app.id, "presence-test-channel")
        .await
        .unwrap();
    assert_eq!(members.len(), 3, "Should have 3 members");

    for i in 1..=3 {
        let user_id = format!("user-{}", i);
        assert!(members.contains_key(&user_id), "Should contain {}", user_id);
        let member = members.get(&user_id).unwrap();
        assert_eq!(member.user_info["name"], format!("User {}", i));
    }
}

#[tokio::test]
async fn test_presence_channel_leave() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Join the presence channel
    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": "user-123",
        "user_info": {
            "name": "Test User"
        }
    })
    .to_string();
    let auth =
        generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, &channel_data);

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    let response = manager.join(&app, &socket, channel, Some(message)).await;
    assert!(response.success, "Join should succeed");

    // Verify member is tracked
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 1, "Should have 1 member");

    // Leave the channel
    manager.leave(&app, &socket.id, channel).await.unwrap();

    // Verify socket is not in channel
    let is_in = adapter
        .is_in_channel(&app.id, channel, &socket.id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in the channel");

    // Verify member is removed
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 0, "Should have no members");
}

#[tokio::test]
async fn test_presence_channel_same_user_multiple_sockets() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app
    let app = create_test_app();

    // Add same user with 2 different sockets
    let socket1 = create_test_socket("socket-1");
    let socket2 = create_test_socket("socket-2");

    adapter.add_socket(&app.id, socket1.clone()).await.unwrap();
    adapter.add_socket(&app.id, socket2.clone()).await.unwrap();

    let channel = "presence-test-channel";

    // Join with first socket
    let channel_data = json!({
        "user_id": "user-123",
        "user_info": {
            "name": "Test User"
        }
    })
    .to_string();
    let auth1 = generate_presence_auth_signature(
        &app.key,
        &app.secret,
        &socket1.id,
        channel,
        &channel_data,
    );

    let message1 = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth1,
            "channel_data": channel_data.clone()
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    let response1 = manager.join(&app, &socket1, channel, Some(message1)).await;
    assert!(response1.success, "First socket should join successfully");

    // Join with second socket (same user)
    let auth2 = generate_presence_auth_signature(
        &app.key,
        &app.secret,
        &socket2.id,
        channel,
        &channel_data,
    );

    let message2 = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth2,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    let response2 = manager.join(&app, &socket2, channel, Some(message2)).await;
    assert!(response2.success, "Second socket should join successfully");

    // Verify only 1 unique member (same user_id)
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 1, "Should have 1 unique member");
    assert!(members.contains_key("user-123"));

    // Leave with first socket
    manager.leave(&app, &socket1.id, channel).await.unwrap();

    // Verify member is still present (second socket still connected)
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 1, "Member should still be present");

    // Leave with second socket
    manager.leave(&app, &socket2.id, channel).await.unwrap();

    // Verify member is now removed
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(
        members.len(),
        0,
        "Member should be removed after all sockets leave"
    );
}

#[tokio::test]
async fn test_presence_channel_join_with_minimal_channel_data() {
    // Create adapter and channel manager
    let adapter = Arc::new(LocalAdapter::new()) as Arc<dyn Adapter>;
    adapter.init().await.unwrap();
    let manager = PresenceChannelManager::new(adapter.clone());

    // Create test app and socket
    let app = create_test_app();
    let socket = create_test_socket("socket-1");

    // Add socket to adapter first
    adapter.add_socket(&app.id, socket.clone()).await.unwrap();

    // Generate auth with minimal channel_data (only user_id, no user_info)
    let channel = "presence-test-channel";
    let channel_data = json!({
        "user_id": "user-123"
    })
    .to_string();
    let auth =
        generate_presence_auth_signature(&app.key, &app.secret, &socket.id, channel, &channel_data);

    let message = PusherMessage {
        event: "pusher:subscribe".to_string(),
        data: Some(json!({
            "auth": auth,
            "channel_data": channel_data
        })),
        channel: Some(channel.to_string()),
        socket_id: None,
    };

    // Join the presence channel
    let response = manager.join(&app, &socket, channel, Some(message)).await;

    // Verify success
    assert!(
        response.success,
        "Join should succeed with minimal channel_data"
    );

    // Verify member is tracked with null user_info
    let members = adapter.get_channel_members(&app.id, channel).await.unwrap();
    assert_eq!(members.len(), 1, "Should have 1 member");
    let member = members.get("user-123").unwrap();
    assert_eq!(member.user_id, "user-123");
    assert_eq!(member.user_info, serde_json::Value::Null);
}
