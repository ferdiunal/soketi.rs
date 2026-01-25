use crate::error::Result;
use async_trait::async_trait;

pub mod memory;
pub mod redis;

pub use memory::MemoryCacheManager;
pub use redis::RedisCacheManager;

/// CacheManager trait for caching data with configurable backends.
///
/// Implementations provide different storage backends (memory, Redis, etc.)
/// for caching application data with TTL support.
#[async_trait]
pub trait CacheManager: Send + Sync {
    /// Get a value from the cache by key.
    ///
    /// Returns `Ok(Some(value))` if the key exists and hasn't expired,
    /// `Ok(None)` if the key doesn't exist or has expired,
    /// or an error if the operation fails.
    ///
    /// **Validates: Requirement 6.4**
    async fn get(&self, key: &str) -> Result<Option<String>>;

    /// Set a value in the cache with an optional TTL.
    ///
    /// If `ttl_seconds` is `Some(n)`, the value will expire after n seconds.
    /// If `ttl_seconds` is `None`, the value will not expire (or use a default TTL).
    ///
    /// **Validates: Requirement 6.3**
    async fn set(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()>;

    /// Delete a value from the cache by key.
    ///
    /// Returns `Ok(())` whether the key existed or not.
    ///
    /// **Validates: Requirement 6.5**
    async fn delete(&self, key: &str) -> Result<()>;

    /// Disconnect from the cache backend and clean up resources.
    ///
    /// This should be called during graceful shutdown.
    async fn disconnect(&self) -> Result<()>;
}
