/// Example demonstrating how to use MysqlAppManager
///
/// This example shows:
/// 1. Setting up a MySQL connection pool
/// 2. Creating a MysqlAppManager with and without caching
/// 3. Looking up apps by ID and key
/// 4. Using connection pooling for efficient database access
///
/// Prerequisites:
/// - MySQL server running on localhost:3306
/// - Database 'pusher' created
/// - Apps table created (see schema in mysql.rs)
/// - At least one app inserted in the table
///
/// Run with: cargo run --example mysql_app_manager_example
use soketi_rs::app::App;
use soketi_rs::app_managers::{AppManager, MysqlAppManager};
use soketi_rs::cache_managers::MemoryCacheManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== MySQL App Manager Example ===\n");

    // Example 1: Basic usage without caching
    println!("Example 1: Basic usage without caching");
    println!("---------------------------------------");

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect("mysql://root:password@localhost/pusher")
        .await?;

    let manager = MysqlAppManager::new(pool.clone(), "apps".to_string(), None);

    // Create and insert a test app
    println!("Creating test app...");
    create_test_app(&pool).await?;

    // Lookup by ID
    println!("Looking up app by ID...");
    match manager.find_by_id("example-app-1").await? {
        Some(app) => {
            println!("✓ Found app by ID:");
            println!("  - ID: {}", app.id);
            println!("  - Key: {}", app.key);
            println!("  - Enabled: {}", app.enabled);
        }
        None => println!("✗ App not found"),
    }

    // Lookup by key
    println!("\nLooking up app by key...");
    match manager.find_by_key("example-key-1").await? {
        Some(app) => {
            println!("✓ Found app by key:");
            println!("  - ID: {}", app.id);
            println!("  - Key: {}", app.key);
            println!("  - Enabled: {}", app.enabled);
        }
        None => println!("✗ App not found"),
    }

    // Example 2: Usage with caching
    println!("\n\nExample 2: Usage with caching");
    println!("------------------------------");

    let cache = Arc::new(MemoryCacheManager::new());
    let manager_with_cache =
        MysqlAppManager::new(pool.clone(), "apps".to_string(), Some(cache.clone()));

    // First lookup - hits database
    println!("First lookup (database hit)...");
    let start = std::time::Instant::now();
    let _ = manager_with_cache.find_by_id("example-app-1").await?;
    let db_time = start.elapsed();
    println!("✓ Lookup completed in {:?}", db_time);

    // Second lookup - hits cache
    println!("\nSecond lookup (cache hit)...");
    let start = std::time::Instant::now();
    let _ = manager_with_cache.find_by_id("example-app-1").await?;
    let cache_time = start.elapsed();
    println!("✓ Lookup completed in {:?}", cache_time);
    println!(
        "  Cache speedup: {:.2}x faster",
        db_time.as_micros() as f64 / cache_time.as_micros() as f64
    );

    // Example 3: Connection pooling
    println!("\n\nExample 3: Connection pooling");
    println!("------------------------------");

    // Create multiple test apps
    for i in 2..=5 {
        create_test_app_with_id(&pool, i).await?;
    }

    println!("Performing 4 concurrent lookups...");
    let start = std::time::Instant::now();

    let mut handles = vec![];
    for i in 2..=5 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            manager_clone
                .find_by_id(&format!("example-app-{}", i))
                .await
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(Some(_))) = handle.await {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("✓ Completed {} lookups in {:?}", success_count, elapsed);
    println!("  Average time per lookup: {:?}", elapsed / success_count);

    // Example 4: Handling missing apps
    println!("\n\nExample 4: Handling missing apps");
    println!("---------------------------------");

    match manager.find_by_id("nonexistent-app").await? {
        Some(_) => println!("✗ Unexpected: Found nonexistent app"),
        None => println!("✓ Correctly returned None for nonexistent app"),
    }

    // Example 5: Disabled apps
    println!("\n\nExample 5: Working with disabled apps");
    println!("--------------------------------------");

    create_disabled_app(&pool).await?;

    match manager.find_by_id("disabled-app").await? {
        Some(app) => {
            println!("✓ Found disabled app:");
            println!("  - ID: {}", app.id);
            println!("  - Enabled: {}", app.enabled);
            println!(
                "  Note: App manager returns disabled apps; validation happens at connection time"
            );
        }
        None => println!("✗ App not found"),
    }

    // Cleanup
    println!("\n\nCleaning up...");
    cleanup_test_apps(&pool).await?;
    pool.close().await;

    println!("✓ Example completed successfully!");

    Ok(())
}

async fn create_test_app(pool: &sqlx::mysql::MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    create_test_app_with_id(pool, 1).await
}

async fn create_test_app_with_id(
    pool: &sqlx::mysql::MySqlPool,
    id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(
        format!("example-app-{}", id),
        format!("example-key-{}", id),
        format!("example-secret-{}", id),
    );

    let webhooks_json = serde_json::to_string(&app.webhooks)?;

    sqlx::query(
        "INSERT INTO apps (
            id, `key`, secret, max_connections, enable_client_messages, enabled,
            max_backend_events_per_second, max_client_events_per_second,
            max_read_requests_per_second, webhooks, max_presence_members_per_channel,
            max_presence_member_size_in_kb, max_channel_name_length,
            max_event_channels_at_once, max_event_name_length, max_event_payload_in_kb,
            max_event_batch_size, enable_user_authentication
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE `key` = VALUES(`key`)",
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
    .await?;

    Ok(())
}

async fn create_disabled_app(
    pool: &sqlx::mysql::MySqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(
        "disabled-app".to_string(),
        "disabled-key".to_string(),
        "disabled-secret".to_string(),
    );
    app.enabled = false;

    let webhooks_json = serde_json::to_string(&app.webhooks)?;

    sqlx::query(
        "INSERT INTO apps (
            id, `key`, secret, max_connections, enable_client_messages, enabled,
            max_backend_events_per_second, max_client_events_per_second,
            max_read_requests_per_second, webhooks, max_presence_members_per_channel,
            max_presence_member_size_in_kb, max_channel_name_length,
            max_event_channels_at_once, max_event_name_length, max_event_payload_in_kb,
            max_event_batch_size, enable_user_authentication
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE enabled = VALUES(enabled)",
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
    .await?;

    Ok(())
}

async fn cleanup_test_apps(
    pool: &sqlx::mysql::MySqlPool,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DELETE FROM apps WHERE id LIKE 'example-app-%' OR id = 'disabled-app'")
        .execute(pool)
        .await?;
    Ok(())
}
