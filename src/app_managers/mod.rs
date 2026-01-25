use crate::app::App;
use crate::error::Result;
use async_trait::async_trait;

pub mod array;
pub mod dynamodb;
pub mod mysql;
pub mod postgres;

pub use array::ArrayAppManager;
pub use dynamodb::DynamoDbAppManager;
pub use mysql::MysqlAppManager;
pub use postgres::PostgresAppManager;

/// AppManager trait defines the interface for managing application configurations and credentials.
///
/// Implementations of this trait are responsible for:
/// - Looking up applications by ID or key
/// - Optionally caching app configurations
/// - Validating app credentials during connection and API requests
///
/// The trait returns `Result<Option<App>>` to distinguish between:
/// - `Ok(Some(app))`: App found successfully
/// - `Ok(None)`: App not found (valid query, no match)
/// - `Err(e)`: Error occurred during lookup (database error, network issue, etc.)
#[async_trait]
pub trait AppManager: Send + Sync {
    /// Find an application by its ID
    ///
    /// # Arguments
    /// * `id` - The application ID to search for
    ///
    /// # Returns
    /// * `Ok(Some(App))` - Application found
    /// * `Ok(None)` - Application not found
    /// * `Err(PusherError)` - Error occurred during lookup
    async fn find_by_id(&self, id: &str) -> Result<Option<App>>;

    /// Find an application by its key
    ///
    /// # Arguments
    /// * `key` - The application key to search for
    ///
    /// # Returns
    /// * `Ok(Some(App))` - Application found
    /// * `Ok(None)` - Application not found
    /// * `Err(PusherError)` - Error occurred during lookup
    async fn find_by_key(&self, key: &str) -> Result<Option<App>>;
}
