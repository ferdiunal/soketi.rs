use crate::cache_managers::CacheManager;
use crate::error::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Entry stored in the memory cache with value and optional expiration time.
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    /// Check if this cache entry has expired.
    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() >= expires_at
        } else {
            false
        }
    }
}

/// In-memory cache manager using DashMap for concurrent access.
///
/// This implementation provides thread-safe caching with TTL support
/// and automatic background cleanup of expired entries.
///
/// **Validates: Requirement 6.1**
pub struct MemoryCacheManager {
    cache: Arc<DashMap<String, CacheEntry>>,
    cleanup_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl MemoryCacheManager {
    /// Create a new MemoryCacheManager with background cleanup.
    ///
    /// The cleanup task runs every 60 seconds to remove expired entries.
    pub fn new() -> Self {
        let cache = Arc::new(DashMap::new());
        let cleanup_task = Arc::new(RwLock::new(None));

        let manager = Self {
            cache: cache.clone(),
            cleanup_task: cleanup_task.clone(),
        };

        // Start background cleanup task
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_expired_entries(&cache_clone);
            }
        });

        // Store the cleanup task handle
        tokio::spawn(async move {
            let mut task = cleanup_task.write().await;
            *task = Some(handle);
        });

        manager
    }

    /// Remove all expired entries from the cache.
    fn cleanup_expired_entries(cache: &DashMap<String, CacheEntry>) {
        cache.retain(|_, entry| !entry.is_expired());
    }
}

impl Default for MemoryCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheManager for MemoryCacheManager {
    /// Get a value from the cache by key.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    /// Expired entries are removed when accessed.
    ///
    /// **Validates: Requirement 6.4**
    async fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(entry) = self.cache.get(key) {
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.cache.remove(key);
                Ok(None)
            } else {
                Ok(Some(entry.value.clone()))
            }
        } else {
            Ok(None)
        }
    }

    /// Set a value in the cache with an optional TTL.
    ///
    /// If `ttl_seconds` is provided, the entry will expire after that duration.
    ///
    /// **Validates: Requirement 6.3**
    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()> {
        let expires_at = ttl_seconds.map(|ttl| Instant::now() + Duration::from_secs(ttl));

        let entry = CacheEntry {
            value: value.to_string(),
            expires_at,
        };

        self.cache.insert(key.to_string(), entry);
        Ok(())
    }

    /// Delete a value from the cache by key.
    ///
    /// Returns `Ok(())` whether the key existed or not.
    ///
    /// **Validates: Requirement 6.5**
    async fn delete(&self, key: &str) -> Result<()> {
        self.cache.remove(key);
        Ok(())
    }

    /// Disconnect from the cache and clean up resources.
    ///
    /// This stops the background cleanup task and clears all cache entries.
    async fn disconnect(&self) -> Result<()> {
        let mut task = self.cleanup_task.write().await;
        if let Some(handle) = task.take() {
            handle.abort();
        }
        // Clear all cache entries
        self.cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn test_set_and_get() {
        let cache = MemoryCacheManager::new();

        cache.set("key1", "value1", None).await.unwrap();
        let result = cache.get("key1").await.unwrap();

        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let cache = MemoryCacheManager::new();

        let result = cache.get("nonexistent").await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_delete() {
        let cache = MemoryCacheManager::new();

        cache.set("key1", "value1", None).await.unwrap();
        cache.delete("key1").await.unwrap();
        let result = cache.get("key1").await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = MemoryCacheManager::new();

        // Set with 1 second TTL
        cache.set("key1", "value1", Some(1)).await.unwrap();

        // Should exist immediately
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        // Wait for expiration
        sleep(Duration::from_secs(2)).await;

        // Should be expired
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_overwrite_value() {
        let cache = MemoryCacheManager::new();

        cache.set("key1", "value1", None).await.unwrap();
        cache.set("key1", "value2", None).await.unwrap();
        let result = cache.get("key1").await.unwrap();

        assert_eq!(result, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_keys() {
        let cache = MemoryCacheManager::new();

        cache.set("key1", "value1", None).await.unwrap();
        cache.set("key2", "value2", None).await.unwrap();
        cache.set("key3", "value3", None).await.unwrap();

        assert_eq!(cache.get("key1").await.unwrap(), Some("value1".to_string()));
        assert_eq!(cache.get("key2").await.unwrap(), Some("value2".to_string()));
        assert_eq!(cache.get("key3").await.unwrap(), Some("value3".to_string()));
    }

    #[tokio::test]
    async fn test_disconnect() {
        let cache = MemoryCacheManager::new();

        cache.set("key1", "value1", None).await.unwrap();
        cache.disconnect().await.unwrap();

        // Cache should be cleared after disconnect
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, None);
    }
}
