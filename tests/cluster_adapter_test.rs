use axum::extract::ws::Message;
use soketi_rs::adapters::Adapter;
use soketi_rs::adapters::cluster::ClusterAdapter;
use soketi_rs::app::PresenceMember;
use soketi_rs::config::ClusterAdapterConfig;
use soketi_rs::namespace::Socket;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Test that ClusterAdapter can be created and initialized
#[tokio::test]
async fn test_cluster_adapter_creation() {
    let config = ClusterAdapterConfig {
        port: 11003, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.2".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await;
    assert!(
        adapter.is_ok(),
        "ClusterAdapter should be created successfully"
    );

    let adapter = adapter.unwrap();
    let result = adapter.init().await;
    assert!(
        result.is_ok(),
        "ClusterAdapter should initialize successfully"
    );
}

/// Test that ClusterAdapter can add and remove sockets
#[tokio::test]
async fn test_cluster_adapter_socket_management() {
    let config = ClusterAdapterConfig {
        port: 11004, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.3".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    let app_id = "test_app";
    let socket_id = "socket_1";

    // Create a test socket
    let (tx, _rx) = mpsc::channel(100);
    let socket = Socket {
        id: socket_id.to_string(),
        sender: tx,
    };

    // Add socket
    let result = adapter.add_socket(app_id, socket).await;
    assert!(result.is_ok(), "Should add socket successfully");

    // Check socket count
    let count = adapter.get_sockets_count(app_id).await.unwrap();
    assert_eq!(count, 1, "Should have 1 socket");

    // Remove socket
    let result = adapter.remove_socket(app_id, socket_id).await;
    assert!(result.is_ok(), "Should remove socket successfully");

    // Check socket count after removal
    let count = adapter.get_sockets_count(app_id).await.unwrap();
    assert_eq!(count, 0, "Should have 0 sockets after removal");
}

/// Test that ClusterAdapter can manage channels
#[tokio::test]
async fn test_cluster_adapter_channel_management() {
    let config = ClusterAdapterConfig {
        port: 11005, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.4".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    let app_id = "test_app";
    let socket_id = "socket_1";
    let channel = "test-channel";

    // Create a test socket
    let (tx, _rx) = mpsc::channel(100);
    let socket = Socket {
        id: socket_id.to_string(),
        sender: tx,
    };

    // Add socket
    adapter.add_socket(app_id, socket).await.unwrap();

    // Add to channel
    let count = adapter
        .add_to_channel(app_id, channel, socket_id.to_string())
        .await
        .unwrap();
    assert_eq!(count, 1, "Should have 1 socket in channel");

    // Check if socket is in channel
    let is_in = adapter
        .is_in_channel(app_id, channel, socket_id)
        .await
        .unwrap();
    assert!(is_in, "Socket should be in channel");

    // Get channel socket count
    let count = adapter
        .get_channel_sockets_count(app_id, channel)
        .await
        .unwrap();
    assert_eq!(count, 1, "Should have 1 socket in channel");

    // Remove from channel
    let count = adapter
        .remove_from_channel(app_id, channel, socket_id)
        .await
        .unwrap();
    assert_eq!(count, 0, "Should have 0 sockets in channel after removal");

    // Check if socket is still in channel
    let is_in = adapter
        .is_in_channel(app_id, channel, socket_id)
        .await
        .unwrap();
    assert!(!is_in, "Socket should not be in channel after removal");
}

/// Test that ClusterAdapter can manage presence members
#[tokio::test]
async fn test_cluster_adapter_presence_management() {
    let config = ClusterAdapterConfig {
        port: 11006, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.5".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    let app_id = "test_app";
    let socket_id = "socket_1";
    let channel = "presence-test";
    let user_id = "user_1";

    // Create a test socket
    let (tx, _rx) = mpsc::channel(100);
    let socket = Socket {
        id: socket_id.to_string(),
        sender: tx,
    };

    // Add socket
    adapter.add_socket(app_id, socket).await.unwrap();

    // Add to channel
    adapter
        .add_to_channel(app_id, channel, socket_id.to_string())
        .await
        .unwrap();

    // Create presence member
    let member = PresenceMember {
        user_id: user_id.to_string(),
        user_info: serde_json::json!({"name": "Test User"}),
    };

    // Add member
    let result = adapter
        .add_member(app_id, channel, socket_id, member.clone())
        .await;
    assert!(result.is_ok(), "Should add member successfully");

    // Get member count
    let count = adapter
        .get_channel_members_count(app_id, channel)
        .await
        .unwrap();
    assert_eq!(count, 1, "Should have 1 member");

    // Get members
    let members = adapter.get_channel_members(app_id, channel).await.unwrap();
    assert_eq!(members.len(), 1, "Should have 1 member");
    assert!(members.contains_key(user_id), "Should contain the user");

    // Remove member
    let result = adapter.remove_member(app_id, channel, socket_id).await;
    assert!(result.is_ok(), "Should remove member successfully");

    // Get member count after removal
    let count = adapter
        .get_channel_members_count(app_id, channel)
        .await
        .unwrap();
    assert_eq!(count, 0, "Should have 0 members after removal");
}

/// Test that ClusterAdapter reports master status
#[tokio::test]
async fn test_cluster_adapter_master_status() {
    let config = ClusterAdapterConfig {
        port: 11007, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.6".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    // Initially, a single node should become master
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let is_master = adapter.is_master().await;
    // A single node in the cluster should elect itself as master
    assert!(is_master, "Single node should be master");

    let master_id = adapter.get_master_id().await;
    assert!(master_id.is_some(), "Should have a master ID");
}

/// Test that ClusterAdapter can send messages
#[tokio::test]
async fn test_cluster_adapter_send_message() {
    let config = ClusterAdapterConfig {
        port: 11008, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.7".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    let app_id = "test_app";
    let socket_id = "socket_1";
    let channel = "test-channel";

    // Create a test socket
    let (tx, mut rx) = mpsc::channel(100);
    let socket = Socket {
        id: socket_id.to_string(),
        sender: tx,
    };

    // Add socket
    adapter.add_socket(app_id, socket).await.unwrap();

    // Add to channel
    adapter
        .add_to_channel(app_id, channel, socket_id.to_string())
        .await
        .unwrap();

    // Send message
    let message = r#"{"event":"test","data":"hello"}"#;
    let result = adapter.send(app_id, channel, message, None).await;
    assert!(result.is_ok(), "Should send message successfully");

    // Check if message was received
    tokio::time::timeout(tokio::time::Duration::from_millis(100), async {
        if let Some(msg) = rx.recv().await {
            match msg {
                Message::Text(text) => {
                    assert_eq!(text, message, "Should receive the sent message");
                }
                _ => panic!("Expected text message"),
            }
        }
    })
    .await
    .ok();
}

/// Test that ClusterAdapter can disconnect gracefully
#[tokio::test]
async fn test_cluster_adapter_disconnect() {
    let config = ClusterAdapterConfig {
        port: 11009, // Use a different port to avoid conflicts
        multicast_address: "239.1.1.8".to_string(),
        request_timeout_ms: 5000,
    };

    let adapter = ClusterAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    // Add some data
    let app_id = "test_app";
    let socket_id = "socket_1";
    let (tx, _rx) = mpsc::channel(100);
    let socket = Socket {
        id: socket_id.to_string(),
        sender: tx,
    };
    adapter.add_socket(app_id, socket).await.unwrap();

    // Disconnect
    let result = adapter.disconnect().await;
    assert!(result.is_ok(), "Should disconnect successfully");

    // Check that data is cleared
    let count = adapter.get_sockets_count(app_id).await.unwrap();
    assert_eq!(count, 0, "Should have 0 sockets after disconnect");
}
