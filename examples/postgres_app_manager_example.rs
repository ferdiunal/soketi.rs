use soketi_rs::app_managers::{AppManager, PostgresAppManager};
use soketi_rs::cache_managers::MemoryCacheManager;
use std::sync::Arc;

/// Example demonstrating how to use PostgresAppManager
///
/// This example shows:
/// 1. Connecting to PostgreSQL with connection pooling
/// 2. Creating a PostgresAppManager with optional caching
/// 3. Looking up apps by ID and key
/// 4. Using connection pooling for concurrent requests
///
/// Prerequisites:
/// - PostgreSQL server running on localhost:5432
/// - Database named "pusher" created
/// - Apps table created (see docs/POSTGRES_SETUP.md)
/// - At least one app inserted in the database
///
/// Run with:
/// cargo run --example postgres_app_manager_example

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("PostgresAppManager Example");
    println!("==========================\n");

    // Example 1: Basic usage without caching
    println!("Example 1: Basic PostgresAppManager without caching");
    println!("---------------------------------------------------");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/pusher")
        .await?;

    let manager = PostgresAppManager::new(pool.clone(), "apps".to_string(), None);

    // Look up an app by ID
    match manager.find_by_id("app-1").await? {
        Some(app) => {
            println!("Found app by ID:");
            println!("  ID: {}", app.id);
            println!("  Key: {}", app.key);
            println!("  Enabled: {}", app.enabled);
            println!("  Client messages: {}", app.enable_client_messages);
        }
        None => println!("App not found by ID"),
    }

    // Look up an app by key
    match manager.find_by_key("app-key-1").await? {
        Some(app) => {
            println!("\nFound app by key:");
            println!("  ID: {}", app.id);
            println!("  Key: {}", app.key);
        }
        None => println!("\nApp not found by key"),
    }

    // Example 2: With caching
    println!("\n\nExample 2: PostgresAppManager with caching");
    println!("-------------------------------------------");

    let cache = Arc::new(MemoryCacheManager::new());
    let manager_with_cache =
        PostgresAppManager::new(pool.clone(), "apps".to_string(), Some(cache.clone()));

    // First lookup - hits database
    println!("First lookup (database hit):");
    let start = std::time::Instant::now();
    let app1 = manager_with_cache.find_by_id("app-1").await?;
    let duration1 = start.elapsed();
    println!("  Time: {:?}", duration1);
    if let Some(app) = app1 {
        println!("  Found: {}", app.key);
    }

    // Second lookup - hits cache
    println!("\nSecond lookup (cache hit):");
    let start = std::time::Instant::now();
    let app2 = manager_with_cache.find_by_id("app-1").await?;
    let duration2 = start.elapsed();
    println!("  Time: {:?}", duration2);
    if let Some(app) = app2 {
        println!("  Found: {}", app.key);
    }
    println!(
        "  Speedup: {:.2}x",
        duration1.as_micros() as f64 / duration2.as_micros() as f64
    );

    // Example 3: Connection pooling with concurrent requests
    println!("\n\nExample 3: Connection pooling with concurrent requests");
    println!("------------------------------------------------------");

    let pool_with_limits = sqlx::postgres::PgPoolOptions::new()
        .min_connections(2)
        .max_connections(5)
        .connect("postgresql://postgres:password@localhost/pusher")
        .await?;

    let manager_pooled = PostgresAppManager::new(pool_with_limits, "apps".to_string(), None);

    println!("Making 10 concurrent requests...");
    let start = std::time::Instant::now();

    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = manager_pooled.clone();
        let handle = tokio::spawn(async move {
            let result = manager_clone.find_by_id("app-1").await;
            (i, result)
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        let (i, result) = handle.await?;
        match result {
            Ok(Some(_)) => {
                success_count += 1;
                println!("  Request {}: Success", i);
            }
            Ok(None) => println!("  Request {}: Not found", i),
            Err(e) => println!("  Request {}: Error - {}", i, e),
        }
    }

    let duration = start.elapsed();
    println!(
        "\nCompleted {} successful requests in {:?}",
        success_count, duration
    );
    println!("Average time per request: {:?}", duration / 10);

    // Example 4: Handling disabled apps
    println!("\n\nExample 4: Handling disabled apps");
    println!("----------------------------------");

    match manager.find_by_id("disabled-app").await? {
        Some(app) => {
            println!("Found app:");
            println!("  ID: {}", app.id);
            println!("  Enabled: {}", app.enabled);
            if !app.enabled {
                println!("  ⚠️  This app is disabled and should not accept connections");
            }
        }
        None => println!("App not found"),
    }

    // Example 5: App with all configuration fields
    println!("\n\nExample 5: App with full configuration");
    println!("---------------------------------------");

    match manager.find_by_id("app-1").await? {
        Some(app) => {
            println!("App configuration:");
            println!("  ID: {}", app.id);
            println!("  Key: {}", app.key);
            println!("  Enabled: {}", app.enabled);
            println!("  Client messages: {}", app.enable_client_messages);
            println!("  User authentication: {}", app.enable_user_authentication);

            if let Some(max_conn) = app.max_connections {
                println!("  Max connections: {}", max_conn);
            }
            if let Some(rate) = app.max_backend_events_per_second {
                println!("  Backend events/sec: {}", rate);
            }
            if let Some(rate) = app.max_client_events_per_second {
                println!("  Client events/sec: {}", rate);
            }
            if let Some(rate) = app.max_read_requests_per_second {
                println!("  Read requests/sec: {}", rate);
            }

            println!("  Webhooks configured: {}", app.webhooks.len());
        }
        None => println!("App not found"),
    }

    println!("\n\nExample completed successfully!");
    Ok(())
}
