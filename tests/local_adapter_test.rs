use axum::extract::ws::Message;
use soketi_rs::adapters::{Adapter, local::LocalAdapter};
use soketi_rs::app::PresenceMember;
use soketi_rs::namespace::Socket;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_local_adapter_socket_management() {
    let adapter = LocalAdapter::new();
    adapter.init().await.unwrap();

    // Create a test socket
    let (tx, _rx) = mpsc::channel::<Message>(100);
    let socket = Socket {
        id: "socket-1".to_string(),
        sender: tx,
    };

    // Test add_socket
    adapter.add_socket("app-1", socket.clone()).await.unwrap();

    // Test get_sockets_count
    let count = adapter.get_sockets_count("app-1").await.unwrap();
    assert_eq!(count, 1);

    // Test remove_socket
    adapter.remove_socket("app-1", "socket-1").await.unwrap();
    let count = adapter.get_sockets_count("app-1").await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_local_adapter_channel_management() {
    let adapter = LocalAdapter::new();
    adapter.init().await.unwrap();

    // Create a test socket
    let (tx, _rx) = mpsc::channel::<Message>(100);
    let socket = Socket {
        id: "socket-1".to_string(),
        sender: tx,
    };

    adapter.add_socket("app-1", socket.clone()).await.unwrap();

    // Test add_to_channel
    let count = adapter
        .add_to_channel("app-1", "test-channel", "socket-1".to_string())
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Test is_in_channel
    let is_in = adapter
        .is_in_channel("app-1", "test-channel", "socket-1")
        .await
        .unwrap();
    assert!(is_in);

    // Test get_channel_sockets_count
    let count = adapter
        .get_channel_sockets_count("app-1", "test-channel")
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Test get_channels_with_sockets_count
    let channels = adapter
        .get_channels_with_sockets_count("app-1")
        .await
        .unwrap();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels.get("test-channel"), Some(&1));

    // Test remove_from_channel
    let count = adapter
        .remove_from_channel("app-1", "test-channel", "socket-1")
        .await
        .unwrap();
    assert_eq!(count, 0);

    let is_in = adapter
        .is_in_channel("app-1", "test-channel", "socket-1")
        .await
        .unwrap();
    assert!(!is_in);
}

#[tokio::test]
async fn test_local_adapter_presence_management() {
    let adapter = LocalAdapter::new();
    adapter.init().await.unwrap();

    let member = PresenceMember {
        user_id: "user-1".to_string(),
        user_info: serde_json::json!({"name": "Test User"}),
    };

    // Test add_member
    adapter
        .add_member("app-1", "presence-channel", "socket-1", member.clone())
        .await
        .unwrap();

    // Test get_channel_members_count
    let count = adapter
        .get_channel_members_count("app-1", "presence-channel")
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Test get_channel_members
    let members = adapter
        .get_channel_members("app-1", "presence-channel")
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
    assert!(members.contains_key("user-1"));

    // Test remove_member
    adapter
        .remove_member("app-1", "presence-channel", "socket-1")
        .await
        .unwrap();
    let count = adapter
        .get_channel_members_count("app-1", "presence-channel")
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_local_adapter_user_tracking() {
    let adapter = LocalAdapter::new();
    adapter.init().await.unwrap();

    // Test add_user
    adapter
        .add_user("app-1", "user-1", "socket-1")
        .await
        .unwrap();
    adapter
        .add_user("app-1", "user-1", "socket-2")
        .await
        .unwrap();

    // Test get_user_sockets
    let sockets = adapter.get_user_sockets("app-1", "user-1").await.unwrap();
    assert_eq!(sockets.len(), 2);
    assert!(sockets.contains(&"socket-1".to_string()));
    assert!(sockets.contains(&"socket-2".to_string()));

    // Test remove_user
    adapter
        .remove_user("app-1", "user-1", "socket-1")
        .await
        .unwrap();
    let sockets = adapter.get_user_sockets("app-1", "user-1").await.unwrap();
    assert_eq!(sockets.len(), 1);
    assert!(sockets.contains(&"socket-2".to_string()));
}

#[tokio::test]
async fn test_local_adapter_namespace_isolation() {
    let adapter = LocalAdapter::new();
    adapter.init().await.unwrap();

    // Create sockets for different apps
    let (tx1, _rx1) = mpsc::channel::<Message>(100);
    let socket1 = Socket {
        id: "socket-1".to_string(),
        sender: tx1,
    };

    let (tx2, _rx2) = mpsc::channel::<Message>(100);
    let socket2 = Socket {
        id: "socket-2".to_string(),
        sender: tx2,
    };

    adapter.add_socket("app-1", socket1).await.unwrap();
    adapter.add_socket("app-2", socket2).await.unwrap();

    // Verify namespace isolation
    let count1 = adapter.get_sockets_count("app-1").await.unwrap();
    let count2 = adapter.get_sockets_count("app-2").await.unwrap();
    assert_eq!(count1, 1);
    assert_eq!(count2, 1);

    // Clear one namespace
    adapter.clear_namespace("app-1").await.unwrap();
    let count1 = adapter.get_sockets_count("app-1").await.unwrap();
    let count2 = adapter.get_sockets_count("app-2").await.unwrap();
    assert_eq!(count1, 0);
    assert_eq!(count2, 1);
}

#[tokio::test]
async fn test_local_adapter_disconnect() {
    let adapter = LocalAdapter::new();
    adapter.init().await.unwrap();

    let (tx, _rx) = mpsc::channel::<Message>(100);
    let socket = Socket {
        id: "socket-1".to_string(),
        sender: tx,
    };

    adapter.add_socket("app-1", socket).await.unwrap();

    // Test disconnect clears all namespaces
    adapter.disconnect().await.unwrap();
    let count = adapter.get_sockets_count("app-1").await.unwrap();
    assert_eq!(count, 0);
}
