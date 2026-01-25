use soketi_rs::app::App;
use soketi_rs::app_managers::{AppManager, MysqlAppManager};
use soketi_rs::cache_managers::{CacheManager, MemoryCacheManager};
use std::sync::Arc;

// Helper function to create test database and table
async fn setup_test_db() -> sqlx::mysql::MySqlPool {
    // Connect to MySQL server (without database)
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:password@localhost")
        .await
        .expect("Failed to connect to MySQL server. Make sure MySQL is running on localhost:3306 with root:password");

    // Create test database
    sqlx::query("CREATE DATABASE IF NOT EXISTS test_pusher_mysql")
        .execute(&pool)
        .await
        .expect("Failed to create test database");

    // Close connection and reconnect to the test database
    pool.close().await;

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:password@localhost/test_pusher_mysql")
        .await
        .expect("Failed to connect to test database");

    // Create apps table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS apps (
            id VARCHAR(255) PRIMARY KEY,
            `key` VARCHAR(255) NOT NULL UNIQUE,
            secret VARCHAR(255) NOT NULL,
            max_connections BIGINT UNSIGNED,
            enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            max_backend_events_per_second BIGINT UNSIGNED,
            max_client_events_per_second BIGINT UNSIGNED,
            max_read_requests_per_second BIGINT UNSIGNED,
            webhooks JSON,
            max_presence_members_per_channel BIGINT UNSIGNED,
            max_presence_member_size_in_kb DOUBLE,
            max_channel_name_length BIGINT UNSIGNED,
            max_event_channels_at_once BIGINT UNSIGNED,
            max_event_name_length BIGINT UNSIGNED,
            max_event_payload_in_kb DOUBLE,
            max_event_batch_size BIGINT UNSIGNED,
            enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE,
            INDEX idx_key (`key`)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create apps table");

    // Clear any existing data
    sqlx::query("DELETE FROM apps")
        .execute(&pool)
        .await
        .expect("Failed to clear apps table");

    pool
}

// Helper function to insert a test app
async fn insert_test_app(pool: &sqlx::mysql::MySqlPool, app: &App) {
    let webhooks_json = serde_json::to_string(&app.webhooks).unwrap();

    sqlx::query(
        "INSERT INTO apps (
            id, `key`, secret, max_connections, enable_client_messages, enabled,
            max_backend_events_per_second, max_client_events_per_second,
            max_read_requests_per_second, webhooks, max_presence_members_per_channel,
            max_presence_member_size_in_kb, max_channel_name_length,
            max_event_channels_at_once, max_event_name_length, max_event_payload_in_kb,
            max_event_batch_size, enable_user_authentication
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&app.id)
    .bind(&app.key)
    .bind(&app.secret)
    .bind(app.max_connections)
    .bind(app.enable_client_messages)
    .bind(app.enabled)
    .bind(app.max_backend_events_per_second)
    .bind(app.max_client_events_per_second)
    .bind(app.max_read_requests_per_second)
    .bind(&webhooks_json)
    .bind(app.max_presence_members_per_channel)
    .bind(app.max_presence_member_size_in_kb)
    .bind(app.max_channel_name_length)
    .bind(app.max_event_channels_at_once)
    .bind(app.max_event_name_length)
    .bind(app.max_event_payload_in_kb)
    .bind(app.max_event_batch_size)
    .bind(app.enable_user_authentication)
    .execute(pool)
    .await
    .expect("Failed to insert test app");
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_find_by_id_success() {
    let pool = setup_test_db().await;

    // Insert test app
    let app = App::new(
        "test-app-1".to_string(),
        "test-key-1".to_string(),
        "test-secret-1".to_string(),
    );
    insert_test_app(&pool, &app).await;

    // Create manager and test lookup
    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    let result = manager.find_by_id("test-app-1").await;
    assert!(result.is_ok());

    let found_app = result.unwrap();
    assert!(found_app.is_some());

    let found_app = found_app.unwrap();
    assert_eq!(found_app.id, "test-app-1");
    assert_eq!(found_app.key, "test-key-1");
    assert_eq!(found_app.secret, "test-secret-1");
    assert_eq!(found_app.enabled, true);

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_find_by_id_not_found() {
    let pool = setup_test_db().await;

    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    let result = manager.find_by_id("nonexistent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_find_by_key_success() {
    let pool = setup_test_db().await;

    // Insert test app
    let app = App::new(
        "test-app-2".to_string(),
        "test-key-2".to_string(),
        "test-secret-2".to_string(),
    );
    insert_test_app(&pool, &app).await;

    // Create manager and test lookup
    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    let result = manager.find_by_key("test-key-2").await;
    assert!(result.is_ok());

    let found_app = result.unwrap();
    assert!(found_app.is_some());

    let found_app = found_app.unwrap();
    assert_eq!(found_app.id, "test-app-2");
    assert_eq!(found_app.key, "test-key-2");
    assert_eq!(found_app.secret, "test-secret-2");

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_find_by_key_not_found() {
    let pool = setup_test_db().await;

    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    let result = manager.find_by_key("nonexistent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_find_with_optional_fields() {
    let pool = setup_test_db().await;

    // Insert app with optional fields
    let mut app = App::new(
        "test-app-3".to_string(),
        "test-key-3".to_string(),
        "test-secret-3".to_string(),
    );
    app.max_connections = Some(100);
    app.enable_client_messages = true;
    app.max_backend_events_per_second = Some(50);
    app.max_presence_members_per_channel = Some(200);
    app.max_presence_member_size_in_kb = Some(2.5);

    insert_test_app(&pool, &app).await;

    // Create manager and test lookup
    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    let result = manager.find_by_id("test-app-3").await;
    assert!(result.is_ok());

    let found_app = result.unwrap().unwrap();
    assert_eq!(found_app.max_connections, Some(100));
    assert_eq!(found_app.enable_client_messages, true);
    assert_eq!(found_app.max_backend_events_per_second, Some(50));
    assert_eq!(found_app.max_presence_members_per_channel, Some(200));
    assert_eq!(found_app.max_presence_member_size_in_kb, Some(2.5));

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_find_disabled_app() {
    let pool = setup_test_db().await;

    // Insert disabled app
    let mut app = App::new(
        "test-app-4".to_string(),
        "test-key-4".to_string(),
        "test-secret-4".to_string(),
    );
    app.enabled = false;

    insert_test_app(&pool, &app).await;

    // Create manager and test lookup
    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    let result = manager.find_by_id("test-app-4").await;
    assert!(result.is_ok());

    let found_app = result.unwrap().unwrap();
    assert_eq!(found_app.enabled, false);

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_caching_by_id() {
    let pool = setup_test_db().await;
    let cache = Arc::new(MemoryCacheManager::new());

    // Insert test app
    let app = App::new(
        "test-app-5".to_string(),
        "test-key-5".to_string(),
        "test-secret-5".to_string(),
    );
    insert_test_app(&pool, &app).await;

    // Create manager with cache
    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), Some(cache.clone()));

    // First lookup - should hit database and cache
    let result1 = manager.find_by_id("test-app-5").await;
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_some());

    // Verify it's in cache
    let cached = cache.get("app:test-app-5").await.unwrap();
    assert!(cached.is_some());

    // Second lookup - should hit cache
    let result2 = manager.find_by_id("test-app-5").await;
    assert!(result2.is_ok());
    assert!(result2.unwrap().is_some());

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_caching_by_key() {
    let pool = setup_test_db().await;
    let cache = Arc::new(MemoryCacheManager::new());

    // Insert test app
    let app = App::new(
        "test-app-6".to_string(),
        "test-key-6".to_string(),
        "test-secret-6".to_string(),
    );
    insert_test_app(&pool, &app).await;

    // Create manager with cache
    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), Some(cache.clone()));

    // First lookup - should hit database and cache
    let result1 = manager.find_by_key("test-key-6").await;
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_some());

    // Verify it's in cache
    let cached = cache.get("app_key:test-key-6").await.unwrap();
    assert!(cached.is_some());

    // Second lookup - should hit cache
    let result2 = manager.find_by_key("test-key-6").await;
    assert!(result2.is_ok());
    assert!(result2.unwrap().is_some());

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_connection_pooling() {
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(3)
        .min_connections(1)
        .connect("mysql://root:password@localhost/test_pusher_mysql")
        .await
        .expect("Failed to connect to test database");

    // Create apps table if needed
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS apps (
            id VARCHAR(255) PRIMARY KEY,
            `key` VARCHAR(255) NOT NULL UNIQUE,
            secret VARCHAR(255) NOT NULL,
            max_connections BIGINT UNSIGNED,
            enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            max_backend_events_per_second BIGINT UNSIGNED,
            max_client_events_per_second BIGINT UNSIGNED,
            max_read_requests_per_second BIGINT UNSIGNED,
            webhooks JSON,
            max_presence_members_per_channel BIGINT UNSIGNED,
            max_presence_member_size_in_kb DOUBLE,
            max_channel_name_length BIGINT UNSIGNED,
            max_event_channels_at_once BIGINT UNSIGNED,
            max_event_name_length BIGINT UNSIGNED,
            max_event_payload_in_kb DOUBLE,
            max_event_batch_size BIGINT UNSIGNED,
            enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE,
            INDEX idx_key (`key`)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create apps table");

    // Clear and insert test apps
    sqlx::query("DELETE FROM apps")
        .execute(&pool)
        .await
        .unwrap();

    for i in 0..5 {
        let app = App::new(
            format!("pool-test-{}", i),
            format!("pool-key-{}", i),
            format!("pool-secret-{}", i),
        );
        insert_test_app(&pool, &app).await;
    }

    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    // Perform multiple concurrent lookups to test connection pooling
    let mut handles = vec![];
    for i in 0..5 {
        let manager_clone = manager.clone();
        let handle =
            tokio::spawn(
                async move { manager_clone.find_by_id(&format!("pool-test-{}", i)).await },
            );
        handles.push(handle);
    }

    // Wait for all lookups to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    pool.close().await;
}

#[tokio::test]
#[ignore] // Requires MySQL server running
async fn test_lookup_consistency() {
    let pool = setup_test_db().await;

    // Insert test app
    let app = App::new(
        "test-app-7".to_string(),
        "test-key-7".to_string(),
        "test-secret-7".to_string(),
    );
    insert_test_app(&pool, &app).await;

    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    // Lookup by ID
    let by_id = manager.find_by_id("test-app-7").await.unwrap().unwrap();

    // Lookup by key
    let by_key = manager.find_by_key("test-key-7").await.unwrap().unwrap();

    // Both should return the same app
    assert_eq!(by_id.id, by_key.id);
    assert_eq!(by_id.key, by_key.key);
    assert_eq!(by_id.secret, by_key.secret);

    pool.close().await;
}
