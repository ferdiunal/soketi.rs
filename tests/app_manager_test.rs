use soketi_rs::app::App;
use soketi_rs::app_managers::{AppManager, array::ArrayAppManager};

#[tokio::test]
async fn test_array_app_manager_find_by_id() {
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );
    let app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );

    let manager = ArrayAppManager::new(vec![app1.clone(), app2.clone()]);

    // Test finding existing app
    let result = manager.find_by_id("app1").await;
    assert!(result.is_ok());
    let found = result.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().key, "key1");

    // Test finding non-existent app
    let result = manager.find_by_id("app3").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_array_app_manager_find_by_key() {
    let app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );
    let app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );

    let manager = ArrayAppManager::new(vec![app1.clone(), app2.clone()]);

    // Test finding existing app
    let result = manager.find_by_key("key2").await;
    assert!(result.is_ok());
    let found = result.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "app2");

    // Test finding non-existent app
    let result = manager.find_by_key("key3").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_array_app_manager_empty() {
    let manager = ArrayAppManager::new(vec![]);

    let result = manager.find_by_id("app1").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    let result = manager.find_by_key("key1").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_array_app_manager_lookup_consistency() {
    // Property 20: App Lookup Consistency
    // For any app, looking up by ID and looking up by key should return the same app if both match
    let app = App::new(
        "test_id".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    let manager = ArrayAppManager::new(vec![app.clone()]);

    let by_id = manager.find_by_id("test_id").await.unwrap().unwrap();
    let by_key = manager.find_by_key("test_key").await.unwrap().unwrap();

    assert_eq!(by_id.id, by_key.id);
    assert_eq!(by_id.key, by_key.key);
    assert_eq!(by_id.secret, by_key.secret);
}
