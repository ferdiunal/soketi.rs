use soketi_rs::app_managers::{AppManager, PostgresAppManager};
use soketi_rs::cache_managers::{CacheManager, MemoryCacheManager};
use std::sync::Arc;

// These tests require a running PostgreSQL instance with the test database set up.
// Run with: cargo test --test postgres_app_manager_test -- --ignored
//
// To set up the test database:
// 1. Create a PostgreSQL database: createdb test_pusher
// 2. Run the schema creation script (see docs/POSTGRES_SETUP.md)
// 3. Insert test data

#[tokio::test]
#[ignore]
async fn test_find_by_id_success() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    // Insert test app
    sqlx::query(
        "INSERT INTO apps (id, key, secret, enabled, enable_client_messages, enable_user_authentication) 
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET key = $2, secret = $3"
    )
    .bind("test-app-1")
    .bind("test-key-1")
    .bind("test-secret-1")
    .bind(true)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .expect("Failed to insert test app");

    let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

    let result = manager.find_by_id("test-app-1").await;
    assert!(result.is_ok());
    let app = result.unwrap();
    assert!(app.is_some());
    let app = app.unwrap();
    assert_eq!(app.id, "test-app-1");
    assert_eq!(app.key, "test-key-1");
    assert_eq!(app.secret, "test-secret-1");
    assert!(app.enabled);
}

#[tokio::test]
#[ignore]
async fn test_find_by_key_success() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    // Insert test app
    sqlx::query(
        "INSERT INTO apps (id, key, secret, enabled, enable_client_messages, enable_user_authentication) 
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET key = $2, secret = $3"
    )
    .bind("test-app-2")
    .bind("test-key-2")
    .bind("test-secret-2")
    .bind(true)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .expect("Failed to insert test app");

    let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

    let result = manager.find_by_key("test-key-2").await;
    assert!(result.is_ok());
    let app = result.unwrap();
    assert!(app.is_some());
    let app = app.unwrap();
    assert_eq!(app.id, "test-app-2");
    assert_eq!(app.key, "test-key-2");
    assert_eq!(app.secret, "test-secret-2");
}

#[tokio::test]
#[ignore]
async fn test_find_by_id_not_found() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

    let result = manager.find_by_id("nonexistent-app").await;
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

    let result = manager.find_by_key("nonexistent-key").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
#[ignore]
async fn test_with_cache() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    // Insert test app
    sqlx::query(
        "INSERT INTO apps (id, key, secret, enabled, enable_client_messages, enable_user_authentication) 
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET key = $2, secret = $3"
    )
    .bind("test-app-cached")
    .bind("test-key-cached")
    .bind("test-secret-cached")
    .bind(true)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .expect("Failed to insert test app");

    let cache = Arc::new(MemoryCacheManager::new());
    let manager = PostgresAppManager::new(pool, "apps".to_string(), Some(cache.clone()));

    // First lookup - should hit database and cache the result
    let result1 = manager.find_by_id("test-app-cached").await;
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_some());

    // Second lookup - should hit cache
    let result2 = manager.find_by_id("test-app-cached").await;
    assert!(result2.is_ok());
    let app = result2.unwrap().unwrap();
    assert_eq!(app.id, "test-app-cached");
    assert_eq!(app.key, "test-key-cached");

    // Lookup by key - should also hit cache
    let result3 = manager.find_by_key("test-key-cached").await;
    assert!(result3.is_ok());
    let app = result3.unwrap().unwrap();
    assert_eq!(app.id, "test-app-cached");
}

#[tokio::test]
#[ignore]
async fn test_app_with_all_fields() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    // Insert test app with all fields
    sqlx::query(
        "INSERT INTO apps (
            id, key, secret, enabled, enable_client_messages, enable_user_authentication,
            max_connections, max_backend_events_per_second, max_client_events_per_second,
            max_read_requests_per_second, max_presence_members_per_channel,
            max_presence_member_size_in_kb, max_channel_name_length,
            max_event_channels_at_once, max_event_name_length,
            max_event_payload_in_kb, max_event_batch_size, webhooks
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
         ON CONFLICT (id) DO UPDATE SET 
            key = $2, secret = $3, max_connections = $7",
    )
    .bind("test-app-full")
    .bind("test-key-full")
    .bind("test-secret-full")
    .bind(true)
    .bind(true)
    .bind(true)
    .bind(1000i64)
    .bind(100i64)
    .bind(50i64)
    .bind(200i64)
    .bind(100i64)
    .bind(2.0f64)
    .bind(200i64)
    .bind(10i64)
    .bind(100i64)
    .bind(10.0f64)
    .bind(10i64)
    .bind(serde_json::json!([]))
    .execute(&pool)
    .await
    .expect("Failed to insert test app");

    let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

    let result = manager.find_by_id("test-app-full").await;
    assert!(result.is_ok());
    let app = result.unwrap().unwrap();
    assert_eq!(app.id, "test-app-full");
    assert_eq!(app.max_connections, Some(1000));
    assert_eq!(app.max_backend_events_per_second, Some(100));
    assert_eq!(app.max_client_events_per_second, Some(50));
    assert_eq!(app.max_read_requests_per_second, Some(200));
    assert_eq!(app.max_presence_members_per_channel, Some(100));
    assert_eq!(app.max_presence_member_size_in_kb, Some(2.0));
    assert_eq!(app.max_channel_name_length, Some(200));
    assert_eq!(app.max_event_channels_at_once, Some(10));
    assert_eq!(app.max_event_name_length, Some(100));
    assert_eq!(app.max_event_payload_in_kb, Some(10.0));
    assert_eq!(app.max_event_batch_size, Some(10));
    assert!(app.enable_client_messages);
    assert!(app.enable_user_authentication);
}

#[tokio::test]
#[ignore]
async fn test_disabled_app() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    // Insert disabled app
    sqlx::query(
        "INSERT INTO apps (id, key, secret, enabled, enable_client_messages, enable_user_authentication) 
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET enabled = $4"
    )
    .bind("test-app-disabled")
    .bind("test-key-disabled")
    .bind("test-secret-disabled")
    .bind(false)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .expect("Failed to insert test app");

    let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

    let result = manager.find_by_id("test-app-disabled").await;
    assert!(result.is_ok());
    let app = result.unwrap().unwrap();
    assert_eq!(app.id, "test-app-disabled");
    assert!(!app.enabled);
}

#[tokio::test]
#[ignore]
async fn test_connection_pooling() {
    // Test that connection pooling works correctly
    let pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(2)
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/test_pusher")
        .await
        .expect("Failed to connect to PostgreSQL");

    // Insert test app
    sqlx::query(
        "INSERT INTO apps (id, key, secret, enabled, enable_client_messages, enable_user_authentication) 
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET key = $2"
    )
    .bind("test-app-pool")
    .bind("test-key-pool")
    .bind("test-secret-pool")
    .bind(true)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .expect("Failed to insert test app");

    let manager = PostgresAppManager::new(pool, "apps".to_string(), None);

    // Make multiple concurrent requests to test pooling
    let mut handles = vec![];
    for _ in 0..10 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move { manager_clone.find_by_id("test-app-pool").await });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
