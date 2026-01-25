use crate::adapters::Adapter;
use crate::adapters::cluster::ClusterAdapter;
use crate::adapters::local::LocalAdapter;
use crate::adapters::nats::NatsAdapter;
use crate::adapters::redis::RedisAdapter;
use crate::app_managers::AppManager;
use crate::app_managers::array::ArrayAppManager;
use crate::app_managers::dynamodb::DynamoDbAppManager;
use crate::app_managers::mysql::MysqlAppManager;
use crate::app_managers::postgres::PostgresAppManager;
use crate::cache_managers::CacheManager;
use crate::cache_managers::memory::MemoryCacheManager;
use crate::cache_managers::redis::RedisCacheManager;
use crate::config::{
    AdapterDriver, AppManagerDriver, CacheDriver, MetricsDriver, QueueDriver, RateLimiterDriver,
    ServerConfig, SslConfig,
};
use crate::error::{PusherError, Result};
use crate::metrics::MetricsManager;
use crate::metrics::prometheus::PrometheusMetricsManager;
use crate::queues::QueueManager;
use crate::queues::redis::RedisQueueManager;
use crate::queues::sqs::SqsQueueManager;
use crate::queues::sync::SyncQueueManager;
use crate::rate_limiters::RateLimiter;
use crate::rate_limiters::cluster::ClusterRateLimiter;
use crate::rate_limiters::local::LocalRateLimiter;
use crate::rate_limiters::redis::RedisRateLimiter;
use crate::state::AppState;
use crate::webhook_sender::WebhookSender;
use crate::ws_handler::WsHandler;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Main server struct that initializes and manages all components
///
/// The Server is responsible for:
/// - Loading configuration from ServerConfig
/// - Initializing all managers (Adapter, AppManager, CacheManager, RateLimiter, QueueManager, MetricsManager, WebhookSender)
/// - Creating AppState with all initialized managers
///
/// **Validates: Requirements 1.1, 1.2**
pub struct Server {
    config: ServerConfig,
    state: Option<Arc<AppState>>,
}

impl Server {
    /// Create a new Server instance with the given configuration
    ///
    /// This constructor only stores the configuration. Call `initialize()` to
    /// actually initialize all managers and create the AppState.
    ///
    /// # Arguments
    /// * `config` - Server configuration
    ///
    /// # Returns
    /// * `Server` - A new Server instance
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: None,
        }
    }

    /// Initialize all managers and create AppState
    ///
    /// This method:
    /// 1. Initializes the Adapter based on configuration
    /// 2. Initializes the AppManager based on configuration
    /// 3. Initializes the CacheManager based on configuration
    /// 4. Initializes the RateLimiter based on configuration
    /// 5. Initializes the QueueManager based on configuration
    /// 6. Initializes the MetricsManager if enabled
    /// 7. Initializes the WebhookSender
    /// 8. Creates AppState with all managers
    ///
    /// **Validates: Requirements 1.1, 1.2**
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing server components...");

        // Initialize Adapter
        let adapter = self.initialize_adapter().await?;
        tracing::info!("Adapter initialized: {:?}", self.config.adapter.driver);

        // Initialize CacheManager (needed for AppManager)
        let cache_manager = self.initialize_cache_manager().await?;
        tracing::info!("Cache manager initialized: {:?}", self.config.cache.driver);

        // Initialize AppManager
        let app_manager = self.initialize_app_manager(cache_manager.clone()).await?;
        tracing::info!(
            "App manager initialized: {:?}",
            self.config.app_manager.driver
        );

        // Initialize RateLimiter
        let rate_limiter = self.initialize_rate_limiter().await?;
        tracing::info!(
            "Rate limiter initialized: {:?}",
            self.config.rate_limiter.driver
        );

        // Initialize WebhookSender
        let webhook_sender = self.initialize_webhook_sender();
        tracing::info!("Webhook sender initialized");

        // Initialize QueueManager
        let queue_manager = self
            .initialize_queue_manager(webhook_sender.clone())
            .await?;
        tracing::info!("Queue manager initialized: {:?}", self.config.queue.driver);

        // Initialize MetricsManager if enabled
        let metrics_manager = if self.config.metrics.enabled {
            Some(self.initialize_metrics_manager()?)
        } else {
            None
        };
        if metrics_manager.is_some() {
            tracing::info!("Metrics manager initialized");
        }

        // Create closing flag
        let closing = Arc::new(AtomicBool::new(false));

        // Create AppState
        let state = AppState {
            adapter,
            app_manager,
            cache_manager,
            rate_limiter,
            queue_manager,
            webhook_sender,
            metrics_manager,
            config: self.config.clone(),
            closing,
        };

        self.state = Some(Arc::new(state));

        tracing::info!("Server initialization complete");
        Ok(())
    }

    /// Get the initialized AppState
    ///
    /// Returns None if `initialize()` hasn't been called yet.
    pub fn state(&self) -> Option<Arc<AppState>> {
        self.state.clone()
    }

    /// Initialize the Adapter based on configuration
    async fn initialize_adapter(&self) -> Result<Arc<dyn Adapter>> {
        let adapter: Arc<dyn Adapter> = match self.config.adapter.driver {
            AdapterDriver::Local => Arc::new(LocalAdapter::new()),
            AdapterDriver::Cluster => {
                let cluster_adapter =
                    ClusterAdapter::new(self.config.adapter.cluster.clone()).await?;
                Arc::new(cluster_adapter)
            }
            AdapterDriver::Redis => {
                let redis_adapter = RedisAdapter::new(self.config.adapter.redis.clone()).await?;
                Arc::new(redis_adapter)
            }
            AdapterDriver::Nats => {
                let nats_adapter = NatsAdapter::new(self.config.adapter.nats.clone()).await?;
                Arc::new(nats_adapter)
            }
        };

        // Initialize the adapter
        adapter.init().await?;

        Ok(adapter)
    }

    /// Initialize the AppManager based on configuration
    async fn initialize_app_manager(
        &self,
        cache_manager: Arc<dyn CacheManager>,
    ) -> Result<Arc<dyn AppManager>> {
        let cache = if self.config.app_manager.cache.enabled {
            Some(cache_manager)
        } else {
            None
        };

        let app_manager: Arc<dyn AppManager> = match self.config.app_manager.driver {
            AppManagerDriver::Array => {
                tracing::debug!("Loading ArrayAppManager with {} apps", self.config.app_manager.array.apps.len());
                for app in &self.config.app_manager.array.apps {
                    tracing::debug!("App: id='{}', key='{}', enabled={}", app.id, app.key, app.enabled);
                }
                Arc::new(ArrayAppManager::new(
                    self.config.app_manager.array.apps.clone(),
                ))
            },
            AppManagerDriver::DynamoDb => {
                // Create DynamoDB client
                let aws_config = if let Some(endpoint) = &self.config.app_manager.dynamodb.endpoint
                {
                    aws_config::defaults(aws_config::BehaviorVersion::latest())
                        .region(aws_sdk_dynamodb::config::Region::new(
                            self.config.app_manager.dynamodb.region.clone(),
                        ))
                        .endpoint_url(endpoint)
                        .load()
                        .await
                } else {
                    aws_config::defaults(aws_config::BehaviorVersion::latest())
                        .region(aws_sdk_dynamodb::config::Region::new(
                            self.config.app_manager.dynamodb.region.clone(),
                        ))
                        .load()
                        .await
                };
                let client = aws_sdk_dynamodb::Client::new(&aws_config);

                let dynamodb_manager = DynamoDbAppManager::new(
                    client,
                    self.config.app_manager.dynamodb.table.clone(),
                    cache,
                );
                Arc::new(dynamodb_manager)
            }
            AppManagerDriver::Mysql => {
                // Build MySQL connection URL
                let mysql_url = format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.config.app_manager.mysql.user,
                    self.config.app_manager.mysql.password,
                    self.config.app_manager.mysql.host,
                    self.config.app_manager.mysql.port,
                    self.config.app_manager.mysql.database
                );

                // Create connection pool
                let pool = sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(10)
                    .connect(&mysql_url)
                    .await
                    .map_err(|e| PusherError::DatabaseError(e.to_string()))?;

                let mysql_manager =
                    MysqlAppManager::new(pool, self.config.app_manager.mysql.table.clone(), cache);
                Arc::new(mysql_manager)
            }
            AppManagerDriver::Postgres => {
                // Build PostgreSQL connection URL
                let postgres_url = format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    self.config.app_manager.postgres.user,
                    self.config.app_manager.postgres.password,
                    self.config.app_manager.postgres.host,
                    self.config.app_manager.postgres.port,
                    self.config.app_manager.postgres.database
                );

                // Create connection pool
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(10)
                    .connect(&postgres_url)
                    .await
                    .map_err(|e| PusherError::DatabaseError(e.to_string()))?;

                let postgres_manager = PostgresAppManager::new(
                    pool,
                    self.config.app_manager.postgres.table.clone(),
                    cache,
                );
                Arc::new(postgres_manager)
            }
        };

        Ok(app_manager)
    }

    /// Initialize the CacheManager based on configuration
    async fn initialize_cache_manager(&self) -> Result<Arc<dyn CacheManager>> {
        let cache_manager: Arc<dyn CacheManager> = match self.config.cache.driver {
            CacheDriver::Memory => Arc::new(MemoryCacheManager::new()),
            CacheDriver::Redis => {
                // Build Redis connection URL
                let redis_url = if let Some(password) = &self.config.cache.redis.password {
                    format!(
                        "redis://{}:{}@{}:{}/{}",
                        self.config.cache.redis.username.as_deref().unwrap_or(""),
                        password,
                        self.config.cache.redis.host,
                        self.config.cache.redis.port,
                        self.config.cache.redis.db
                    )
                } else {
                    format!(
                        "redis://{}:{}/{}",
                        self.config.cache.redis.host,
                        self.config.cache.redis.port,
                        self.config.cache.redis.db
                    )
                };

                let redis_cache = RedisCacheManager::new(&redis_url).await?;
                Arc::new(redis_cache)
            }
        };

        Ok(cache_manager)
    }

    /// Initialize the RateLimiter based on configuration
    async fn initialize_rate_limiter(&self) -> Result<Arc<dyn RateLimiter>> {
        let rate_limiter: Arc<dyn RateLimiter> = match self.config.rate_limiter.driver {
            RateLimiterDriver::Local => Arc::new(LocalRateLimiter::new()),
            RateLimiterDriver::Cluster => Arc::new(ClusterRateLimiter::new(false)),
            RateLimiterDriver::Redis => {
                // Build Redis connection URL
                let redis_url = if let Some(password) = &self.config.rate_limiter.redis.password {
                    format!(
                        "redis://{}:{}@{}:{}/{}",
                        self.config
                            .rate_limiter
                            .redis
                            .username
                            .as_deref()
                            .unwrap_or(""),
                        password,
                        self.config.rate_limiter.redis.host,
                        self.config.rate_limiter.redis.port,
                        self.config.rate_limiter.redis.db
                    )
                } else {
                    format!(
                        "redis://{}:{}/{}",
                        self.config.rate_limiter.redis.host,
                        self.config.rate_limiter.redis.port,
                        self.config.rate_limiter.redis.db
                    )
                };

                let redis_limiter = RedisRateLimiter::new(&redis_url).await?;
                Arc::new(redis_limiter)
            }
        };

        Ok(rate_limiter)
    }

    /// Initialize the QueueManager based on configuration
    async fn initialize_queue_manager(
        &self,
        webhook_sender: WebhookSender,
    ) -> Result<Arc<dyn QueueManager>> {
        let queue_manager: Arc<dyn QueueManager> = match self.config.queue.driver {
            QueueDriver::Sync => Arc::new(SyncQueueManager::new(Arc::new(webhook_sender))),
            QueueDriver::Redis => {
                // Build Redis connection URL
                let redis_url = if let Some(password) = &self.config.queue.redis.password {
                    format!(
                        "redis://{}:{}@{}:{}/{}",
                        self.config.queue.redis.username.as_deref().unwrap_or(""),
                        password,
                        self.config.queue.redis.host,
                        self.config.queue.redis.port,
                        self.config.queue.redis.db
                    )
                } else {
                    format!(
                        "redis://{}:{}/{}",
                        self.config.queue.redis.host,
                        self.config.queue.redis.port,
                        self.config.queue.redis.db
                    )
                };

                // Create RedisQueueConfig from ServerConfig
                let redis_config = crate::queues::redis::RedisQueueConfig {
                    redis_url,
                    concurrency: self.config.queue.redis.concurrency,
                    max_retries: 3,
                    retry_delay_ms: 1000,
                    queue_prefix: "pusher:queue".to_string(),
                };

                let redis_queue = RedisQueueManager::new(redis_config, Arc::new(webhook_sender))?;
                Arc::new(redis_queue)
            }
            QueueDriver::Sqs => {
                // Create SqsQueueConfig from ServerConfig
                let sqs_config = crate::queues::sqs::SqsQueueConfig {
                    queue_url: self.config.queue.sqs.queue_url.clone(),
                    concurrency: self.config.queue.sqs.concurrency,
                    batch_size: self.config.queue.sqs.batch_size as i32,
                    wait_time_seconds: 20,
                    visibility_timeout: 30,
                    max_retries: 3,
                    region: Some(self.config.queue.sqs.region.clone()),
                };

                let sqs_queue = SqsQueueManager::new(sqs_config, Arc::new(webhook_sender)).await?;
                Arc::new(sqs_queue)
            }
        };

        Ok(queue_manager)
    }

    /// Initialize the MetricsManager
    fn initialize_metrics_manager(&self) -> Result<Arc<dyn MetricsManager>> {
        match self.config.metrics.driver {
            MetricsDriver::Prometheus => {
                let metrics = PrometheusMetricsManager::new(Some(&self.config.metrics.prefix))?;
                Ok(Arc::new(metrics))
            }
        }
    }

    /// Initialize the WebhookSender
    fn initialize_webhook_sender(&self) -> WebhookSender {
        WebhookSender::new()
    }

    /// Stop the server gracefully
    ///
    /// This method implements graceful shutdown by:
    /// 1. Setting the closing flag to reject new connections
    /// 2. Closing all existing WebSocket connections with code 4200
    /// 3. Waiting for the configured grace period
    /// 4. Disconnecting all managers (adapter, cache, rate limiter, queue)
    ///
    /// **Validates: Requirements 1.3, 1.4**
    pub async fn stop(&self) -> Result<()> {
        let state = self.state.as_ref().ok_or_else(|| {
            PusherError::ServerError("Server not initialized. Call initialize() first.".to_string())
        })?;

        tracing::info!("Initiating graceful shutdown...");

        // Step 1: Set closing flag to reject new connections
        state
            .closing
            .store(true, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("Closing flag set - new connections will be rejected");

        // Step 2: Close all existing connections with code 4200
        // We need to create a WsHandler to call close_all_local_sockets
        let ws_handler = WsHandler::new(state.clone());
        ws_handler.close_all_local_sockets().await;
        tracing::info!("All existing connections have been signaled to close");

        // Step 3: Wait for grace period to allow connections to close cleanly
        let grace_period_ms = self.config.shutdown_grace_period_ms;
        tracing::info!(
            "Waiting {}ms grace period for connections to close...",
            grace_period_ms
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(grace_period_ms)).await;

        // Step 4: Disconnect all managers
        tracing::info!("Disconnecting managers...");

        // Disconnect adapter
        if let Err(e) = state.adapter.disconnect().await {
            tracing::error!("Error disconnecting adapter: {}", e);
        } else {
            tracing::info!("Adapter disconnected");
        }

        // Disconnect cache manager
        if let Err(e) = state.cache_manager.disconnect().await {
            tracing::error!("Error disconnecting cache manager: {}", e);
        } else {
            tracing::info!("Cache manager disconnected");
        }

        // Disconnect rate limiter
        if let Err(e) = state.rate_limiter.disconnect().await {
            tracing::error!("Error disconnecting rate limiter: {}", e);
        } else {
            tracing::info!("Rate limiter disconnected");
        }

        // Disconnect queue manager
        if let Err(e) = state.queue_manager.disconnect().await {
            tracing::error!("Error disconnecting queue manager: {}", e);
        } else {
            tracing::info!("Queue manager disconnected");
        }

        tracing::info!("Graceful shutdown complete");
        Ok(())
    }

    /// Start the server
    ///
    /// This method:
    /// 1. Starts the Axum server with WebSocket and HTTP routes
    /// 2. Starts the metrics server on a separate port if enabled
    /// 3. Applies path prefix to all routes
    /// 4. Configures SSL if enabled
    /// 5. Logs startup information
    ///
    /// **Validates: Requirements 1.5, 1.7**
    pub async fn start(self) -> Result<()> {
        let state = self.state.ok_or_else(|| {
            PusherError::ServerError("Server not initialized. Call initialize() first.".to_string())
        })?;

        // Log startup information
        tracing::info!("Starting Pusher server...");
        tracing::info!("  Host: {}", self.config.host);
        tracing::info!("  Port: {}", self.config.port);
        tracing::info!(
            "  Path prefix: {}",
            if self.config.path_prefix.is_empty() {
                "(none)"
            } else {
                &self.config.path_prefix
            }
        );
        tracing::info!("  SSL enabled: {}", self.config.ssl.enabled);
        tracing::info!("  Debug mode: {}", self.config.debug);
        tracing::info!("  Adapter: {:?}", self.config.adapter.driver);
        tracing::info!("  App manager: {:?}", self.config.app_manager.driver);
        tracing::info!("  Cache: {:?}", self.config.cache.driver);
        tracing::info!("  Rate limiter: {:?}", self.config.rate_limiter.driver);
        tracing::info!("  Queue: {:?}", self.config.queue.driver);
        tracing::info!("  Metrics enabled: {}", self.config.metrics.enabled);

        // Start metrics server if enabled
        if self.config.metrics.enabled {
            let metrics_state = state.clone();
            let metrics_port = self.config.metrics.port;
            let metrics_host = self.config.host.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    start_metrics_server(metrics_state, metrics_host, metrics_port).await
                {
                    tracing::error!("Metrics server error: {}", e);
                }
            });

            tracing::info!(
                "Metrics server started on {}:{}",
                self.config.host,
                self.config.metrics.port
            );
        }

        // Build the main application router
        let app = build_app_router(state.clone(), &self.config.path_prefix);

        // Build the server address
        let addr_str = format!("{}:{}", self.config.host, self.config.port);
        let addr = addr_str
            .parse::<std::net::SocketAddr>()
            .map_err(|e| PusherError::ServerError(format!("Invalid host/port: {}", e)))?;

        tracing::info!("Server listening on {}", addr);

        // Start the server with or without SSL
        if self.config.ssl.enabled {
            start_with_ssl(app, addr, &self.config.ssl).await?;
        } else {
            start_without_ssl(app, addr).await?;
        }

        Ok(())
    }
}

/// Build the main application router with WebSocket and HTTP routes
///
/// This function creates the Axum router with:
/// - WebSocket handler at /app/{app_key}
/// - HTTP API routes (health checks, events, channels, etc.)
/// - Optional path prefix applied to all routes
///
/// **Validates: Requirements 1.7**
fn build_app_router(state: Arc<AppState>, path_prefix: &str) -> axum::Router {
    use crate::api;
    use crate::ws::ws_handler;
    use axum::{Router, routing::get};

    // Build the base router with WebSocket and API routes
    let router = Router::new()
        .route("/app/{app_key}", get(ws_handler))
        .merge(api::routes(state.clone()))
        .with_state(state);

    // Apply path prefix if configured
    if !path_prefix.is_empty() {
        Router::new().nest(path_prefix, router)
    } else {
        router
    }
}

/// Start the metrics server on a separate port
///
/// The metrics server only exposes the /metrics endpoint and runs on a separate
/// port for security reasons (to avoid exposing metrics on the public API).
///
/// **Validates: Requirements 11.6**
async fn start_metrics_server(state: Arc<AppState>, host: String, port: u16) -> Result<()> {
    use crate::api::metrics;
    use axum::{Router, routing::get};

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(state);

    let addr_str = format!("{}:{}", host, port);
    let addr = addr_str
        .parse::<std::net::SocketAddr>()
        .map_err(|e| PusherError::ServerError(format!("Invalid metrics host/port: {}", e)))?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| PusherError::IoError(e.to_string()))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| PusherError::ServerError(format!("Metrics server error: {}", e)))?;

    Ok(())
}

/// Start the server without SSL
async fn start_without_ssl(app: axum::Router, addr: std::net::SocketAddr) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| PusherError::IoError(e.to_string()))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| PusherError::ServerError(format!("Server error: {}", e)))?;

    Ok(())
}

/// Start the server with SSL/TLS
///
/// This function configures SSL/TLS using the certificate and key files
/// specified in the configuration.
///
/// **Validates: Requirements 1.5**
async fn start_with_ssl(
    app: axum::Router,
    addr: std::net::SocketAddr,
    ssl_config: &SslConfig,
) -> Result<()> {
    use axum_server::tls_rustls::RustlsConfig;

    // Load SSL certificate and key
    let tls_config = RustlsConfig::from_pem_file(&ssl_config.cert_path, &ssl_config.key_path)
        .await
        .map_err(|e| PusherError::ServerError(format!("Failed to load SSL certificate: {}", e)))?;

    // Start the server with TLS
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .map_err(|e| PusherError::ServerError(format!("SSL server error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[tokio::test]
    async fn test_server_new() {
        let config = ServerConfig::default();
        let server = Server::new(config);

        assert!(server.state.is_none());
    }

    #[tokio::test]
    async fn test_server_initialize_with_defaults() {
        let mut config = ServerConfig::default();

        // Set up a default app for testing
        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config);
        let result = server.initialize().await;

        assert!(
            result.is_ok(),
            "Server initialization failed: {:?}",
            result.err()
        );
        assert!(server.state.is_some());

        let state = server.state().unwrap();
        // Just verify the state exists and has the expected components
        assert!(Arc::strong_count(&state) >= 1);
    }

    #[tokio::test]
    async fn test_server_initialize_with_local_adapter() {
        let mut config = ServerConfig::default();
        config.adapter.driver = AdapterDriver::Local;

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config);
        let result = server.initialize().await;

        assert!(result.is_ok());
        let state = server.state().unwrap();

        // Verify adapter is initialized
        let socket_count = state.adapter.get_sockets_count("test_app").await;
        assert!(socket_count.is_ok());
    }

    #[tokio::test]
    async fn test_server_initialize_with_memory_cache() {
        let mut config = ServerConfig::default();
        config.cache.driver = CacheDriver::Memory;

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config);
        let result = server.initialize().await;

        assert!(result.is_ok());
        let state = server.state().unwrap();

        // Verify cache manager is initialized
        let cache_result = state
            .cache_manager
            .set("test_key", "test_value", None)
            .await;
        assert!(cache_result.is_ok());

        let get_result = state.cache_manager.get("test_key").await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_server_initialize_with_metrics_enabled() {
        let mut config = ServerConfig::default();
        config.metrics.enabled = true;
        config.metrics.prefix = "test_pusher".to_string();

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config);
        let result = server.initialize().await;

        assert!(result.is_ok());
        let state = server.state().unwrap();

        // Verify metrics manager is initialized
        assert!(state.metrics_manager.is_some());
    }

    #[tokio::test]
    async fn test_server_initialize_with_metrics_disabled() {
        let mut config = ServerConfig::default();
        config.metrics.enabled = false;

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config);
        let result = server.initialize().await;

        assert!(result.is_ok());
        let state = server.state().unwrap();

        // Verify metrics manager is not initialized
        assert!(state.metrics_manager.is_none());
    }

    #[tokio::test]
    async fn test_server_initialize_with_array_app_manager() {
        let mut config = ServerConfig::default();
        config.app_manager.driver = AppManagerDriver::Array;

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
        config.app_manager.array.apps = vec![app1, app2];

        let mut server = Server::new(config);
        let result = server.initialize().await;

        assert!(result.is_ok());
        let state = server.state().unwrap();

        // Verify app manager can find apps
        let app1_result = state.app_manager.find_by_id("app1").await;
        assert!(app1_result.is_ok());
        assert!(app1_result.unwrap().is_some());

        let app2_result = state.app_manager.find_by_key("key2").await;
        assert!(app2_result.is_ok());
        assert!(app2_result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_server_state_before_initialization() {
        let config = ServerConfig::default();
        let server = Server::new(config);

        assert!(server.state().is_none());
    }

    #[tokio::test]
    async fn test_server_state_after_initialization() {
        let mut config = ServerConfig::default();

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config);
        server.initialize().await.unwrap();

        assert!(server.state().is_some());

        // Verify we can get state multiple times
        let state1 = server.state().unwrap();
        let state2 = server.state().unwrap();

        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&state1, &state2));
    }

    #[tokio::test]
    async fn test_build_app_router_without_prefix() {
        let mut config = ServerConfig {
            path_prefix: "".to_string(),
            ..Default::default()
        };

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config.clone());
        server.initialize().await.unwrap();
        let state = server.state().unwrap();

        // Build router without prefix
        let _router = build_app_router(state, &config.path_prefix);

        // Just verify it builds successfully
        // In a real test, we would make requests to verify routes work
    }

    #[tokio::test]
    async fn test_build_app_router_with_prefix() {
        let mut config = ServerConfig {
            path_prefix: "/pusher".to_string(),
            ..Default::default()
        };

        let default_app = App::new(
            "test_app".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        );
        config.app_manager.array.apps = vec![default_app];

        let mut server = Server::new(config.clone());
        server.initialize().await.unwrap();
        let state = server.state().unwrap();

        // Build router with prefix
        let _router = build_app_router(state, &config.path_prefix);

        // Just verify it builds successfully
        // In a real test, we would make requests to verify routes work with prefix
    }

    #[tokio::test]
    async fn test_server_start_requires_initialization() {
        let config = ServerConfig::default();
        let server = Server::new(config);

        // Try to start without initialization
        let result = server.start().await;

        // Should fail because server is not initialized
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }
}
