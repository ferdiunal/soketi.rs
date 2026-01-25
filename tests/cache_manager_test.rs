use soketi_rs::cache_managers::{CacheManager, memory::MemoryCacheManager};
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_cache_set_and_get() {
    let cache = MemoryCacheManager::new();

    // Set a value
    cache.set("test_key", "test_value", None).await.unwrap();

    // Get the value
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_cache_get_nonexistent_key() {
    let cache = MemoryCacheManager::new();

    // Get a non-existent key
    let result = cache.get("nonexistent").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_cache_delete() {
    let cache = MemoryCacheManager::new();

    // Set a value
    cache.set("test_key", "test_value", None).await.unwrap();

    // Verify it exists
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));

    // Delete the value
    cache.delete("test_key").await.unwrap();

    // Verify it's gone
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let cache = MemoryCacheManager::new();

    // Set a value with 1 second TTL
    cache.set("test_key", "test_value", Some(1)).await.unwrap();

    // Immediately get the value - should exist
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));

    // Wait for 2 seconds
    sleep(Duration::from_secs(2)).await;

    // Get the value again - should be expired
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_cache_set_without_ttl() {
    let cache = MemoryCacheManager::new();

    // Set a value without TTL
    cache.set("test_key", "test_value", None).await.unwrap();

    // Wait for 2 seconds
    sleep(Duration::from_secs(2)).await;

    // Value should still exist
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_cache_disconnect() {
    let cache = MemoryCacheManager::new();

    // Set multiple values
    cache.set("key1", "value1", None).await.unwrap();
    cache.set("key2", "value2", None).await.unwrap();
    cache.set("key3", "value3", None).await.unwrap();

    // Verify they exist
    assert_eq!(cache.get("key1").await.unwrap(), Some("value1".to_string()));
    assert_eq!(cache.get("key2").await.unwrap(), Some("value2".to_string()));
    assert_eq!(cache.get("key3").await.unwrap(), Some("value3".to_string()));

    // Disconnect (should clear all entries)
    cache.disconnect().await.unwrap();

    // Verify all entries are gone
    assert_eq!(cache.get("key1").await.unwrap(), None);
    assert_eq!(cache.get("key2").await.unwrap(), None);
    assert_eq!(cache.get("key3").await.unwrap(), None);
}

#[tokio::test]
async fn test_cache_overwrite_value() {
    let cache = MemoryCacheManager::new();

    // Set a value
    cache.set("test_key", "value1", None).await.unwrap();
    assert_eq!(
        cache.get("test_key").await.unwrap(),
        Some("value1".to_string())
    );

    // Overwrite with a new value
    cache.set("test_key", "value2", None).await.unwrap();
    assert_eq!(
        cache.get("test_key").await.unwrap(),
        Some("value2".to_string())
    );
}

#[tokio::test]
async fn test_cache_delete_nonexistent_key() {
    let cache = MemoryCacheManager::new();

    // Delete a non-existent key - should not error
    cache.delete("nonexistent").await.unwrap();
}

// ============================================================================
// RedisCacheManager Tests
// ============================================================================

use soketi_rs::cache_managers::redis::RedisCacheManager;

// Helper function to get Redis URL from environment or use default
fn get_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
}

// Helper function to check if Redis is available
async fn is_redis_available() -> bool {
    match redis::Client::open(get_redis_url().as_str()) {
        Ok(client) => match client.get_multiplexed_async_connection().await {
            Ok(_) => true,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_redis_cache_set_and_get() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

    // Set a value
    cache.set(&test_key, "test_value", None).await.unwrap();

    // Get the value
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));

    // Cleanup
    cache.delete(&test_key).await.unwrap();
}

#[tokio::test]
async fn test_redis_cache_get_nonexistent_key() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("nonexistent_{}", uuid::Uuid::new_v4());

    // Get a non-existent key
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_redis_cache_delete() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

    // Set a value
    cache.set(&test_key, "test_value", None).await.unwrap();

    // Verify it exists
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));

    // Delete the value
    cache.delete(&test_key).await.unwrap();

    // Verify it's gone
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_redis_cache_ttl_expiration() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

    // Set a value with 1 second TTL
    cache.set(&test_key, "test_value", Some(1)).await.unwrap();

    // Immediately get the value - should exist
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));

    // Wait for 2 seconds
    sleep(Duration::from_secs(2)).await;

    // Get the value again - should be expired
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_redis_cache_set_without_ttl() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

    // Set a value without TTL
    cache.set(&test_key, "test_value", None).await.unwrap();

    // Wait for 2 seconds
    sleep(Duration::from_secs(2)).await;

    // Value should still exist
    let result = cache.get(&test_key).await.unwrap();
    assert_eq!(result, Some("test_value".to_string()));

    // Cleanup
    cache.delete(&test_key).await.unwrap();
}

#[tokio::test]
async fn test_redis_cache_disconnect() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key1 = format!("key1_{}", uuid::Uuid::new_v4());
    let test_key2 = format!("key2_{}", uuid::Uuid::new_v4());
    let test_key3 = format!("key3_{}", uuid::Uuid::new_v4());

    // Set multiple values
    cache.set(&test_key1, "value1", None).await.unwrap();
    cache.set(&test_key2, "value2", None).await.unwrap();
    cache.set(&test_key3, "value3", None).await.unwrap();

    // Verify they exist
    assert_eq!(
        cache.get(&test_key1).await.unwrap(),
        Some("value1".to_string())
    );
    assert_eq!(
        cache.get(&test_key2).await.unwrap(),
        Some("value2".to_string())
    );
    assert_eq!(
        cache.get(&test_key3).await.unwrap(),
        Some("value3".to_string())
    );

    // Disconnect (note: unlike MemoryCacheManager, Redis data persists)
    cache.disconnect().await.unwrap();

    // Create a new connection to verify data persists
    let cache2 = RedisCacheManager::new(&get_redis_url()).await.unwrap();

    // Verify entries still exist in Redis
    assert_eq!(
        cache2.get(&test_key1).await.unwrap(),
        Some("value1".to_string())
    );
    assert_eq!(
        cache2.get(&test_key2).await.unwrap(),
        Some("value2".to_string())
    );
    assert_eq!(
        cache2.get(&test_key3).await.unwrap(),
        Some("value3".to_string())
    );

    // Cleanup
    cache2.delete(&test_key1).await.unwrap();
    cache2.delete(&test_key2).await.unwrap();
    cache2.delete(&test_key3).await.unwrap();
}

#[tokio::test]
async fn test_redis_cache_overwrite_value() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

    // Set a value
    cache.set(&test_key, "value1", None).await.unwrap();
    assert_eq!(
        cache.get(&test_key).await.unwrap(),
        Some("value1".to_string())
    );

    // Overwrite with a new value
    cache.set(&test_key, "value2", None).await.unwrap();
    assert_eq!(
        cache.get(&test_key).await.unwrap(),
        Some("value2".to_string())
    );

    // Cleanup
    cache.delete(&test_key).await.unwrap();
}

#[tokio::test]
async fn test_redis_cache_delete_nonexistent_key() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key = format!("nonexistent_{}", uuid::Uuid::new_v4());

    // Delete a non-existent key - should not error
    cache.delete(&test_key).await.unwrap();
}

#[tokio::test]
async fn test_redis_cache_connection_error() {
    // Try to connect to an invalid Redis URL
    let result = RedisCacheManager::new("redis://invalid-host:9999").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_redis_cache_multiple_keys() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
    let test_key1 = format!("test_key1_{}", uuid::Uuid::new_v4());
    let test_key2 = format!("test_key2_{}", uuid::Uuid::new_v4());
    let test_key3 = format!("test_key3_{}", uuid::Uuid::new_v4());

    // Set multiple values
    cache.set(&test_key1, "value1", None).await.unwrap();
    cache.set(&test_key2, "value2", None).await.unwrap();
    cache.set(&test_key3, "value3", None).await.unwrap();

    // Verify all values
    assert_eq!(
        cache.get(&test_key1).await.unwrap(),
        Some("value1".to_string())
    );
    assert_eq!(
        cache.get(&test_key2).await.unwrap(),
        Some("value2".to_string())
    );
    assert_eq!(
        cache.get(&test_key3).await.unwrap(),
        Some("value3".to_string())
    );

    // Cleanup
    cache.delete(&test_key1).await.unwrap();
    cache.delete(&test_key2).await.unwrap();
    cache.delete(&test_key3).await.unwrap();
}
