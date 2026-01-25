use axum::extract::ws::Message;
use soketi_rs::adapters::{Adapter, nats::NatsAdapter};
use soketi_rs::app::PresenceMember;
use soketi_rs::config::NatsAdapterConfig;
use soketi_rs::namespace::Socket;
use tokio::sync::mpsc;

// Helper function to check if NATS is available
async fn is_nats_available() -> bool {
    let config = NatsAdapterConfig::default();

    match async_nats::connect(&config.servers[0]).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_nats_adapter_creation() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig::default();
    let adapter = NatsAdapter::new(config).await;
    assert!(adapter.is_ok(), "Failed to create NatsAdapter");
}

#[tokio::test]
async fn test_nats_adapter_init() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig::default();
    let adapter = NatsAdapter::new(config).await.unwrap();
    let result = adapter.init().await;
    assert!(result.is_ok(), "Failed to initialize NatsAdapter");
}

#[tokio::test]
async fn test_nats_adapter_socket_management() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };
    let adapter = NatsAdapter::new(config).await.unwrap();
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

    // Cleanup
    adapter.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_nats_adapter_channel_management() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };
    let adapter = NatsAdapter::new(config).await.unwrap();
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

    // Cleanup
    adapter.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_nats_adapter_presence_management() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };
    let adapter = NatsAdapter::new(config).await.unwrap();
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

    // Give NATS pub/sub time to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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

    // Give NATS pub/sub time to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let count = adapter
        .get_channel_members_count("app-1", "presence-channel")
        .await
        .unwrap();
    assert_eq!(count, 0);

    // Cleanup
    adapter.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_nats_adapter_user_tracking() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };
    let adapter = NatsAdapter::new(config).await.unwrap();
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

    // Cleanup
    adapter.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_nats_adapter_message_broadcast() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };

    // Create two adapter instances to simulate multiple servers
    let adapter1 = NatsAdapter::new(config.clone()).await.unwrap();
    adapter1.init().await.unwrap();

    let adapter2 = NatsAdapter::new(config).await.unwrap();
    adapter2.init().await.unwrap();

    // Give adapters time to connect and subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Create sockets on both adapters
    let (tx1, mut rx1) = mpsc::channel::<Message>(100);
    let socket1 = Socket {
        id: "socket-1".to_string(),
        sender: tx1,
    };

    let (tx2, mut rx2) = mpsc::channel::<Message>(100);
    let socket2 = Socket {
        id: "socket-2".to_string(),
        sender: tx2,
    };

    adapter1.add_socket("app-1", socket1).await.unwrap();
    adapter1
        .add_to_channel("app-1", "test-channel", "socket-1".to_string())
        .await
        .unwrap();

    adapter2.add_socket("app-1", socket2).await.unwrap();
    adapter2
        .add_to_channel("app-1", "test-channel", "socket-2".to_string())
        .await
        .unwrap();

    // Send a message from adapter1
    let test_message = r#"{"event":"test","data":"hello"}"#;
    adapter1
        .send("app-1", "test-channel", test_message, None)
        .await
        .unwrap();

    // Give NATS pub/sub time to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Check that socket1 received the message (local delivery)
    if let Ok(msg) = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx1.recv()).await
    {
        if let Some(Message::Text(text)) = msg {
            assert_eq!(text, test_message);
        } else {
            panic!("Expected text message on socket1");
        }
    } else {
        panic!("Socket1 did not receive message");
    }

    // Check that socket2 received the message (via NATS)
    if let Ok(msg) = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx2.recv()).await
    {
        if let Some(Message::Text(text)) = msg {
            assert_eq!(text, test_message);
        } else {
            panic!("Expected text message on socket2");
        }
    } else {
        panic!("Socket2 did not receive message via NATS");
    }

    // Cleanup
    adapter1.disconnect().await.unwrap();
    adapter2.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_nats_adapter_terminate_user_connections() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };

    // Create two adapter instances
    let adapter1 = NatsAdapter::new(config.clone()).await.unwrap();
    adapter1.init().await.unwrap();

    let adapter2 = NatsAdapter::new(config).await.unwrap();
    adapter2.init().await.unwrap();

    // Give adapters time to connect and subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Add user sockets on both adapters
    adapter1
        .add_user("app-1", "user-1", "socket-1")
        .await
        .unwrap();
    adapter2
        .add_user("app-1", "user-1", "socket-2")
        .await
        .unwrap();

    // Terminate user connections from adapter1
    adapter1
        .terminate_user_connections("app-1", "user-1")
        .await
        .unwrap();

    // Give NATS pub/sub time to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Verify user sockets are removed on both adapters
    let sockets1 = adapter1.get_user_sockets("app-1", "user-1").await.unwrap();
    let sockets2 = adapter2.get_user_sockets("app-1", "user-1").await.unwrap();

    assert_eq!(sockets1.len(), 0);
    assert_eq!(sockets2.len(), 0);

    // Cleanup
    adapter1.disconnect().await.unwrap();
    adapter2.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_nats_adapter_except_socket_id() {
    if !is_nats_available().await {
        println!("Skipping test: NATS is not available");
        return;
    }

    let config = NatsAdapterConfig {
        prefix: format!("test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };

    let adapter = NatsAdapter::new(config).await.unwrap();
    adapter.init().await.unwrap();

    // Give adapter time to connect and subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Create two sockets
    let (tx1, mut rx1) = mpsc::channel::<Message>(100);
    let socket1 = Socket {
        id: "socket-1".to_string(),
        sender: tx1,
    };

    let (tx2, mut rx2) = mpsc::channel::<Message>(100);
    let socket2 = Socket {
        id: "socket-2".to_string(),
        sender: tx2,
    };

    adapter.add_socket("app-1", socket1).await.unwrap();
    adapter
        .add_to_channel("app-1", "test-channel", "socket-1".to_string())
        .await
        .unwrap();

    adapter.add_socket("app-1", socket2).await.unwrap();
    adapter
        .add_to_channel("app-1", "test-channel", "socket-2".to_string())
        .await
        .unwrap();

    // Send a message excluding socket-1
    let test_message = r#"{"event":"test","data":"hello"}"#;
    adapter
        .send("app-1", "test-channel", test_message, Some("socket-1"))
        .await
        .unwrap();

    // Give time for message delivery
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Socket1 should NOT receive the message
    let result1 = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx1.recv()).await;
    assert!(
        result1.is_err(),
        "Socket1 should not have received the message"
    );

    // Socket2 should receive the message
    if let Ok(msg) = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx2.recv()).await
    {
        if let Some(Message::Text(text)) = msg {
            assert_eq!(text, test_message);
        } else {
            panic!("Expected text message on socket2");
        }
    } else {
        panic!("Socket2 did not receive message");
    }

    // Cleanup
    adapter.disconnect().await.unwrap();
}
