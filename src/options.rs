use crate::config::ServerConfig;
use crate::error::{PusherError, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Options struct for loading configuration from multiple sources
///
/// This struct supports loading configuration from:
/// 1. Command-line arguments (via clap)
/// 2. Environment variables (via envy)
/// 3. Configuration files (JSON, YAML, TOML)
///
/// The loading priority is (highest to lowest):
/// 1. Command-line arguments
/// 2. Environment variables
/// 3. Configuration file
/// 4. Default values
///
/// **Validates: Requirements 1.2, 14.1-14.10**
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
#[derive(Default)]
pub struct Options {
    /// Path to configuration file (JSON, YAML, or TOML)
    #[arg(long, short = 'c', env = "PUSHER_CONFIG_FILE")]
    #[serde(skip)]
    pub config_file: Option<PathBuf>,

    /// Server host address
    #[arg(long, env = "PUSHER_HOST")]
    #[serde(default)]
    pub host: Option<String>,

    /// Server port
    #[arg(long, env = "PUSHER_PORT")]
    #[serde(default)]
    pub port: Option<u16>,

    /// Path prefix for all routes
    #[arg(long, env = "PUSHER_PATH_PREFIX")]
    #[serde(default)]
    pub path_prefix: Option<String>,

    /// Enable debug mode
    #[arg(long, env = "PUSHER_DEBUG")]
    #[serde(default)]
    pub debug: Option<bool>,

    /// Server mode (full, server, worker)
    #[arg(long, env = "PUSHER_MODE")]
    #[serde(default)]
    pub mode: Option<String>,

    /// Shutdown grace period in milliseconds
    #[arg(long, env = "PUSHER_SHUTDOWN_GRACE_PERIOD_MS")]
    #[serde(default)]
    pub shutdown_grace_period_ms: Option<u64>,

    // Adapter configuration
    /// Adapter driver (local, cluster, redis, nats)
    #[arg(long, env = "PUSHER_ADAPTER_DRIVER")]
    #[serde(default)]
    pub adapter_driver: Option<String>,

    /// Redis adapter host
    #[arg(long, env = "PUSHER_ADAPTER_REDIS_HOST")]
    #[serde(default)]
    pub adapter_redis_host: Option<String>,

    /// Redis adapter port
    #[arg(long, env = "PUSHER_ADAPTER_REDIS_PORT")]
    #[serde(default)]
    pub adapter_redis_port: Option<u16>,

    /// Redis adapter database
    #[arg(long, env = "PUSHER_ADAPTER_REDIS_DB")]
    #[serde(default)]
    pub adapter_redis_db: Option<u8>,

    /// Redis adapter password
    #[arg(long, env = "PUSHER_ADAPTER_REDIS_PASSWORD")]
    #[serde(default)]
    pub adapter_redis_password: Option<String>,

    /// Cluster adapter port
    #[arg(long, env = "PUSHER_ADAPTER_CLUSTER_PORT")]
    #[serde(default)]
    pub adapter_cluster_port: Option<u16>,

    /// NATS adapter servers (comma-separated)
    #[arg(long, env = "PUSHER_ADAPTER_NATS_SERVERS")]
    #[serde(default)]
    pub adapter_nats_servers: Option<String>,

    // App Manager configuration
    /// App manager driver (array, dynamodb, mysql, postgres)
    #[arg(long, env = "PUSHER_APP_MANAGER_DRIVER")]
    #[serde(default)]
    pub app_manager_driver: Option<String>,

    /// App manager cache enabled
    #[arg(long, env = "PUSHER_APP_MANAGER_CACHE_ENABLED")]
    #[serde(default)]
    pub app_manager_cache_enabled: Option<bool>,

    /// App manager cache TTL in seconds
    #[arg(long, env = "PUSHER_APP_MANAGER_CACHE_TTL")]
    #[serde(default)]
    pub app_manager_cache_ttl: Option<u64>,

    /// Array app manager apps (JSON string)
    #[arg(long, env = "PUSHER_APP_MANAGER_ARRAY_APPS")]
    #[serde(default)]
    pub app_manager_array_apps: Option<String>,

    /// Default app ID (creates a single default app)
    #[arg(long, env = "PUSHER_DEFAULT_APP_ID")]
    #[serde(default)]
    pub default_app_id: Option<String>,

    /// Default app key (creates a single default app)
    #[arg(long, env = "PUSHER_DEFAULT_APP_KEY")]
    #[serde(default)]
    pub default_app_key: Option<String>,

    /// Default app secret (creates a single default app)
    #[arg(long, env = "PUSHER_DEFAULT_APP_SECRET")]
    #[serde(default)]
    pub default_app_secret: Option<String>,

    /// DynamoDB table name
    #[arg(long, env = "PUSHER_DYNAMODB_TABLE")]
    #[serde(default)]
    pub dynamodb_table: Option<String>,

    /// DynamoDB region
    #[arg(long, env = "PUSHER_DYNAMODB_REGION")]
    #[serde(default)]
    pub dynamodb_region: Option<String>,

    /// MySQL host
    #[arg(long, env = "PUSHER_MYSQL_HOST")]
    #[serde(default)]
    pub mysql_host: Option<String>,

    /// MySQL port
    #[arg(long, env = "PUSHER_MYSQL_PORT")]
    #[serde(default)]
    pub mysql_port: Option<u16>,

    /// MySQL user
    #[arg(long, env = "PUSHER_MYSQL_USER")]
    #[serde(default)]
    pub mysql_user: Option<String>,

    /// MySQL password
    #[arg(long, env = "PUSHER_MYSQL_PASSWORD")]
    #[serde(default)]
    pub mysql_password: Option<String>,

    /// MySQL database
    #[arg(long, env = "PUSHER_MYSQL_DATABASE")]
    #[serde(default)]
    pub mysql_database: Option<String>,

    /// PostgreSQL host
    #[arg(long, env = "PUSHER_POSTGRES_HOST")]
    #[serde(default)]
    pub postgres_host: Option<String>,

    /// PostgreSQL port
    #[arg(long, env = "PUSHER_POSTGRES_PORT")]
    #[serde(default)]
    pub postgres_port: Option<u16>,

    /// PostgreSQL user
    #[arg(long, env = "PUSHER_POSTGRES_USER")]
    #[serde(default)]
    pub postgres_user: Option<String>,

    /// PostgreSQL password
    #[arg(long, env = "PUSHER_POSTGRES_PASSWORD")]
    #[serde(default)]
    pub postgres_password: Option<String>,

    /// PostgreSQL database
    #[arg(long, env = "PUSHER_POSTGRES_DATABASE")]
    #[serde(default)]
    pub postgres_database: Option<String>,

    // Cache Manager configuration
    /// Cache driver (memory, redis)
    #[arg(long, env = "PUSHER_CACHE_DRIVER")]
    #[serde(default)]
    pub cache_driver: Option<String>,

    /// Redis cache host
    #[arg(long, env = "PUSHER_CACHE_REDIS_HOST")]
    #[serde(default)]
    pub cache_redis_host: Option<String>,

    /// Redis cache port
    #[arg(long, env = "PUSHER_CACHE_REDIS_PORT")]
    #[serde(default)]
    pub cache_redis_port: Option<u16>,

    /// Redis cache password
    #[arg(long, env = "PUSHER_CACHE_REDIS_PASSWORD")]
    #[serde(default)]
    pub cache_redis_password: Option<String>,

    // Rate Limiter configuration
    /// Rate limiter driver (local, cluster, redis)
    #[arg(long, env = "PUSHER_RATE_LIMITER_DRIVER")]
    #[serde(default)]
    pub rate_limiter_driver: Option<String>,

    /// Redis rate limiter host
    #[arg(long, env = "PUSHER_RATE_LIMITER_REDIS_HOST")]
    #[serde(default)]
    pub rate_limiter_redis_host: Option<String>,

    /// Redis rate limiter port
    #[arg(long, env = "PUSHER_RATE_LIMITER_REDIS_PORT")]
    #[serde(default)]
    pub rate_limiter_redis_port: Option<u16>,

    // Queue Manager configuration
    /// Queue driver (sync, redis, sqs)
    #[arg(long, env = "PUSHER_QUEUE_DRIVER")]
    #[serde(default)]
    pub queue_driver: Option<String>,

    /// Redis queue host
    #[arg(long, env = "PUSHER_QUEUE_REDIS_HOST")]
    #[serde(default)]
    pub queue_redis_host: Option<String>,

    /// Redis queue port
    #[arg(long, env = "PUSHER_QUEUE_REDIS_PORT")]
    #[serde(default)]
    pub queue_redis_port: Option<u16>,

    /// SQS queue URL
    #[arg(long, env = "PUSHER_QUEUE_SQS_URL")]
    #[serde(default)]
    pub queue_sqs_url: Option<String>,

    /// SQS region
    #[arg(long, env = "PUSHER_QUEUE_SQS_REGION")]
    #[serde(default)]
    pub queue_sqs_region: Option<String>,

    // Metrics configuration
    /// Enable metrics
    #[arg(long, env = "PUSHER_METRICS_ENABLED")]
    #[serde(default)]
    pub metrics_enabled: Option<bool>,

    /// Metrics port
    #[arg(long, env = "PUSHER_METRICS_PORT")]
    #[serde(default)]
    pub metrics_port: Option<u16>,

    /// Metrics prefix
    #[arg(long, env = "PUSHER_METRICS_PREFIX")]
    #[serde(default)]
    pub metrics_prefix: Option<String>,

    // SSL configuration
    /// Enable SSL
    #[arg(long, env = "PUSHER_SSL_ENABLED")]
    #[serde(default)]
    pub ssl_enabled: Option<bool>,

    /// SSL certificate path
    #[arg(long, env = "PUSHER_SSL_CERT_PATH")]
    #[serde(default)]
    pub ssl_cert_path: Option<String>,

    /// SSL key path
    #[arg(long, env = "PUSHER_SSL_KEY_PATH")]
    #[serde(default)]
    pub ssl_key_path: Option<String>,

    // CORS configuration
    /// Enable CORS
    #[arg(long, env = "PUSHER_CORS_ENABLED")]
    #[serde(default)]
    pub cors_enabled: Option<bool>,

    /// CORS allowed origins (comma-separated)
    #[arg(long, env = "PUSHER_CORS_ORIGINS")]
    #[serde(default)]
    pub cors_origins: Option<String>,

    // Channel limits
    /// Maximum channel name length
    #[arg(long, env = "PUSHER_CHANNEL_MAX_NAME_LENGTH")]
    #[serde(default)]
    pub channel_max_name_length: Option<u64>,

    // Event limits
    /// Maximum event name length
    #[arg(long, env = "PUSHER_EVENT_MAX_NAME_LENGTH")]
    #[serde(default)]
    pub event_max_name_length: Option<u64>,

    /// Maximum event payload size in KB
    #[arg(long, env = "PUSHER_EVENT_MAX_PAYLOAD_KB")]
    #[serde(default)]
    pub event_max_payload_kb: Option<f64>,

    /// Maximum batch size
    #[arg(long, env = "PUSHER_EVENT_MAX_BATCH_SIZE")]
    #[serde(default)]
    pub event_max_batch_size: Option<u64>,

    // Presence limits
    /// Maximum presence members per channel
    #[arg(long, env = "PUSHER_PRESENCE_MAX_MEMBERS")]
    #[serde(default)]
    pub presence_max_members: Option<u64>,

    /// Maximum presence member size in KB
    #[arg(long, env = "PUSHER_PRESENCE_MAX_MEMBER_SIZE_KB")]
    #[serde(default)]
    pub presence_max_member_size_kb: Option<f64>,

    // HTTP API configuration
    /// Maximum HTTP request size in KB
    #[arg(long, env = "PUSHER_HTTP_MAX_REQUEST_SIZE_KB")]
    #[serde(default)]
    pub http_max_request_size_kb: Option<f64>,

    /// Memory threshold for accept-traffic endpoint in MB
    #[arg(long, env = "PUSHER_HTTP_MEMORY_THRESHOLD_MB")]
    #[serde(default)]
    pub http_memory_threshold_mb: Option<u64>,

    // User authentication
    /// User authentication timeout in milliseconds
    #[arg(long, env = "PUSHER_USER_AUTH_TIMEOUT_MS")]
    #[serde(default)]
    pub user_auth_timeout_ms: Option<u64>,
}

impl Options {
    /// Create a new Options instance by parsing command-line arguments
    ///
    /// This is the primary entry point for loading configuration.
    /// It will:
    /// 1. Parse command-line arguments
    /// 2. Load configuration file if specified
    /// 3. Merge with environment variables
    /// 4. Apply defaults
    pub fn new() -> Self {
        Options::parse()
    }

    /// Load options from all sources and convert to ServerConfig
    ///
    /// This method:
    /// 1. Starts with default ServerConfig
    /// 2. Loads configuration file if specified
    /// 3. Applies environment variables
    /// 4. Applies command-line arguments (highest priority)
    ///
    /// **Validates: Requirements 1.2, 14.1-14.10**
    pub fn load() -> Result<ServerConfig> {
        // Parse command-line arguments
        let cli_options = Options::parse();

        // Start with default config
        let mut config = ServerConfig::default();

        // Load from configuration file if specified
        if let Some(config_file) = &cli_options.config_file {
            config = Self::load_from_file(config_file)?;
        }

        // Apply environment variables
        Self::apply_env_vars(&mut config)?;

        // Apply command-line arguments (highest priority)
        cli_options.apply_to_config(&mut config);

        Ok(config)
    }

    /// Load configuration from a file (JSON, YAML, or TOML)
    ///
    /// The file format is determined by the file extension:
    /// - .json: JSON format
    /// - .yaml or .yml: YAML format
    /// - .toml: TOML format
    pub fn load_from_file(path: &PathBuf) -> Result<ServerConfig> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| PusherError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PusherError::ConfigError("Config file has no extension".to_string()))?;

        match extension {
            "json" => serde_json::from_str(&contents).map_err(|e| {
                PusherError::ConfigError(format!("Failed to parse JSON config: {}", e))
            }),
            "yaml" | "yml" => serde_yaml::from_str(&contents).map_err(|e| {
                PusherError::ConfigError(format!("Failed to parse YAML config: {}", e))
            }),
            "toml" => toml::from_str(&contents).map_err(|e| {
                PusherError::ConfigError(format!("Failed to parse TOML config: {}", e))
            }),
            _ => Err(PusherError::ConfigError(format!(
                "Unsupported config file format: {}",
                extension
            ))),
        }
    }

    /// Apply environment variables to the configuration
    ///
    /// This method reads environment variables with the PUSHER_ prefix
    /// and applies them to the configuration.
    fn apply_env_vars(config: &mut ServerConfig) -> Result<()> {
        // Use envy to load environment variables into an Options struct
        // Then apply those options to the config
        if let Ok(env_options) = envy::prefixed("PUSHER_").from_env::<Options>() {
            env_options.apply_to_config(config);
        }

        Ok(())
    }

    /// Apply this Options instance to a ServerConfig
    ///
    /// This method applies all non-None values from this Options instance
    /// to the provided ServerConfig, overriding existing values.
    pub fn apply_to_config(&self, config: &mut ServerConfig) {
        use crate::config::*;

        // Server configuration
        if let Some(host) = &self.host {
            config.host = host.clone();
        }
        if let Some(port) = self.port {
            config.port = port;
        }
        if let Some(path_prefix) = &self.path_prefix {
            config.path_prefix = path_prefix.clone();
        }
        if let Some(debug) = self.debug {
            config.debug = debug;
        }
        if let Some(mode) = &self.mode {
            config.mode = match mode.as_str() {
                "full" => ServerMode::Full,
                "server" => ServerMode::Server,
                "worker" => ServerMode::Worker,
                _ => ServerMode::Full,
            };
        }
        if let Some(grace_period) = self.shutdown_grace_period_ms {
            config.shutdown_grace_period_ms = grace_period;
        }

        // Adapter configuration
        if let Some(driver) = &self.adapter_driver {
            config.adapter.driver = match driver.as_str() {
                "local" => AdapterDriver::Local,
                "cluster" => AdapterDriver::Cluster,
                "redis" => AdapterDriver::Redis,
                "nats" => AdapterDriver::Nats,
                _ => AdapterDriver::Local,
            };
        }
        if let Some(host) = &self.adapter_redis_host {
            config.adapter.redis.host = host.clone();
        }
        if let Some(port) = self.adapter_redis_port {
            config.adapter.redis.port = port;
        }
        if let Some(db) = self.adapter_redis_db {
            config.adapter.redis.db = db;
        }
        if let Some(password) = &self.adapter_redis_password {
            config.adapter.redis.password = Some(password.clone());
        }
        if let Some(port) = self.adapter_cluster_port {
            config.adapter.cluster.port = port;
        }
        if let Some(servers) = &self.adapter_nats_servers {
            config.adapter.nats.servers =
                servers.split(',').map(|s| s.trim().to_string()).collect();
        }

        // App Manager configuration
        if let Some(driver) = &self.app_manager_driver {
            config.app_manager.driver = match driver.as_str() {
                "array" => AppManagerDriver::Array,
                "dynamodb" => AppManagerDriver::DynamoDb,
                "mysql" => AppManagerDriver::Mysql,
                "postgres" => AppManagerDriver::Postgres,
                _ => AppManagerDriver::Array,
            };
        }
        if let Some(enabled) = self.app_manager_cache_enabled {
            config.app_manager.cache.enabled = enabled;
        }
        if let Some(ttl) = self.app_manager_cache_ttl {
            config.app_manager.cache.ttl_seconds = ttl;
        }

        // Handle array apps from environment variable (JSON string)
        if let Some(apps_json) = &self.app_manager_array_apps {
            if let Ok(apps) = serde_json::from_str::<Vec<crate::app::App>>(apps_json) {
                config.app_manager.array.apps = apps;
            }
        }

        // Handle default app creation from environment variables
        if let (Some(id), Some(key), Some(secret)) = (
            &self.default_app_id,
            &self.default_app_key,
            &self.default_app_secret,
        ) {
            // Create a default app if all three values are provided
            let default_app = crate::app::App::new(id.clone(), key.clone(), secret.clone());
            
            // If no apps exist, add the default app
            // If apps exist, replace the first one or add it
            if config.app_manager.array.apps.is_empty() {
                config.app_manager.array.apps.push(default_app);
            } else {
                // Replace first app with default app
                config.app_manager.array.apps[0] = default_app;
            }
        }

        if let Some(table) = &self.dynamodb_table {
            config.app_manager.dynamodb.table = table.clone();
        }
        if let Some(region) = &self.dynamodb_region {
            config.app_manager.dynamodb.region = region.clone();
        }
        if let Some(host) = &self.mysql_host {
            config.app_manager.mysql.host = host.clone();
        }
        if let Some(port) = self.mysql_port {
            config.app_manager.mysql.port = port;
        }
        if let Some(user) = &self.mysql_user {
            config.app_manager.mysql.user = user.clone();
        }
        if let Some(password) = &self.mysql_password {
            config.app_manager.mysql.password = password.clone();
        }
        if let Some(database) = &self.mysql_database {
            config.app_manager.mysql.database = database.clone();
        }
        if let Some(host) = &self.postgres_host {
            config.app_manager.postgres.host = host.clone();
        }
        if let Some(port) = self.postgres_port {
            config.app_manager.postgres.port = port;
        }
        if let Some(user) = &self.postgres_user {
            config.app_manager.postgres.user = user.clone();
        }
        if let Some(password) = &self.postgres_password {
            config.app_manager.postgres.password = password.clone();
        }
        if let Some(database) = &self.postgres_database {
            config.app_manager.postgres.database = database.clone();
        }

        // Cache Manager configuration
        if let Some(driver) = &self.cache_driver {
            config.cache.driver = match driver.as_str() {
                "memory" => CacheDriver::Memory,
                "redis" => CacheDriver::Redis,
                _ => CacheDriver::Memory,
            };
        }
        if let Some(host) = &self.cache_redis_host {
            config.cache.redis.host = host.clone();
        }
        if let Some(port) = self.cache_redis_port {
            config.cache.redis.port = port;
        }
        if let Some(password) = &self.cache_redis_password {
            config.cache.redis.password = Some(password.clone());
        }

        // Rate Limiter configuration
        if let Some(driver) = &self.rate_limiter_driver {
            config.rate_limiter.driver = match driver.as_str() {
                "local" => RateLimiterDriver::Local,
                "cluster" => RateLimiterDriver::Cluster,
                "redis" => RateLimiterDriver::Redis,
                _ => RateLimiterDriver::Local,
            };
        }
        if let Some(host) = &self.rate_limiter_redis_host {
            config.rate_limiter.redis.host = host.clone();
        }
        if let Some(port) = self.rate_limiter_redis_port {
            config.rate_limiter.redis.port = port;
        }

        // Queue Manager configuration
        if let Some(driver) = &self.queue_driver {
            config.queue.driver = match driver.as_str() {
                "sync" => QueueDriver::Sync,
                "redis" => QueueDriver::Redis,
                "sqs" => QueueDriver::Sqs,
                _ => QueueDriver::Sync,
            };
        }
        if let Some(host) = &self.queue_redis_host {
            config.queue.redis.host = host.clone();
        }
        if let Some(port) = self.queue_redis_port {
            config.queue.redis.port = port;
        }
        if let Some(url) = &self.queue_sqs_url {
            config.queue.sqs.queue_url = url.clone();
        }
        if let Some(region) = &self.queue_sqs_region {
            config.queue.sqs.region = region.clone();
        }

        // Metrics configuration
        if let Some(enabled) = self.metrics_enabled {
            config.metrics.enabled = enabled;
        }
        if let Some(port) = self.metrics_port {
            config.metrics.port = port;
        }
        if let Some(prefix) = &self.metrics_prefix {
            config.metrics.prefix = prefix.clone();
        }

        // SSL configuration
        if let Some(enabled) = self.ssl_enabled {
            config.ssl.enabled = enabled;
        }
        if let Some(cert_path) = &self.ssl_cert_path {
            config.ssl.cert_path = cert_path.clone();
        }
        if let Some(key_path) = &self.ssl_key_path {
            config.ssl.key_path = key_path.clone();
        }

        // CORS configuration
        if let Some(enabled) = self.cors_enabled {
            config.cors.enabled = enabled;
        }
        if let Some(origins) = &self.cors_origins {
            config.cors.origins = origins.split(',').map(|s| s.trim().to_string()).collect();
        }

        // Channel limits
        if let Some(max_length) = self.channel_max_name_length {
            config.channel_limits.max_name_length = max_length;
        }

        // Event limits
        if let Some(max_length) = self.event_max_name_length {
            config.event_limits.max_name_length = max_length;
        }
        if let Some(max_payload) = self.event_max_payload_kb {
            config.event_limits.max_payload_in_kb = max_payload;
        }
        if let Some(max_batch) = self.event_max_batch_size {
            config.event_limits.max_batch_size = max_batch;
        }

        // Presence limits
        if let Some(max_members) = self.presence_max_members {
            config.presence.max_members_per_channel = max_members;
        }
        if let Some(max_size) = self.presence_max_member_size_kb {
            config.presence.max_member_size_in_kb = max_size;
        }

        // HTTP API configuration
        if let Some(max_size) = self.http_max_request_size_kb {
            config.http_api.max_request_size_in_kb = max_size;
        }
        if let Some(threshold) = self.http_memory_threshold_mb {
            config.http_api.accept_traffic_memory_threshold_mb = threshold;
        }

        // User authentication
        if let Some(timeout) = self.user_auth_timeout_ms {
            config.user_authentication_timeout_ms = timeout;
        }
    }
}
