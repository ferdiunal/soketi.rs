use soketi_rs::cache_managers::{CacheManager, MemoryCacheManager};

#[tokio::main]
async fn main() {
    let cache = MemoryCacheManager::new();

    // Test basic operations
    cache.set("key1", "value1", None).await.unwrap();
    let result = cache.get("key1").await.unwrap();
    println!("Get key1: {:?}", result);

    // Test TTL
    cache.set("key2", "value2", Some(2)).await.unwrap();
    let result = cache.get("key2").await.unwrap();
    println!("Get key2 (before expiry): {:?}", result);

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    let result = cache.get("key2").await.unwrap();
    println!("Get key2 (after expiry): {:?}", result);

    println!("Cache test completed successfully!");
}
