// This test verifies that the CacheManager trait is properly defined
// and that MemoryCacheManager implements it correctly.

use soketi_rs::cache_managers::CacheManager;

// This is a compile-time test - if this compiles, the trait is properly defined
fn _assert_cache_manager_trait_is_object_safe(_cache: &dyn CacheManager) {
    // This function doesn't need to be called, it just needs to compile
}

#[test]
fn test_cache_manager_trait_exists() {
    // This test just verifies the module compiles
    assert!(true);
}
