/// Example demonstrating how to use DynamoDbAppManager
///
/// This example shows:
/// 1. Creating a DynamoDB client
/// 2. Initializing DynamoDbAppManager with and without caching
/// 3. Looking up apps by ID and key
///
/// To run this example:
/// 1. Set up AWS credentials (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION)
/// 2. Create a DynamoDB table named "apps" with:
///    - Primary key: id (String)
///    - Global Secondary Index: key-index with partition key "key" (String)
/// 3. Add test data to the table
/// 4. Run: cargo run --example dynamodb_app_manager_example
use soketi_rs::app_managers::{AppManager, DynamoDbAppManager};
use soketi_rs::cache_managers::MemoryCacheManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("DynamoDB App Manager Example");
    println!("============================\n");

    // Load AWS configuration from environment
    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    // Example 1: DynamoDbAppManager without caching
    println!("Example 1: Without caching");
    println!("--------------------------");
    let manager_no_cache =
        DynamoDbAppManager::new(dynamodb_client.clone(), "apps".to_string(), None);

    match manager_no_cache.find_by_id("test-app-1").await {
        Ok(Some(app)) => {
            println!("✓ Found app by ID: {} (key: {})", app.id, app.key);
        }
        Ok(None) => {
            println!("✗ App not found by ID");
        }
        Err(e) => {
            println!("✗ Error finding app by ID: {}", e);
        }
    }

    match manager_no_cache.find_by_key("test-key-1").await {
        Ok(Some(app)) => {
            println!("✓ Found app by key: {} (id: {})", app.key, app.id);
        }
        Ok(None) => {
            println!("✗ App not found by key");
        }
        Err(e) => {
            println!("✗ Error finding app by key: {}", e);
        }
    }

    println!();

    // Example 2: DynamoDbAppManager with caching
    println!("Example 2: With caching");
    println!("-----------------------");
    let cache = Arc::new(MemoryCacheManager::new());
    let manager_with_cache = DynamoDbAppManager::new(
        dynamodb_client.clone(),
        "apps".to_string(),
        Some(cache.clone()),
    );

    // First lookup - will query DynamoDB and cache the result
    println!("First lookup (will query DynamoDB):");
    match manager_with_cache.find_by_id("test-app-1").await {
        Ok(Some(app)) => {
            println!("✓ Found app by ID: {} (key: {})", app.id, app.key);
        }
        Ok(None) => {
            println!("✗ App not found by ID");
        }
        Err(e) => {
            println!("✗ Error finding app by ID: {}", e);
        }
    }

    // Second lookup - will use cache
    println!("Second lookup (will use cache):");
    match manager_with_cache.find_by_id("test-app-1").await {
        Ok(Some(app)) => {
            println!(
                "✓ Found app by ID from cache: {} (key: {})",
                app.id, app.key
            );
        }
        Ok(None) => {
            println!("✗ App not found by ID");
        }
        Err(e) => {
            println!("✗ Error finding app by ID: {}", e);
        }
    }

    println!();

    // Example 3: Lookup by key with caching
    println!("Example 3: Lookup by key with caching");
    println!("--------------------------------------");
    match manager_with_cache.find_by_key("test-key-1").await {
        Ok(Some(app)) => {
            println!("✓ Found app by key: {} (id: {})", app.key, app.id);
            println!("  - Enabled: {}", app.enabled);
            println!("  - Max connections: {:?}", app.max_connections);
            println!("  - Client messages: {}", app.enable_client_messages);
        }
        Ok(None) => {
            println!("✗ App not found by key");
        }
        Err(e) => {
            println!("✗ Error finding app by key: {}", e);
        }
    }

    println!();
    println!("Example completed!");

    Ok(())
}
