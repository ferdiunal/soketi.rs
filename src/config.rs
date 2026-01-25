use serde::{Deserialize, Serialize};

/// Main server configuration
///
/// This struct contains all configuration options for the Pusher server.
/// It can be loaded from:
/// - Configuration files (JSON, YAML, TOML)
/// - Environment variables (with PUSHER_ prefix)
/// - Command-line arguments
///
/// # Examples
///
/// ## Creating a default configuration
/// ```
/// use soketi_rs::config::ServerConfig;
///
/// let config = ServerConfig::default();
/// assert_eq!(config.host, "0.0.0.0");
/// assert_eq!(config.port, 6001);
/// ```
///
/// ## Loading from environment variables
/// ```ignore
/// // Set environment variables:
/// // PUSHER_HOST=127.0.0.1
/// // PUSHER_PORT=8080
/// // PUSHER_DEBUG=true
///
/// use soketi_rs::options::Options;
/// let config = Options::load().unwrap();
/// ```
///
/// **Validates: Requirements 1.2, 14.1-14.10**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address (default: "0.0.0.0")
    pub host: String,
    /// Server port (default: 6001)
    pub port: u16,
    /// Path prefix for all routes (default: "")
    pub path_prefix: String,
    /// Enable debug mode for verbose logging (default: false)
    pub debug: bool,
    /// Server mode: Full, Server, or Worker (default: Full)
    pub mode: ServerMode,
    /// Graceful shutdown grace period in milliseconds (default: 3000)
    pub shutdown_grace_period_ms: u64,

    /// Adapter configuration for socket management
    pub adapter: AdapterConfig,
    /// App manager configuration for application credentials
    pub app_manager: AppManagerConfig,
    /// Cache manager configuration
    pub cache: CacheConfig,
    /// Rate limiter configuration
    pub rate_limiter: RateLimiterConfig,
    /// Queue manager configuration for webhook processing
    pub queue: QueueConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// SSL/TLS configuration
    pub ssl: SslConfig,
    /// CORS configuration
    pub cors: CorsConfig,
    /// Channel limits (name length, cache TTL)
    pub channel_limits: ChannelLimits,
    /// Event limits (payload size, batch size, etc.)
    pub event_limits: EventLimits,
    /// Presence channel limits
    pub presence: PresenceLimits,
    /// HTTP API configuration
    pub http_api: HttpApiConfig,
    /// Webhook configuration
    pub webhooks: WebhookConfig,
    /// User authentication timeout in milliseconds (default: 30000)
    pub user_authentication_timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 6001,
            path_prefix: "".to_string(),
            debug: false,
            mode: ServerMode::Full,
            shutdown_grace_period_ms: 3000,

            adapter: AdapterConfig::default(),
            app_manager: AppManagerConfig::default(),
            cache: CacheConfig::default(),
            rate_limiter: RateLimiterConfig::default(),
            queue: QueueConfig::default(),
            metrics: MetricsConfig::default(),

            ssl: SslConfig::default(),
            cors: CorsConfig::default(),
            channel_limits: ChannelLimits::default(),
            event_limits: EventLimits::default(),
            presence: PresenceLimits::default(),
            http_api: HttpApiConfig::default(),
            webhooks: WebhookConfig::default(),
            user_authentication_timeout_ms: 30000,
        }
    }
}

/// Server operating mode
///
/// Determines which components of the server are active:
/// - **Full**: Run both server (WebSocket/HTTP) and worker (queue processing)
/// - **Server**: Run only the server components (no queue workers)
/// - **Worker**: Run only the queue workers (no server)
///
/// This allows for horizontal scaling by running separate server and worker instances.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerMode {
    /// Run both server and worker components
    Full,
    /// Run only server components (WebSocket/HTTP)
    Server,
    /// Run only worker components (queue processing)
    Worker,
}

/// Adapter configuration
///
/// The adapter manages socket connections and message distribution.
/// Different drivers support different deployment scenarios:
/// - **Local**: Single-instance deployment (no message distribution)
/// - **Cluster**: Multi-instance deployment with UDP discovery
/// - **Redis**: Horizontal scaling using Redis pub/sub
/// - **NATS**: Horizontal scaling using NATS messaging
///
/// **Validates: Requirements 4.1-4.4**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Adapter driver to use
    pub driver: AdapterDriver,
    /// Redis adapter configuration
    pub redis: RedisAdapterConfig,
    /// Cluster adapter configuration
    pub cluster: ClusterAdapterConfig,
    /// NATS adapter configuration
    pub nats: NatsAdapterConfig,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            driver: AdapterDriver::Local,
            redis: RedisAdapterConfig::default(),
            cluster: ClusterAdapterConfig::default(),
            nats: NatsAdapterConfig::default(),
        }
    }
}

/// Adapter driver selection
///
/// **Validates: Requirements 4.1-4.4**
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdapterDriver {
    /// Local adapter for single-instance deployments
    Local,
    /// Cluster adapter for multi-instance deployments with UDP discovery
    Cluster,
    /// Redis adapter for horizontal scaling using Redis pub/sub
    Redis,
    /// NATS adapter for horizontal scaling using NATS messaging
    Nats,
}

/// Redis adapter configuration
///
/// Configuration for using Redis as the adapter backend for horizontal scaling.
///
/// # Examples
///
/// ```
/// use soketi_rs::config::RedisAdapterConfig;
///
/// let config = RedisAdapterConfig {
///     host: "redis.example.com".to_string(),
///     port: 6379,
///     db: 0,
///     username: None,
///     password: Some("secret".to_string()),
///     key_prefix: "pusher:".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisAdapterConfig {
    /// Redis server host (default: "127.0.0.1")
    pub host: String,
    /// Redis server port (default: 6379)
    pub port: u16,
    /// Redis database number (default: 0)
    pub db: u8,
    /// Redis username (optional, for Redis 6+)
    pub username: Option<String>,
    /// Redis password (optional)
    pub password: Option<String>,
    /// Key prefix for Redis keys (default: "")
    pub key_prefix: String,
}

impl Default for RedisAdapterConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            db: 0,
            username: None,
            password: None,
            key_prefix: "".to_string(),
        }
    }
}

/// Cluster adapter configuration
///
/// Configuration for using UDP multicast discovery for cluster mode.
/// Nodes discover each other automatically using UDP broadcast.
///
/// **Validates: Requirements 17.1-17.5**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAdapterConfig {
    /// UDP port for cluster communication (default: 11002)
    pub port: u16,
    /// Multicast address for node discovery (default: "239.1.1.1")
    pub multicast_address: String,
    /// Request timeout in milliseconds (default: 5000)
    pub request_timeout_ms: u64,
}

impl Default for ClusterAdapterConfig {
    fn default() -> Self {
        Self {
            port: 11002,
            multicast_address: "239.1.1.1".to_string(),
            request_timeout_ms: 5000,
        }
    }
}

/// NATS adapter configuration
///
/// Configuration for using NATS as the adapter backend for horizontal scaling.
///
/// # Examples
///
/// ```
/// use soketi_rs::config::NatsAdapterConfig;
///
/// let config = NatsAdapterConfig {
///     servers: vec!["nats://nats1:4222".to_string(), "nats://nats2:4222".to_string()],
///     user: Some("pusher".to_string()),
///     password: Some("secret".to_string()),
///     token: None,
///     timeout_ms: 10000,
///     prefix: "pusher".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAdapterConfig {
    /// NATS server URLs (default: ["127.0.0.1:4222"])
    pub servers: Vec<String>,
    /// NATS username (optional)
    pub user: Option<String>,
    /// NATS password (optional)
    pub password: Option<String>,
    /// NATS token for authentication (optional)
    pub token: Option<String>,
    /// Connection timeout in milliseconds (default: 10000)
    pub timeout_ms: u64,
    /// Subject prefix for NATS messages (default: "")
    pub prefix: String,
}

impl Default for NatsAdapterConfig {
    fn default() -> Self {
        Self {
            servers: vec!["127.0.0.1:4222".to_string()],
            user: None,
            password: None,
            token: None,
            timeout_ms: 10000,
            prefix: "".to_string(),
        }
    }
}

/// App Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManagerConfig {
    pub driver: AppManagerDriver,
    pub array: ArrayAppManagerConfig,
    pub dynamodb: DynamoDbAppManagerConfig,
    pub mysql: MysqlAppManagerConfig,
    pub postgres: PostgresAppManagerConfig,
    pub cache: AppManagerCacheConfig,
}

impl Default for AppManagerConfig {
    fn default() -> Self {
        Self {
            driver: AppManagerDriver::Array,
            array: ArrayAppManagerConfig::default(),
            dynamodb: DynamoDbAppManagerConfig::default(),
            mysql: MysqlAppManagerConfig::default(),
            postgres: PostgresAppManagerConfig::default(),
            cache: AppManagerCacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppManagerDriver {
    Array,
    DynamoDb,
    Mysql,
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArrayAppManagerConfig {
    pub apps: Vec<crate::app::App>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamoDbAppManagerConfig {
    pub table: String,
    pub region: String,
    pub endpoint: Option<String>,
}

impl Default for DynamoDbAppManagerConfig {
    fn default() -> Self {
        Self {
            table: "apps".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlAppManagerConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub table: String,
    pub version: String,
}

impl Default for MysqlAppManagerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3306,
            user: "root".to_string(),
            password: "".to_string(),
            database: "main".to_string(),
            table: "apps".to_string(),
            version: "8.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresAppManagerConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub table: String,
    pub version: String,
}

impl Default for PostgresAppManagerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            password: "".to_string(),
            database: "main".to_string(),
            table: "apps".to_string(),
            version: "13.3".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManagerCacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
}

impl Default for AppManagerCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ttl_seconds: 3600,
        }
    }
}

/// Cache Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub driver: CacheDriver,
    pub redis: RedisCacheConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            driver: CacheDriver::Memory,
            redis: RedisCacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheDriver {
    Memory,
    Redis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub username: Option<String>,
    pub password: Option<String>,
    pub key_prefix: String,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            db: 0,
            username: None,
            password: None,
            key_prefix: "".to_string(),
        }
    }
}

/// Rate Limiter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    pub driver: RateLimiterDriver,
    pub redis: RedisRateLimiterConfig,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            driver: RateLimiterDriver::Local,
            redis: RedisRateLimiterConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RateLimiterDriver {
    Local,
    Cluster,
    Redis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisRateLimiterConfig {
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub username: Option<String>,
    pub password: Option<String>,
    pub key_prefix: String,
}

impl Default for RedisRateLimiterConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            db: 0,
            username: None,
            password: None,
            key_prefix: "".to_string(),
        }
    }
}

/// Queue Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub driver: QueueDriver,
    pub redis: RedisQueueConfig,
    pub sqs: SqsQueueConfig,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            driver: QueueDriver::Sync,
            redis: RedisQueueConfig::default(),
            sqs: SqsQueueConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueDriver {
    Sync,
    Redis,
    Sqs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisQueueConfig {
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub username: Option<String>,
    pub password: Option<String>,
    pub concurrency: usize,
}

impl Default for RedisQueueConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            db: 0,
            username: None,
            password: None,
            concurrency: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqsQueueConfig {
    pub region: String,
    pub queue_url: String,
    pub endpoint: Option<String>,
    pub concurrency: usize,
    pub batch_size: usize,
}

impl Default for SqsQueueConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            queue_url: "".to_string(),
            endpoint: None,
            concurrency: 1,
            batch_size: 10,
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub driver: MetricsDriver,
    pub port: u16,
    pub prefix: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            driver: MetricsDriver::Prometheus,
            port: 9601,
            prefix: "pusher".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricsDriver {
    Prometheus,
}

/// SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub passphrase: Option<String>,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: "".to_string(),
            key_path: "".to_string(),
            passphrase: None,
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            origins: vec!["*".to_string()],
            methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            headers: vec!["*".to_string()],
            credentials: true,
        }
    }
}

/// Channel limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelLimits {
    pub max_name_length: u64,
    pub cache_ttl_seconds: u64,
}

impl Default for ChannelLimits {
    fn default() -> Self {
        Self {
            max_name_length: 200,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Event limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLimits {
    pub max_channels_at_once: u64,
    pub max_name_length: u64,
    pub max_payload_in_kb: f64,
    pub max_batch_size: u64,
}

impl Default for EventLimits {
    fn default() -> Self {
        Self {
            max_channels_at_once: 100,
            max_name_length: 200,
            max_payload_in_kb: 100.0,
            max_batch_size: 10,
        }
    }
}

/// Presence limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceLimits {
    pub max_members_per_channel: u64,
    pub max_member_size_in_kb: f64,
}

impl Default for PresenceLimits {
    fn default() -> Self {
        Self {
            max_members_per_channel: 100,
            max_member_size_in_kb: 2.0,
        }
    }
}

/// HTTP API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpApiConfig {
    pub max_request_size_in_kb: f64,
    pub accept_traffic_memory_threshold_mb: u64,
}

impl Default for HttpApiConfig {
    fn default() -> Self {
        Self {
            max_request_size_in_kb: 100.0,
            accept_traffic_memory_threshold_mb: 512,
        }
    }
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub batching_enabled: bool,
    pub batch_duration_ms: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            batching_enabled: false,
            batch_duration_ms: 50,
        }
    }
}
