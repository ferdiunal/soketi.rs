// Test helper functions for creating test configurations and state

use soketi_rs::adapters::local::LocalAdapter;
use soketi_rs::app::App;
use soketi_rs::app_managers::array::ArrayAppManager;
use soketi_rs::cache_managers::memory::MemoryCacheManager;
use soketi_rs::config::ServerConfig;
use soketi_rs::queues::sync::SyncQueueManager;
use soketi_rs::rate_limiters::local::LocalRateLimiter;
use soketi_rs::state::AppState;
use soketi_rs::webhook_sender::WebhookSender;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Create a test ServerConfig with sensible defaults
pub fn create_test_config() -> ServerConfig {
    ServerConfig::default()
}

/// Create a test AppState with default configuration
pub fn create_test_state() -> Arc<AppState> {
    create_test_state_with_apps(vec![])
}

/// Create a test AppState with specific apps
pub fn create_test_state_with_apps(apps: Vec<App>) -> Arc<AppState> {
    let config = create_test_config();

    let adapter = Arc::new(LocalAdapter::new());
    let app_manager = Arc::new(ArrayAppManager::new(apps));
    let cache_manager = Arc::new(MemoryCacheManager::new());
    let rate_limiter = Arc::new(LocalRateLimiter::new());
    let webhook_sender = WebhookSender::new();
    let queue_manager = Arc::new(SyncQueueManager::new(Arc::new(webhook_sender.clone())));
    let closing = Arc::new(AtomicBool::new(false));

    Arc::new(AppState {
        adapter,
        app_manager,
        cache_manager,
        rate_limiter,
        queue_manager,
        webhook_sender,
        metrics_manager: None,
        config,
        closing,
    })
}

/// Create a test app with default values
pub fn create_test_app() -> App {
    App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    )
}
