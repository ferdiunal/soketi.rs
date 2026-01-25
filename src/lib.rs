//! # Soketi-RS: Pusher Protocol Server in Rust
//!
//! A high-performance, feature-complete implementation of the Pusher protocol server in Rust.
//!
//! ## Overview
//!
//! Soketi-RS provides a complete implementation of the Pusher WebSocket protocol with support for:
//! - WebSocket connections with full Pusher protocol support
//! - HTTP REST API for triggering events and querying channels
//! - Multiple adapter types for horizontal scaling
//! - Flexible app management with multiple backend options
//! - Caching, rate limiting, and queue processing
//! - Webhook delivery with HTTP and AWS Lambda support
//! - Prometheus metrics export
//!
//! ## Quick Start
//!
//! ```no_run
//! use soketi_rs::server::Server;
//! use soketi_rs::config::ServerConfig;
//! use soketi_rs::app::App;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration
//!     let mut config = ServerConfig::default();
//!     
//!     // Add an app
//!     let app = App::new(
//!         "app-id".to_string(),
//!         "app-key".to_string(),
//!         "app-secret".to_string(),
//!     );
//!     config.app_manager.array.apps = vec![app];
//!     
//!     // Create and initialize server
//!     let mut server = Server::new(config);
//!     server.initialize().await?;
//!     
//!     // Start the server
//!     server.start().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The server is built around several key components:
//!
//! ### Core Components
//!
//! - **Server**: Main entry point that initializes and manages all components
//! - **AppState**: Shared state containing all managers and configuration
//! - **WsHandler**: Handles WebSocket connections and Pusher protocol messages
//! - **HttpHandler**: Handles REST API requests
//!
//! ### Managers
//!
//! - **Adapter**: Manages socket connections and message distribution
//!   - Local: Single-instance deployment
//!   - Cluster: Multi-instance with UDP discovery
//!   - Redis: Horizontal scaling with Redis pub/sub
//!   - NATS: Horizontal scaling with NATS messaging
//!
//! - **AppManager**: Manages application configurations
//!   - Array: Static configuration
//!   - DynamoDB: AWS DynamoDB backend
//!   - MySQL: MySQL database backend
//!   - PostgreSQL: PostgreSQL database backend
//!
//! - **CacheManager**: Provides caching functionality
//!   - Memory: In-memory caching
//!   - Redis: Distributed caching with Redis
//!
//! - **RateLimiter**: Enforces rate limits
//!   - Local: Single-instance rate limiting
//!   - Cluster: Cluster-wide rate limiting
//!   - Redis: Distributed rate limiting with Redis
//!
//! - **QueueManager**: Manages webhook job processing
//!   - Sync: Immediate processing
//!   - Redis: BullMQ-compatible queue
//!   - SQS: AWS SQS queue
//!
//! - **MetricsManager**: Collects and exposes metrics
//!   - Prometheus: Prometheus metrics export
//!
//! - **WebhookSender**: Sends webhook notifications
//!   - HTTP webhooks
//!   - AWS Lambda invocation
//!
//! ## Channel Types
//!
//! Soketi-RS supports all Pusher channel types:
//!
//! - **Public channels**: No authentication required
//! - **Private channels**: Require authentication signature
//! - **Encrypted private channels**: End-to-end encryption
//! - **Presence channels**: Track channel members
//!
//! ## Configuration
//!
//! Configuration can be loaded from multiple sources:
//!
//! 1. Command-line arguments (highest priority)
//! 2. Environment variables (with `PUSHER_` prefix)
//! 3. Configuration files (JSON, YAML, TOML)
//! 4. Default values (lowest priority)
//!
//! See the [`options`] module for configuration loading and the [`config`] module
//! for configuration structures.
//!
//! ## Examples
//!
//! ### Basic Server Setup
//!
//! ```no_run
//! use soketi_rs::server::Server;
//! use soketi_rs::options::Options;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from all sources
//!     let config = Options::load()?;
//!     
//!     // Create and initialize server
//!     let mut server = Server::new(config);
//!     server.initialize().await?;
//!     
//!     // Start the server
//!     server.start().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Configuration
//!
//! ```no_run
//! use soketi_rs::server::Server;
//! use soketi_rs::config::{ServerConfig, AdapterDriver, CacheDriver};
//! use soketi_rs::app::App;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut config = ServerConfig::default();
//!     
//!     // Configure server
//!     config.host = "0.0.0.0".to_string();
//!     config.port = 6001;
//!     config.debug = true;
//!     
//!     // Use Redis adapter for horizontal scaling
//!     config.adapter.driver = AdapterDriver::Redis;
//!     config.adapter.redis.host = "redis.example.com".to_string();
//!     
//!     // Use Redis cache
//!     config.cache.driver = CacheDriver::Redis;
//!     
//!     // Add apps
//!     let app = App::new(
//!         "app-1".to_string(),
//!         "key-1".to_string(),
//!         "secret-1".to_string(),
//!     );
//!     config.app_manager.array.apps = vec![app];
//!     
//!     // Create and start server
//!     let mut server = Server::new(config);
//!     server.initialize().await?;
//!     server.start().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! This crate does not currently use feature flags, but all functionality is included by default.
//!
//! ## Requirements
//!
//! This implementation validates against the following requirements:
//! - Requirements 1.1-1.7: Core server architecture
//! - Requirements 2.1-2.10: WebSocket handler
//! - Requirements 3.1-3.14: HTTP REST API
//! - Requirements 4.1-4.10: Adapter system
//! - Requirements 5.1-5.9: App manager system
//! - Requirements 6.1-6.6: Cache manager system
//! - Requirements 7.1-7.10: Channel types
//! - Requirements 8.1-8.8: Rate limiting
//! - Requirements 9.1-9.6: Queue system
//! - Requirements 10.1-10.10: Webhook sender
//! - Requirements 11.1-11.7: Metrics system
//! - Requirements 12.1-12.7: Authentication and security
//! - Requirements 13.1-13.9: Error handling
//! - Requirements 14.1-14.10: Configuration
//! - Requirements 15.1-15.7: Client event handling
//! - Requirements 16.1-16.6: User authentication
//! - Requirements 17.1-17.5: Cluster discovery
//! - Requirements 18.1-18.4: Database connection pooling
//! - Requirements 19.1-19.7: Message validation
//! - Requirements 20.1-20.7: Logging and debugging

pub mod adapters;
pub mod api;
pub mod app;
pub mod app_managers;
pub mod auth;
pub mod cache_managers;
pub mod channels;
pub mod config;
pub mod error;
pub mod log;
pub mod metrics;
pub mod namespace;
pub mod options;
pub mod pusher;
pub mod queues;
pub mod rate_limiters;
pub mod server;
pub mod state;
pub mod validation;
pub mod webhook_sender;
pub mod ws;
pub mod ws_handler;
