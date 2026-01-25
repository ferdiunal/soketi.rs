use super::AppManager;
use crate::app::App;
use crate::cache_managers::CacheManager;
use crate::error::{PusherError, Result};
use async_trait::async_trait;
use sqlx::postgres::PgPool;
use std::sync::Arc;

/// PostgresAppManager provides a PostgreSQL-backed app manager for production deployments.
///
/// This implementation is suitable for:
/// - Production deployments with PostgreSQL database
/// - Dynamic app management with PostgreSQL as the source of truth
/// - Scenarios requiring relational database features
///
/// The manager queries apps from a PostgreSQL table and supports optional caching
/// to reduce database load and improve performance. It also supports connection
/// pooling for efficient database resource management.
///
/// # PostgreSQL Table Schema
///
/// The PostgreSQL table should have the following structure:
/// ```sql
/// CREATE TABLE apps (
///     id VARCHAR(255) PRIMARY KEY,
///     key VARCHAR(255) NOT NULL UNIQUE,
///     secret VARCHAR(255) NOT NULL,
///     max_connections BIGINT,
///     enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
///     enabled BOOLEAN NOT NULL DEFAULT TRUE,
///     max_backend_events_per_second BIGINT,
///     max_client_events_per_second BIGINT,
///     max_read_requests_per_second BIGINT,
///     webhooks JSONB,
///     max_presence_members_per_channel BIGINT,
///     max_presence_member_size_in_kb DOUBLE PRECISION,
///     max_channel_name_length BIGINT,
///     max_event_channels_at_once BIGINT,
///     max_event_name_length BIGINT,
///     max_event_payload_in_kb DOUBLE PRECISION,
///     max_event_batch_size BIGINT,
///     enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE
/// );
///
/// CREATE INDEX idx_apps_key ON apps(key);
/// ```
///
/// # Caching
///
/// When a CacheManager is provided, apps are cached with the configured TTL (default 3600 seconds).
/// Cache keys are prefixed with "app:" for ID lookups and "app_key:" for key lookups.
///
/// # Connection Pooling
///
/// The manager uses sqlx's connection pooling to efficiently manage database connections.
/// Pool configuration (min/max connections, timeouts, etc.) should be set when creating
/// the PgPool before passing it to this manager.
#[derive(Clone)]
pub struct PostgresAppManager {
    pool: PgPool,
    table_name: String,
    cache: Option<Arc<dyn CacheManager>>,
}

impl PostgresAppManager {
    /// Create a new PostgresAppManager
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    /// * `table_name` - Name of the PostgreSQL table containing apps (default: "apps")
    /// * `cache` - Optional cache manager for caching app lookups
    ///
    /// # Example
    /// ```no_run
    /// use sqlx::postgres::PgPoolOptions;
    /// use soketi_rs::app_managers::PostgresAppManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let pool = PgPoolOptions::new()
    ///     .max_connections(5)
    ///     .connect("postgresql://user:pass@localhost/pusher").await?;
    ///
    /// let manager = PostgresAppManager::new(
    ///     pool,
    ///     "apps".to_string(),
    ///     None,
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(pool: PgPool, table_name: String, cache: Option<Arc<dyn CacheManager>>) -> Self {
        Self {
            pool,
            table_name,
            cache,
        }
    }

    /// Get an app from cache by ID
    async fn get_from_cache_by_id(&self, id: &str) -> Result<Option<App>> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("app:{}", id);
            if let Some(cached_value) = cache.get(&cache_key).await? {
                match serde_json::from_str::<App>(&cached_value) {
                    Ok(app) => return Ok(Some(app)),
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached app: {}", e);
                        // Continue to database lookup if cache deserialization fails
                    }
                }
            }
        }
        Ok(None)
    }

    /// Get an app from cache by key
    async fn get_from_cache_by_key(&self, key: &str) -> Result<Option<App>> {
        if let Some(cache) = &self.cache {
            let cache_key = format!("app_key:{}", key);
            if let Some(cached_value) = cache.get(&cache_key).await? {
                match serde_json::from_str::<App>(&cached_value) {
                    Ok(app) => return Ok(Some(app)),
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached app: {}", e);
                        // Continue to database lookup if cache deserialization fails
                    }
                }
            }
        }
        Ok(None)
    }

    /// Store an app in cache
    async fn store_in_cache(&self, app: &App) -> Result<()> {
        if let Some(cache) = &self.cache {
            let app_json = serde_json::to_string(app)
                .map_err(|e| PusherError::SerializationError(e.to_string()))?;

            // Cache by ID
            let cache_key_id = format!("app:{}", app.id);
            cache.set(&cache_key_id, &app_json, Some(3600)).await?;

            // Cache by key
            let cache_key_key = format!("app_key:{}", app.key);
            cache.set(&cache_key_key, &app_json, Some(3600)).await?;
        }
        Ok(())
    }

    /// Query PostgreSQL for an app by ID
    async fn query_by_id(&self, id: &str) -> Result<Option<App>> {
        let query = format!(
            "SELECT id, key, secret, max_connections, enable_client_messages, enabled, \
             max_backend_events_per_second, max_client_events_per_second, \
             max_read_requests_per_second, webhooks, max_presence_members_per_channel, \
             max_presence_member_size_in_kb, max_channel_name_length, \
             max_event_channels_at_once, max_event_name_length, max_event_payload_in_kb, \
             max_event_batch_size, enable_user_authentication \
             FROM {} WHERE id = $1",
            self.table_name
        );

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| PusherError::DatabaseError(format!("PostgreSQL query error: {}", e)))?;

        if let Some(row) = row {
            let app = self.parse_postgres_row(row)?;
            // Store in cache for future lookups
            self.store_in_cache(&app).await?;
            Ok(Some(app))
        } else {
            Ok(None)
        }
    }

    /// Query PostgreSQL for an app by key
    async fn query_by_key(&self, key: &str) -> Result<Option<App>> {
        let query = format!(
            "SELECT id, key, secret, max_connections, enable_client_messages, enabled, \
             max_backend_events_per_second, max_client_events_per_second, \
             max_read_requests_per_second, webhooks, max_presence_members_per_channel, \
             max_presence_member_size_in_kb, max_channel_name_length, \
             max_event_channels_at_once, max_event_name_length, max_event_payload_in_kb, \
             max_event_batch_size, enable_user_authentication \
             FROM {} WHERE key = $1",
            self.table_name
        );

        let row = sqlx::query(&query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| PusherError::DatabaseError(format!("PostgreSQL query error: {}", e)))?;

        if let Some(row) = row {
            let app = self.parse_postgres_row(row)?;
            // Store in cache for future lookups
            self.store_in_cache(&app).await?;
            Ok(Some(app))
        } else {
            Ok(None)
        }
    }

    /// Parse a PostgreSQL row into an App struct
    fn parse_postgres_row(&self, row: sqlx::postgres::PgRow) -> Result<App> {
        use sqlx::Row;

        // Helper to get string field
        let get_string = |key: &str| -> Result<String> {
            row.try_get(key).map_err(|e| {
                PusherError::DatabaseError(format!("Missing or invalid field {}: {}", key, e))
            })
        };

        // Helper to get optional i64 field and convert to u64
        let get_optional_u64 = |key: &str| -> Option<u64> {
            row.try_get::<Option<i64>, _>(key)
                .ok()
                .flatten()
                .and_then(|v| if v >= 0 { Some(v as u64) } else { None })
        };

        // Helper to get optional f64 field
        let get_optional_f64 =
            |key: &str| -> Option<f64> { row.try_get::<Option<f64>, _>(key).ok().flatten() };

        // Helper to get bool field with default
        let get_bool =
            |key: &str, default: bool| -> bool { row.try_get::<bool, _>(key).unwrap_or(default) };

        // Parse webhooks from JSONB column
        let webhooks = row
            .try_get::<Option<serde_json::Value>, _>("webhooks")
            .ok()
            .flatten()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(App {
            id: get_string("id")?,
            key: get_string("key")?,
            secret: get_string("secret")?,
            max_connections: get_optional_u64("max_connections"),
            enable_client_messages: get_bool("enable_client_messages", false),
            enabled: get_bool("enabled", true),
            max_backend_events_per_second: get_optional_u64("max_backend_events_per_second"),
            max_client_events_per_second: get_optional_u64("max_client_events_per_second"),
            max_read_requests_per_second: get_optional_u64("max_read_requests_per_second"),
            webhooks,
            max_presence_members_per_channel: get_optional_u64("max_presence_members_per_channel"),
            max_presence_member_size_in_kb: get_optional_f64("max_presence_member_size_in_kb"),
            max_channel_name_length: get_optional_u64("max_channel_name_length"),
            max_event_channels_at_once: get_optional_u64("max_event_channels_at_once"),
            max_event_name_length: get_optional_u64("max_event_name_length"),
            max_event_payload_in_kb: get_optional_f64("max_event_payload_in_kb"),
            max_event_batch_size: get_optional_u64("max_event_batch_size"),
            enable_user_authentication: get_bool("enable_user_authentication", false),
        })
    }
}

#[async_trait]
impl AppManager for PostgresAppManager {
    async fn find_by_id(&self, id: &str) -> Result<Option<App>> {
        // Try cache first
        if let Some(app) = self.get_from_cache_by_id(id).await? {
            return Ok(Some(app));
        }

        // Query PostgreSQL if not in cache
        self.query_by_id(id).await
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<App>> {
        // Try cache first
        if let Some(app) = self.get_from_cache_by_key(key).await? {
            return Ok(Some(app));
        }

        // Query PostgreSQL if not in cache
        self.query_by_key(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_managers::MemoryCacheManager;

    // Note: These tests require a running PostgreSQL instance
    // They are marked as ignored by default and should be run with:
    // cargo test --test postgres_app_manager_test -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_find_by_id_not_found() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect("postgresql://postgres:password@localhost/test_pusher")
            .await
            .expect("Failed to connect to PostgreSQL");

        let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

        let result = manager.find_by_id("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_key_not_found() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect("postgresql://postgres:password@localhost/test_pusher")
            .await
            .expect("Failed to connect to PostgreSQL");

        let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

        let result = manager.find_by_key("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_cache_integration() {
        // Test that caching works correctly without requiring a real database
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
        cache
            .set("app_key:test-key", &app_json, Some(3600))
            .await
            .unwrap();

        // Create a dummy pool (won't be used for cache test)
        // We use a connection string that will fail, but we won't actually query the database
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://localhost/nonexistent")
            .await
            .unwrap_or_else(|_| {
                // If connection fails, we'll skip the test
                panic!("Skipping test - no PostgreSQL available");
            });

        let manager = PostgresAppManager::new(pool, "apps".to_string(), Some(cache.clone()));

        // Test cache hit by ID
        let cached_by_id = manager.get_from_cache_by_id("test-id").await.unwrap();
        assert!(cached_by_id.is_some());
        assert_eq!(cached_by_id.unwrap().key, "test-key");

        // Test cache hit by key
        let cached_by_key = manager.get_from_cache_by_key("test-key").await.unwrap();
        assert!(cached_by_key.is_some());
        assert_eq!(cached_by_key.unwrap().id, "test-id");

        // Test cache miss
        let not_cached = manager.get_from_cache_by_id("not-cached").await.unwrap();
        assert!(not_cached.is_none());
    }
}
