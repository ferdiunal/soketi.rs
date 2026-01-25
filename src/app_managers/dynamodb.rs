use super::AppManager;
use crate::app::App;
use crate::cache_managers::CacheManager;
use crate::error::{PusherError, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use std::sync::Arc;

/// DynamoDbAppManager provides a DynamoDB-backed app manager for AWS deployments.
///
/// This implementation is suitable for:
/// - Production deployments on AWS
/// - Dynamic app management with DynamoDB as the source of truth
/// - Scenarios requiring high availability and scalability
///
/// The manager queries apps from a DynamoDB table and supports optional caching
/// to reduce database load and improve performance.
///
/// # DynamoDB Table Schema
///
/// The DynamoDB table should have the following structure:
/// - Primary Key: `id` (String) - The app ID
/// - Global Secondary Index (GSI): `key-index` with partition key `key` (String)
/// - Attributes: All App struct fields serialized as DynamoDB attributes
///
/// # Caching
///
/// When a CacheManager is provided, apps are cached with the configured TTL.
/// Cache keys are prefixed with "app:" for ID lookups and "app_key:" for key lookups.
#[derive(Clone)]
pub struct DynamoDbAppManager {
    client: DynamoDbClient,
    table_name: String,
    cache: Option<Arc<dyn CacheManager>>,
}

impl DynamoDbAppManager {
    /// Create a new DynamoDbAppManager
    ///
    /// # Arguments
    /// * `client` - AWS DynamoDB client
    /// * `table_name` - Name of the DynamoDB table containing apps
    /// * `cache` - Optional cache manager for caching app lookups
    pub fn new(
        client: DynamoDbClient,
        table_name: String,
        cache: Option<Arc<dyn CacheManager>>,
    ) -> Self {
        Self {
            client,
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

    /// Query DynamoDB for an app by ID
    async fn query_by_id(&self, id: &str) -> Result<Option<App>> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key(
                "id",
                aws_sdk_dynamodb::types::AttributeValue::S(id.to_string()),
            )
            .send()
            .await
            .map_err(|e| PusherError::DatabaseError(format!("DynamoDB get_item error: {}", e)))?;

        if let Some(item) = result.item {
            let app = self.parse_dynamodb_item(item)?;
            // Store in cache for future lookups
            self.store_in_cache(&app).await?;
            Ok(Some(app))
        } else {
            Ok(None)
        }
    }

    /// Query DynamoDB for an app by key using GSI
    async fn query_by_key(&self, key: &str) -> Result<Option<App>> {
        let result = self
            .client
            .query()
            .table_name(&self.table_name)
            .index_name("key-index")
            .key_condition_expression("#key = :key")
            .expression_attribute_names("#key", "key")
            .expression_attribute_values(
                ":key",
                aws_sdk_dynamodb::types::AttributeValue::S(key.to_string()),
            )
            .limit(1)
            .send()
            .await
            .map_err(|e| PusherError::DatabaseError(format!("DynamoDB query error: {}", e)))?;

        if let Some(items) = result.items
            && let Some(item) = items.first()
        {
            let app = self.parse_dynamodb_item(item.clone())?;
            // Store in cache for future lookups
            self.store_in_cache(&app).await?;
            return Ok(Some(app));
        }
        Ok(None)
    }

    /// Parse a DynamoDB item into an App struct
    fn parse_dynamodb_item(
        &self,
        item: std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>,
    ) -> Result<App> {
        // Helper function to get string attribute
        let get_string = |key: &str| -> Result<String> {
            item.get(key)
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    PusherError::DatabaseError(format!("Missing or invalid field: {}", key))
                })
        };

        // Helper function to get optional string attribute (currently unused but may be needed for future fields)
        let _get_optional_string = |key: &str| -> Option<String> {
            item.get(key)
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string())
        };

        // Helper function to get optional number attribute as u64
        let get_optional_u64 = |key: &str| -> Option<u64> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|s| s.parse::<u64>().ok())
        };

        // Helper function to get optional number attribute as f64
        let get_optional_f64 = |key: &str| -> Option<f64> {
            item.get(key)
                .and_then(|v| v.as_n().ok())
                .and_then(|s| s.parse::<f64>().ok())
        };

        // Helper function to get boolean attribute
        let get_bool = |key: &str, default: bool| -> bool {
            item.get(key)
                .and_then(|v| v.as_bool().ok())
                .copied()
                .unwrap_or(default)
        };

        // Parse webhooks from JSON string or list
        let webhooks = item
            .get("webhooks")
            .and_then(|v| {
                if let Ok(s) = v.as_s() {
                    serde_json::from_str(s).ok()
                } else {
                    None
                }
            })
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
impl AppManager for DynamoDbAppManager {
    async fn find_by_id(&self, id: &str) -> Result<Option<App>> {
        // Try cache first
        if let Some(app) = self.get_from_cache_by_id(id).await? {
            return Ok(Some(app));
        }

        // Query DynamoDB if not in cache
        self.query_by_id(id).await
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<App>> {
        // Try cache first
        if let Some(app) = self.get_from_cache_by_key(key).await? {
            return Ok(Some(app));
        }

        // Query DynamoDB if not in cache
        self.query_by_key(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_managers::MemoryCacheManager;

    // Note: These tests require a running DynamoDB instance (local or AWS)
    // They are marked as ignored by default and should be run with:
    // cargo test --test dynamodb_app_manager_test -- --ignored

    #[tokio::test]
    #[ignore]
    async fn test_find_by_id_not_found() {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = DynamoDbClient::new(&config);
        let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

        let result = manager.find_by_id("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_key_not_found() {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = DynamoDbClient::new(&config);
        let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), None);

        let result = manager.find_by_key("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_integration() {
        // Test that caching works correctly
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

        // Create manager with cache (no real DynamoDB client needed for cache test)
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = DynamoDbClient::new(&config);
        let manager = DynamoDbAppManager::new(client, "test-apps".to_string(), Some(cache.clone()));

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
