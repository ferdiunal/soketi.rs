use super::AppManager;
use crate::app::App;
use crate::error::Result;
use async_trait::async_trait;

/// ArrayAppManager provides a simple in-memory app manager for static configurations.
///
/// This implementation is suitable for:
/// - Development and testing
/// - Small deployments with a fixed set of applications
/// - Scenarios where apps are defined in configuration files
///
/// The apps are stored in a Vec and lookups are performed using linear search.
/// For production deployments with many apps or dynamic app management,
/// consider using database-backed implementations (DynamoDB, MySQL, PostgreSQL).
#[derive(Clone)]
pub struct ArrayAppManager {
    apps: Vec<App>,
}

impl ArrayAppManager {
    /// Create a new ArrayAppManager with the given apps
    ///
    /// # Arguments
    /// * `apps` - Vector of App configurations to manage
    pub fn new(apps: Vec<App>) -> Self {
        Self { apps }
    }
}

#[async_trait]
impl AppManager for ArrayAppManager {
    async fn find_by_id(&self, id: &str) -> Result<Option<App>> {
        Ok(self.apps.iter().find(|app| app.id == id).cloned())
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<App>> {
        Ok(self.apps.iter().find(|app| app.key == key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_by_id() {
        let app = App::new("id1".to_string(), "key1".to_string(), "secret1".to_string());
        let manager = ArrayAppManager::new(vec![app.clone()]);

        let found = manager.find_by_id("id1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().key, "key1");

        let not_found = manager.find_by_id("id2").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_key() {
        let app = App::new("id1".to_string(), "key1".to_string(), "secret1".to_string());
        let manager = ArrayAppManager::new(vec![app.clone()]);

        let found = manager.find_by_key("key1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "id1");

        let not_found = manager.find_by_key("key2").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_multiple_apps() {
        let app1 = App::new("id1".to_string(), "key1".to_string(), "secret1".to_string());
        let app2 = App::new("id2".to_string(), "key2".to_string(), "secret2".to_string());
        let manager = ArrayAppManager::new(vec![app1, app2]);

        let found1 = manager.find_by_id("id1").await.unwrap();
        assert!(found1.is_some());
        assert_eq!(found1.unwrap().key, "key1");

        let found2 = manager.find_by_key("key2").await.unwrap();
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().id, "id2");
    }

    #[tokio::test]
    async fn test_empty_manager() {
        let manager = ArrayAppManager::new(vec![]);

        let not_found = manager.find_by_id("id1").await.unwrap();
        assert!(not_found.is_none());

        let not_found = manager.find_by_key("key1").await.unwrap();
        assert!(not_found.is_none());
    }
}
