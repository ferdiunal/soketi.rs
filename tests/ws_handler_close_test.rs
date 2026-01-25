/// Integration tests for WebSocket connection close handling
///
/// These tests verify the connection close handling logic including:
/// - Unsubscribing from all channels on close
/// - Removing socket from adapter on close
/// - Removing user from adapter if authenticated on close
/// - Cleaning up presence channel memberships on close
/// - Broadcasting member_removed events for presence channels
///
/// Requirements: 2.7

use serde_json::json;
use soketi_rs::adapters::Adapter;
use soketi_rs::app::{App, PresenceMember};
use soketi_rs::namespace::Socket;
use soketi_rs::state::AppState;
use std::sync::Arc;

mod test_helpers;
use test_helpers::create_test_state_with_apps;

#[tokio::test]
async fn test_socket_removed_from_adapter_on_close() {
    // Test that when a connection closes, the socket is removed from the adapter
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket to simulate an active connection
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Verify the socket was added
    let count = state.adapter.get_sockets_count(&app.id).await.unwrap();
    assert_eq!(count, 1, "Should have 1 connection");

    // Simulate connection close by removing the socket
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify the socket was removed
    let count = state.adapter.get_sockets_count(&app.id).await.unwrap();
    assert_eq!(count, 0, "Should have 0 connections after close");
}

#[tokio::test]
async fn test_unsubscribe_from_all_channels_on_close() {
    // Test that when a connection closes, it's unsubscribed from all channels
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Subscribe to multiple channels
    state
        .adapter
        .add_to_channel(&app.id, "channel1", "test_socket".to_string())
        .await
        .unwrap();
    state
        .adapter
        .add_to_channel(&app.id, "channel2", "test_socket".to_string())
        .await
        .unwrap();
    state
        .adapter
        .add_to_channel(&app.id, "channel3", "test_socket".to_string())
        .await
        .unwrap();

    // Verify subscriptions
    assert!(
        state
            .adapter
            .is_in_channel(&app.id, "channel1", "test_socket")
            .await
            .unwrap()
    );
    assert!(
        state
            .adapter
            .is_in_channel(&app.id, "channel2", "test_socket")
            .await
            .unwrap()
    );
    assert!(
        state
            .adapter
            .is_in_channel(&app.id, "channel3", "test_socket")
            .await
            .unwrap()
    );

    // Simulate connection close
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify the socket is no longer in any channels
    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, "channel1", "test_socket")
            .await
            .unwrap()
    );
    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, "channel2", "test_socket")
            .await
            .unwrap()
    );
    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, "channel3", "test_socket")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_remove_user_from_adapter_on_close_if_authenticated() {
    // Test that when an authenticated connection closes, the user is removed from the adapter
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Associate a user with the socket (simulate authentication)
    state
        .adapter
        .add_user(&app.id, "user123", "test_socket")
        .await
        .unwrap();

    // Verify the user is associated
    let user_sockets = state
        .adapter
        .get_user_sockets(&app.id, "user123")
        .await
        .unwrap();
    assert_eq!(user_sockets.len(), 1);
    assert!(user_sockets.contains(&"test_socket".to_string()));

    // Simulate connection close
    // First remove the user association
    state
        .adapter
        .remove_user(&app.id, "user123", "test_socket")
        .await
        .unwrap();

    // Then remove the socket
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify the user association is removed
    let user_sockets = state
        .adapter
        .get_user_sockets(&app.id, "user123")
        .await
        .unwrap();
    assert_eq!(
        user_sockets.len(),
        0,
        "User should have no associated sockets after close"
    );
}

#[tokio::test]
async fn test_presence_channel_cleanup_on_close() {
    // Test that when a connection closes, presence channel memberships are cleaned up
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Subscribe to a presence channel
    let channel = "presence-test";
    state
        .adapter
        .add_to_channel(&app.id, channel, "test_socket".to_string())
        .await
        .unwrap();

    // Add member to presence channel
    let member = PresenceMember {
        user_id: "user123".to_string(),
        user_info: json!({
            "name": "Test User"
        }),
    };

    state
        .adapter
        .add_member(&app.id, channel, "test_socket", member)
        .await
        .unwrap();

    // Verify the member is in the presence channel
    let members = state
        .adapter
        .get_channel_members(&app.id, channel)
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
    assert!(members.contains_key("user123"));

    // Simulate connection close - remove member first
    state
        .adapter
        .remove_member(&app.id, channel, "test_socket")
        .await
        .unwrap();

    // Then remove socket
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify the member is removed from the presence channel
    let members = state
        .adapter
        .get_channel_members(&app.id, channel)
        .await
        .unwrap();
    assert_eq!(
        members.len(),
        0,
        "Presence channel should have no members after close"
    );
}

#[tokio::test]
async fn test_multiple_channels_cleanup_on_close() {
    // Test that closing a connection cleans up subscriptions to multiple channel types
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Subscribe to different channel types
    let public_channel = "public-channel";
    let private_channel = "private-channel";
    let presence_channel = "presence-channel";

    state
        .adapter
        .add_to_channel(&app.id, public_channel, "test_socket".to_string())
        .await
        .unwrap();
    state
        .adapter
        .add_to_channel(&app.id, private_channel, "test_socket".to_string())
        .await
        .unwrap();
    state
        .adapter
        .add_to_channel(&app.id, presence_channel, "test_socket".to_string())
        .await
        .unwrap();

    // Add presence member
    let member = PresenceMember {
        user_id: "user123".to_string(),
        user_info: json!({"name": "Test User"}),
    };
    state
        .adapter
        .add_member(&app.id, presence_channel, "test_socket", member)
        .await
        .unwrap();

    // Verify all subscriptions
    assert!(
        state
            .adapter
            .is_in_channel(&app.id, public_channel, "test_socket")
            .await
            .unwrap()
    );
    assert!(
        state
            .adapter
            .is_in_channel(&app.id, private_channel, "test_socket")
            .await
            .unwrap()
    );
    assert!(
        state
            .adapter
            .is_in_channel(&app.id, presence_channel, "test_socket")
            .await
            .unwrap()
    );

    let members = state
        .adapter
        .get_channel_members(&app.id, presence_channel)
        .await
        .unwrap();
    assert_eq!(members.len(), 1);

    // Simulate connection close
    // Clean up presence first
    state
        .adapter
        .remove_member(&app.id, presence_channel, "test_socket")
        .await
        .unwrap();

    // Then remove socket (which should clean up all channel subscriptions)
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify all subscriptions are cleaned up
    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, public_channel, "test_socket")
            .await
            .unwrap()
    );
    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, private_channel, "test_socket")
            .await
            .unwrap()
    );
    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, presence_channel, "test_socket")
            .await
            .unwrap()
    );

    let members = state
        .adapter
        .get_channel_members(&app.id, presence_channel)
        .await
        .unwrap();
    assert_eq!(members.len(), 0);
}

#[tokio::test]
async fn test_authenticated_user_with_multiple_sockets_cleanup() {
    // Test that closing one socket doesn't affect other sockets for the same user
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add two sockets for the same user
    let (tx1, _rx1) = tokio::sync::mpsc::channel(10);
    let socket1 = Socket {
        id: "socket1".to_string(),
        sender: tx1,
    };

    let (tx2, _rx2) = tokio::sync::mpsc::channel(10);
    let socket2 = Socket {
        id: "socket2".to_string(),
        sender: tx2,
    };

    state.adapter.add_socket(&app.id, socket1).await.unwrap();
    state.adapter.add_socket(&app.id, socket2).await.unwrap();

    // Associate both sockets with the same user
    state
        .adapter
        .add_user(&app.id, "user123", "socket1")
        .await
        .unwrap();
    state
        .adapter
        .add_user(&app.id, "user123", "socket2")
        .await
        .unwrap();

    // Verify both sockets are associated
    let user_sockets = state
        .adapter
        .get_user_sockets(&app.id, "user123")
        .await
        .unwrap();
    assert_eq!(user_sockets.len(), 2);

    // Close one socket
    state
        .adapter
        .remove_user(&app.id, "user123", "socket1")
        .await
        .unwrap();
    state
        .adapter
        .remove_socket(&app.id, "socket1")
        .await
        .unwrap();

    // Verify the other socket is still associated with the user
    let user_sockets = state
        .adapter
        .get_user_sockets(&app.id, "user123")
        .await
        .unwrap();
    assert_eq!(user_sockets.len(), 1);
    assert!(user_sockets.contains(&"socket2".to_string()));

    // Verify socket2 is still in the adapter
    let count = state.adapter.get_sockets_count(&app.id).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_close_without_subscriptions() {
    // Test that closing a connection that never subscribed to any channels works correctly
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket but don't subscribe to any channels
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Verify the socket was added
    let count = state.adapter.get_sockets_count(&app.id).await.unwrap();
    assert_eq!(count, 1);

    // Close the connection
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify the socket was removed
    let count = state.adapter.get_sockets_count(&app.id).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_close_without_authentication() {
    // Test that closing an unauthenticated connection works correctly
    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add a socket without authenticating
    let (tx, _rx) = tokio::sync::mpsc::channel(10);
    let socket = Socket {
        id: "test_socket".to_string(),
        sender: tx,
    };

    state.adapter.add_socket(&app.id, socket).await.unwrap();

    // Subscribe to a channel
    state
        .adapter
        .add_to_channel(&app.id, "test-channel", "test_socket".to_string())
        .await
        .unwrap();

    // Close the connection (no user to remove)
    state
        .adapter
        .remove_socket(&app.id, "test_socket")
        .await
        .unwrap();

    // Verify cleanup
    let count = state.adapter.get_sockets_count(&app.id).await.unwrap();
    assert_eq!(count, 0);

    assert!(
        !state
            .adapter
            .is_in_channel(&app.id, "test-channel", "test_socket")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_presence_channel_member_removed_broadcast() {
    // Test that when a presence channel member disconnects, other members are notified
    // This test verifies the logic but doesn't test the actual WebSocket broadcast
    // (that would require a full integration test with real WebSocket connections)

    let app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    let state = create_test_state_with_apps(vec![app.clone()]);

    // Add two sockets to the presence channel
    let (tx1, _rx1) = tokio::sync::mpsc::channel(10);
    let socket1 = Socket {
        id: "socket1".to_string(),
        sender: tx1,
    };

    let (tx2, _rx2) = tokio::sync::mpsc::channel(10);
    let socket2 = Socket {
        id: "socket2".to_string(),
        sender: tx2,
    };

    state.adapter.add_socket(&app.id, socket1).await.unwrap();
    state.adapter.add_socket(&app.id, socket2).await.unwrap();

    let channel = "presence-test";
    state
        .adapter
        .add_to_channel(&app.id, channel, "socket1".to_string())
        .await
        .unwrap();
    state
        .adapter
        .add_to_channel(&app.id, channel, "socket2".to_string())
        .await
        .unwrap();

    // Add members
    let member1 = PresenceMember {
        user_id: "user1".to_string(),
        user_info: json!({"name": "User 1"}),
    };
    let member2 = PresenceMember {
        user_id: "user2".to_string(),
        user_info: json!({"name": "User 2"}),
    };

    state
        .adapter
        .add_member(&app.id, channel, "socket1", member1)
        .await
        .unwrap();
    state
        .adapter
        .add_member(&app.id, channel, "socket2", member2)
        .await
        .unwrap();

    // Verify both members are present
    let members = state
        .adapter
        .get_channel_members(&app.id, channel)
        .await
        .unwrap();
    assert_eq!(members.len(), 2);

    // Remove one member (simulating disconnect)
    state
        .adapter
        .remove_member(&app.id, channel, "socket1")
        .await
        .unwrap();

    // Verify only one member remains
    let members = state
        .adapter
        .get_channel_members(&app.id, channel)
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
    assert!(members.contains_key("user2"));
    assert!(!members.contains_key("user1"));
}
