use crate::adapters::Adapter;
use crate::app_managers::AppManager;
use crate::cache_managers::CacheManager;
use crate::config::ServerConfig;
use crate::metrics::MetricsManager;
use crate::queues::QueueManager;
use crate::rate_limiters::RateLimiter;
use crate::webhook_sender::WebhookSender;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Clone)]
pub struct AppState {
    pub adapter: Arc<dyn Adapter>,
    pub app_manager: Arc<dyn AppManager>,
    pub cache_manager: Arc<dyn CacheManager>,
    pub rate_limiter: Arc<dyn RateLimiter>,
    pub queue_manager: Arc<dyn QueueManager>,
    pub webhook_sender: WebhookSender,
    pub metrics_manager: Option<Arc<dyn MetricsManager>>,
    pub config: ServerConfig,     // Store full config in state
    pub closing: Arc<AtomicBool>, // Flag to indicate server is shutting down
}

// AppState is now created by the Server struct during initialization
// The old new() and init_async() methods have been removed in favor of
// using Server::initialize() which properly initializes all managers
