/// Example demonstrating RedisCacheManager functionality
///
/// This example can be run with: cargo run --example test_redis_cache
///
/// Prerequisites:
/// - Redis server running on localhost:6379 (or set REDIS_URL environment variable)
use soketi_rs::cache_managers::{CacheManager, RedisCacheManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Redis URL from environment or use default
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    println!("Connecting to Redis at: {}", redis_url);

    // Create RedisCacheManager
    let cache = match RedisCacheManager::new(&redis_url).await {
        Ok(cache) => {
            println!("✓ Successfully connected to Redis");
            cache
        }
        Err(e) => {
            eprintln!("✗ Failed to connect to Redis: {}", e);
            eprintln!("  Make sure Redis is running on {}", redis_url);
            return Err(e.into());
        }
    };

    // Test 1: Set and Get
    println!("\n--- Test 1: Set and Get ---");
    cache.set("test_key", "test_value", None).await?;
    println!("✓ Set key 'test_key' = 'test_value'");

    let value = cache.get("test_key").await?;
    println!("✓ Get key 'test_key' = {:?}", value);
    assert_eq!(value, Some("test_value".to_string()));

    // Test 2: Get non-existent key
    println!("\n--- Test 2: Get non-existent key ---");
    let value = cache.get("nonexistent").await?;
    println!("✓ Get key 'nonexistent' = {:?}", value);
    assert_eq!(value, None);

    // Test 3: Delete
    println!("\n--- Test 3: Delete ---");
    cache.delete("test_key").await?;
    println!("✓ Deleted key 'test_key'");

    let value = cache.get("test_key").await?;
    println!("✓ Get key 'test_key' after delete = {:?}", value);
    assert_eq!(value, None);

    // Test 4: TTL expiration
    println!("\n--- Test 4: TTL expiration ---");
    cache.set("ttl_key", "ttl_value", Some(2)).await?;
    println!("✓ Set key 'ttl_key' with 2 second TTL");

    let value = cache.get("ttl_key").await?;
    println!("✓ Get key 'ttl_key' immediately = {:?}", value);
    assert_eq!(value, Some("ttl_value".to_string()));

    println!("  Waiting 3 seconds for TTL to expire...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let value = cache.get("ttl_key").await?;
    println!("✓ Get key 'ttl_key' after expiration = {:?}", value);
    assert_eq!(value, None);

    // Test 5: Multiple keys
    println!("\n--- Test 5: Multiple keys ---");
    cache.set("key1", "value1", None).await?;
    cache.set("key2", "value2", None).await?;
    cache.set("key3", "value3", None).await?;
    println!("✓ Set 3 keys");

    let v1 = cache.get("key1").await?;
    let v2 = cache.get("key2").await?;
    let v3 = cache.get("key3").await?;
    println!("✓ Retrieved all 3 keys: {:?}, {:?}, {:?}", v1, v2, v3);

    // Cleanup
    cache.delete("key1").await?;
    cache.delete("key2").await?;
    cache.delete("key3").await?;
    println!("✓ Cleaned up test keys");

    // Test 6: Disconnect
    println!("\n--- Test 6: Disconnect ---");
    cache.disconnect().await?;
    println!("✓ Disconnected from Redis");

    println!("\n✓ All tests passed!");

    Ok(())
}
