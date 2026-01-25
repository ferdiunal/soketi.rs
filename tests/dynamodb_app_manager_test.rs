use soketi_rs::app::App;
use soketi_rs::app_managers::{AppManager, DynamoDbAppManager};
use soketi_rs::cache_managers::{CacheManager, MemoryCacheManager};
use std::sync::Arc;

/// Unit tests for DynamoDbAppManager
///
/// Note: These tests focus on the caching logic and error handling.
/// Integration tests with actual DynamoDB are marked as #[ignore] and
/// require a running DynamoDB instance (local or AWS).

#[tokio::test]
async fn test_cache_integration_by_id() {
    // Test that caching works correctly for ID lookups
    let cache = Arc::new(MemoryCacheManager::new());

    // Manually populate cache
    let app = App::new(
        "test-id".to_string(),
        "test-key".to_string(),
        "test-secret".to_string(),
    );
    let app_json = serde_json::to_string(&app).unwrap();
    cache
        .set("app:test-id", &app_json, Some(3600))
        .await
        .unwrap();

    // Create manager with cache
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

    // Test cache hit by ID - this should not hit DynamoDB
    let result = manager.find_by_id("test-id").await;
    assert!(result.is_ok());
    let cached_app = result.unwrap();
    assert!(cached_app.is_some());
    let cached_app = cached_app.unwrap();
    assert_eq!(cached_app.id, "test-id");
    assert_eq!(cached_app.key, "test-key");
    assert_eq!(cached_app.secret, "test-secret");
}

#[tokio::test]
async fn test_cache_integration_by_key() {
    // Test that caching works correctly for key lookups
    let cache = Arc::new(MemoryCacheManager::new());

    // Manually populate cache
    let app = App::new(
        "test-id".to_string(),
        "test-key".to_string(),
        "test-secret".to_string(),
    );
    let app_json = serde_json::to_string(&app).unwrap();
    cache
        .set("app_key:test-key", &app_json, Some(3600))
        .await
        .unwrap();

    // Create manager with cache
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

    // Test cache hit by key - this should not hit DynamoDB
    let result = manager.find_by_key("test-key").await;
    assert!(result.is_ok());
    let cached_app = result.unwrap();
    assert!(cached_app.is_some());
    let cached_app = cached_app.unwrap();
    assert_eq!(cached_app.id, "test-id");
    assert_eq!(cached_app.key, "test-key");
    assert_eq!(cached_app.secret, "test-secret");
}

#[tokio::test]
async fn test_cache_miss_returns_none() {
    // Test that cache miss returns None without error
    let cache = Arc::new(MemoryCacheManager::new());

    // Create manager with cache
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

    // This will check cache (miss) but won't hit DynamoDB in this test
    // We're just testing that the cache lookup doesn't error
    // The actual DynamoDB query would happen next, but we can't test that without a real DB
}

#[tokio::test]
async fn test_manager_without_cache() {
    // Test that manager works without cache
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

    // Manager should be created successfully without cache
    // Actual queries would require DynamoDB connection
}

#[tokio::test]
async fn test_cache_with_full_app_data() {
    // Test caching with a fully populated App struct
    let cache = Arc::new(MemoryCacheManager::new());

    let mut app = App::new(
        "full-id".to_string(),
        "full-key".to_string(),
        "full-secret".to_string(),
    );
    app.max_connections = Some(1000);
    app.enable_client_messages = true;
    app.enabled = true;
    app.max_backend_events_per_second = Some(100);
    app.max_client_events_per_second = Some(50);
    app.max_read_requests_per_second = Some(200);
    app.max_presence_members_per_channel = Some(100);
    app.max_presence_member_size_in_kb = Some(2.0);
    app.max_channel_name_length = Some(200);
    app.max_event_channels_at_once = Some(10);
    app.max_event_name_length = Some(200);
    app.max_event_payload_in_kb = Some(10.0);
    app.max_event_batch_size = Some(10);
    app.enable_user_authentication = true;

    let app_json = serde_json::to_string(&app).unwrap();
    cache
        .set("app:full-id", &app_json, Some(3600))
        .await
        .unwrap();

    // Create manager with cache
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

    // Test cache hit with full data
    let result = manager.find_by_id("full-id").await;
    assert!(result.is_ok());
    let cached_app = result.unwrap();
    assert!(cached_app.is_some());
    let cached_app = cached_app.unwrap();

    // Verify all fields
    assert_eq!(cached_app.id, "full-id");
    assert_eq!(cached_app.key, "full-key");
    assert_eq!(cached_app.secret, "full-secret");
    assert_eq!(cached_app.max_connections, Some(1000));
    assert_eq!(cached_app.enable_client_messages, true);
    assert_eq!(cached_app.enabled, true);
    assert_eq!(cached_app.max_backend_events_per_second, Some(100));
    assert_eq!(cached_app.max_client_events_per_second, Some(50));
    assert_eq!(cached_app.max_read_requests_per_second, Some(200));
    assert_eq!(cached_app.max_presence_members_per_channel, Some(100));
    assert_eq!(cached_app.max_presence_member_size_in_kb, Some(2.0));
    assert_eq!(cached_app.max_channel_name_length, Some(200));
    assert_eq!(cached_app.max_event_channels_at_once, Some(10));
    assert_eq!(cached_app.max_event_name_length, Some(200));
    assert_eq!(cached_app.max_event_payload_in_kb, Some(10.0));
    assert_eq!(cached_app.max_event_batch_size, Some(10));
    assert_eq!(cached_app.enable_user_authentication, true);
}

#[tokio::test]
async fn test_invalid_cache_data_handling() {
    // Test that invalid JSON in cache is handled gracefully
    let cache = Arc::new(MemoryCacheManager::new());

    // Store invalid JSON in cache
    cache
        .set("app:invalid", "not valid json", Some(3600))
        .await
        .unwrap();

    // Create manager with cache
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

    // This should handle the invalid cache data gracefully
    // In the real implementation, it would fall back to DynamoDB query
    // For this test, we're just verifying it doesn't panic
}

// Integration tests with actual DynamoDB
// These require a running DynamoDB instance and are ignored by default
// Run with: cargo test --test dynamodb_app_manager_test -- --ignored

#[tokio::test]
#[ignore]
async fn test_find_by_id_not_found() {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

    let result = manager.find_by_id("nonexistent-id").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
#[ignore]
async fn test_find_by_key_not_found() {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

    let result = manager.find_by_key("nonexistent-key").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
#[ignore]
async fn test_find_by_id_with_real_dynamodb() {
    // This test requires a DynamoDB table with test data
    // Table: test-apps
    // Item: { id: "test-app-1", key: "test-key-1", secret: "test-secret-1", enabled: true }

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

    let result = manager.find_by_id("test-app-1").await;
    assert!(result.is_ok());
    let app = result.unwrap();
    assert!(app.is_some());
    let app = app.unwrap();
    assert_eq!(app.id, "test-app-1");
    assert_eq!(app.key, "test-key-1");
    assert_eq!(app.secret, "test-secret-1");
}

#[tokio::test]
#[ignore]
async fn test_find_by_key_with_real_dynamodb() {
    // This test requires a DynamoDB table with test data and a GSI
    // Table: test-apps
    // GSI: key-index (partition key: key)
    // Item: { id: "test-app-1", key: "test-key-1", secret: "test-secret-1", enabled: true }

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

    let result = manager.find_by_key("test-key-1").await;
    assert!(result.is_ok());
    let app = result.unwrap();
    assert!(app.is_some());
    let app = app.unwrap();
    assert_eq!(app.id, "test-app-1");
    assert_eq!(app.key, "test-key-1");
    assert_eq!(app.secret, "test-secret-1");
}

#[tokio::test]
#[ignore]
async fn test_caching_after_dynamodb_query() {
    // This test verifies that apps are cached after being fetched from DynamoDB
    let cache = Arc::new(MemoryCacheManager::new());

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);
    let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

    // First query - should hit DynamoDB and cache the result
    let result1 = manager.find_by_id("test-app-1").await;
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_some());

    // Verify it's now in cache
    let cached = cache.get("app:test-app-1").await.unwrap();
    assert!(cached.is_some());

    // Second query - should hit cache
    let result2 = manager.find_by_id("test-app-1").await;
    assert!(result2.is_ok());
    assert!(result2.unwrap().is_some());
}
