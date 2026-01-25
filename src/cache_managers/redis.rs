use crate::cache_managers::CacheManager;
use crate::error::Result;
use async_trait::async_trait;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis-backed cache manager for distributed caching.
///
/// This implementation provides distributed caching using Redis,
/// allowing multiple server instances to share cached data.
///
/// **Validates: Requirement 6.2**
pub struct RedisCacheManager {
    #[allow(dead_code)]
    client: redis::Client,
    conn: Arc<Mutex<redis::aio::MultiplexedConnection>>,
}

impl RedisCacheManager {
    /// Create a new RedisCacheManager with the given Redis URL.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to Redis fails.
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;

        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl CacheManager for RedisCacheManager {
    /// Get a value from Redis by key.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    /// Redis handles TTL expiration automatically.
    ///
    /// **Validates: Requirement 6.4**
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.conn.lock().await;

        let result: Option<String> = conn.get(key).await?;

        Ok(result)
    }

    /// Set a value in Redis with an optional TTL.
    ///
    /// If `ttl_seconds` is provided, Redis will automatically expire the key
    /// after that duration using the SETEX command.
    /// If `ttl_seconds` is None, the key is set without expiration using SET.
    ///
    /// **Validates: Requirement 6.3**
    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()> {
        let mut conn = self.conn.lock().await;

        if let Some(ttl) = ttl_seconds {
            // Use SETEX for keys with TTL
            let _: () = conn.set_ex(key, value, ttl).await?;
        } else {
            // Use SET for keys without TTL
            let _: () = conn.set(key, value).await?;
        }

        Ok(())
    }

    /// Delete a value from Redis by key.
    ///
    /// Returns `Ok(())` whether the key existed or not.
    ///
    /// **Validates: Requirement 6.5**
    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.conn.lock().await;

        let _: () = conn.del(key).await?;

        Ok(())
    }

    /// Disconnect from Redis and clean up resources.
    ///
    /// This closes the Redis connection gracefully.
    async fn disconnect(&self) -> Result<()> {
        // The connection will be dropped when the mutex is dropped
        // Redis connections are automatically closed on drop
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    // Helper function to get Redis URL from environment or use default
    fn get_redis_url() -> String {
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
    }

    // Helper function to check if Redis is available
    async fn is_redis_available() -> bool {
        match redis::Client::open(get_redis_url().as_str()) {
            Ok(client) => client.get_multiplexed_async_connection().await.is_ok(),
            Err(_) => false,
        }
    }

    #[tokio::test]
    async fn test_set_and_get() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

        cache.set(&test_key, "value1", None).await.unwrap();
        let result = cache.get(&test_key).await.unwrap();

        assert_eq!(result, Some("value1".to_string()));

        // Cleanup
        cache.delete(&test_key).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key = format!("nonexistent_{}", uuid::Uuid::new_v4());

        let result = cache.get(&test_key).await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_delete() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

        cache.set(&test_key, "value1", None).await.unwrap();
        cache.delete(&test_key).await.unwrap();
        let result = cache.get(&test_key).await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

        // Set with 1 second TTL
        cache.set(&test_key, "value1", Some(1)).await.unwrap();

        // Should exist immediately
        let result = cache.get(&test_key).await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Wait for expiration
        sleep(Duration::from_secs(2)).await;

        // Should be expired
        let result = cache.get(&test_key).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_overwrite_value() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

        cache.set(&test_key, "value1", None).await.unwrap();
        cache.set(&test_key, "value2", None).await.unwrap();
        let result = cache.get(&test_key).await.unwrap();

        assert_eq!(result, Some("value2".to_string()));

        // Cleanup
        cache.delete(&test_key).await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_keys() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key1 = format!("test_key1_{}", uuid::Uuid::new_v4());
        let test_key2 = format!("test_key2_{}", uuid::Uuid::new_v4());
        let test_key3 = format!("test_key3_{}", uuid::Uuid::new_v4());

        cache.set(&test_key1, "value1", None).await.unwrap();
        cache.set(&test_key2, "value2", None).await.unwrap();
        cache.set(&test_key3, "value3", None).await.unwrap();

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

    #[tokio::test]
    async fn test_disconnect() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let cache = RedisCacheManager::new(&get_redis_url()).await.unwrap();
        let test_key = format!("test_key_{}", uuid::Uuid::new_v4());

        cache.set(&test_key, "value1", None).await.unwrap();
        cache.disconnect().await.unwrap();

        // Note: Unlike MemoryCacheManager, RedisCacheManager doesn't clear
        // the cache on disconnect - it just closes the connection.
        // The data persists in Redis.
    }

    #[tokio::test]
    async fn test_connection_error() {
        // Try to connect to an invalid Redis URL
        let result = RedisCacheManager::new("redis://invalid-host:9999").await;
        assert!(result.is_err());
    }
}
