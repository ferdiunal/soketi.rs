use crate::adapters::{Adapter, local::LocalAdapter};
use crate::app::PresenceMember;
use crate::config::RedisAdapterConfig;
use crate::error::Result;
use crate::namespace::Socket;
use async_trait::async_trait;
use futures::StreamExt;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// RedisAdapter provides horizontal scaling using Redis pub/sub
///
/// This adapter extends LocalAdapter with Redis-based message broadcasting and
/// channel membership synchronization across multiple server instances.
///
/// Requirements: 4.3, 4.5, 4.6
#[derive(Clone)]
pub struct RedisAdapter {
    /// Local adapter for managing local socket connections
    local: LocalAdapter,

    /// Redis client for publishing messages
    redis_client: Client,

    /// Unique identifier for this server instance
    node_id: String,

    /// Channel prefix for Redis pub/sub
    channel_prefix: String,

    /// Configuration (kept for future use)
    #[allow(dead_code)]
    config: RedisAdapterConfig,
}

/// Redis message types for adapter operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RedisMessage {
    /// Broadcast a message to a channel
    #[serde(rename = "broadcast")]
    Broadcast {
        node_id: String,
        app_id: String,
        channel: String,
        message: String,
        except_socket_id: Option<String>,
    },

    /// Add a member to a presence channel
    #[serde(rename = "add_member")]
    AddMember {
        node_id: String,
        app_id: String,
        channel: String,
        socket_id: String,
        member: PresenceMember,
    },

    /// Remove a member from a presence channel
    #[serde(rename = "remove_member")]
    RemoveMember {
        node_id: String,
        app_id: String,
        channel: String,
        socket_id: String,
    },

    /// Terminate user connections
    #[serde(rename = "terminate_user")]
    TerminateUser {
        node_id: String,
        app_id: String,
        user_id: String,
    },
}

impl RedisAdapter {
    /// Create a new RedisAdapter
    ///
    /// # Arguments
    ///
    /// * `config` - Redis adapter configuration
    pub async fn new(config: RedisAdapterConfig) -> Result<Self> {
        let local = LocalAdapter::new();

        // Build Redis connection URL
        let redis_url = if let Some(password) = &config.password {
            if let Some(username) = &config.username {
                format!(
                    "redis://{}:{}@{}:{}/{}",
                    username, password, config.host, config.port, config.db
                )
            } else {
                format!(
                    "redis://:{}@{}:{}/{}",
                    password, config.host, config.port, config.db
                )
            }
        } else {
            format!("redis://{}:{}/{}", config.host, config.port, config.db)
        };

        let redis_client = Client::open(redis_url).map_err(|e| {
            crate::error::PusherError::AdapterError(format!("Failed to create Redis client: {}", e))
        })?;

        // Test connection
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                crate::error::PusherError::AdapterError(format!(
                    "Failed to connect to Redis: {}",
                    e
                ))
            })?;

        // Ping to verify connection
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                crate::error::PusherError::AdapterError(format!("Redis ping failed: {}", e))
            })?;

        let node_id = Uuid::new_v4().to_string();
        let channel_prefix = if config.key_prefix.is_empty() {
            "pusher-rs".to_string()
        } else {
            config.key_prefix.clone()
        };

        Ok(Self {
            local,
            redis_client,
            node_id,
            channel_prefix,
            config,
        })
    }

    /// Get the local adapter (for accessing namespaces)
    pub fn get_local_adapter(&self) -> &LocalAdapter {
        &self.local
    }

    /// Get the Redis channel name for pub/sub
    fn get_redis_channel(&self) -> String {
        format!("{}:adapter", self.channel_prefix)
    }

    /// Start the Redis pub/sub listener
    pub async fn start(&self) -> Result<()> {
        let redis_client = self.redis_client.clone();
        let local = self.local.clone();
        let node_id = self.node_id.clone();
        let redis_channel = self.get_redis_channel();

        tokio::spawn(async move {
            loop {
                match Self::subscribe_and_listen(
                    redis_client.clone(),
                    local.clone(),
                    node_id.clone(),
                    redis_channel.clone(),
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("Redis pub/sub listener stopped normally");
                        break;
                    }
                    Err(e) => {
                        tracing::error!(
                            "Redis pub/sub listener error: {}. Reconnecting in 5 seconds...",
                            e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Subscribe to Redis channel and listen for messages
    async fn subscribe_and_listen(
        redis_client: Client,
        local: LocalAdapter,
        node_id: String,
        redis_channel: String,
    ) -> Result<()> {
        // Get a connection for pub/sub
        let conn = redis_client.get_async_pubsub().await.map_err(|e| {
            crate::error::PusherError::AdapterError(format!(
                "Failed to get Redis connection: {}",
                e
            ))
        })?;

        let mut pubsub = conn;

        // Subscribe to the adapter channel
        pubsub.subscribe(&redis_channel).await.map_err(|e| {
            crate::error::PusherError::AdapterError(format!(
                "Failed to subscribe to Redis channel: {}",
                e
            ))
        })?;

        tracing::info!("Subscribed to Redis channel: {}", redis_channel);

        // Listen for messages
        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("Failed to get message payload: {}", e);
                    continue;
                }
            };

            // Parse and handle the message
            match serde_json::from_str::<RedisMessage>(&payload) {
                Ok(redis_msg) => {
                    // Ignore messages from self
                    if Self::is_from_self(&redis_msg, &node_id) {
                        continue;
                    }

                    if let Err(e) = Self::handle_redis_message(&local, redis_msg).await {
                        tracing::error!("Failed to handle Redis message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse Redis message: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Check if a message is from this node
    fn is_from_self(msg: &RedisMessage, node_id: &str) -> bool {
        match msg {
            RedisMessage::Broadcast {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
            RedisMessage::AddMember {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
            RedisMessage::RemoveMember {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
            RedisMessage::TerminateUser {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
        }
    }

    /// Handle a Redis message received from another node
    async fn handle_redis_message(local: &LocalAdapter, msg: RedisMessage) -> Result<()> {
        match msg {
            RedisMessage::Broadcast {
                app_id,
                channel,
                message,
                except_socket_id,
                ..
            } => {
                local
                    .send(&app_id, &channel, &message, except_socket_id.as_deref())
                    .await?;
            }
            RedisMessage::AddMember {
                app_id,
                channel,
                socket_id,
                member,
                ..
            } => {
                local
                    .add_member(&app_id, &channel, &socket_id, member)
                    .await?;
            }
            RedisMessage::RemoveMember {
                app_id,
                channel,
                socket_id,
                ..
            } => {
                local.remove_member(&app_id, &channel, &socket_id).await?;
            }
            RedisMessage::TerminateUser {
                app_id, user_id, ..
            } => {
                local.terminate_user_connections(&app_id, &user_id).await?;
            }
        }
        Ok(())
    }

    /// Publish a message to Redis
    async fn publish_to_redis(&self, msg: RedisMessage) -> Result<()> {
        let redis_channel = self.get_redis_channel();
        let payload = serde_json::to_string(&msg).map_err(|e| {
            crate::error::PusherError::AdapterError(format!("Failed to serialize message: {}", e))
        })?;

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                crate::error::PusherError::AdapterError(format!(
                    "Failed to get Redis connection: {}",
                    e
                ))
            })?;

        conn.publish::<_, _, ()>(&redis_channel, payload)
            .await
            .map_err(|e| {
                crate::error::PusherError::AdapterError(format!(
                    "Failed to publish to Redis: {}",
                    e
                ))
            })?;

        Ok(())
    }
}

#[async_trait]
impl Adapter for RedisAdapter {
    async fn init(&self) -> Result<()> {
        self.local.init().await?;
        self.start().await?;
        Ok(())
    }

    fn get_namespace(&self, app_id: &str) -> Option<crate::namespace::Namespace> {
        self.local.get_namespace(app_id)
    }

    // ===== Socket Management Methods =====

    async fn add_socket(&self, app_id: &str, socket: Socket) -> Result<()> {
        self.local.add_socket(app_id, socket).await
    }

    async fn remove_socket(&self, app_id: &str, socket_id: &str) -> Result<()> {
        self.local.remove_socket(app_id, socket_id).await
    }

    async fn get_sockets_count(&self, app_id: &str) -> Result<usize> {
        // For Redis mode, we only return local socket count
        // To get cluster-wide count, we would need to query all nodes
        self.local.get_sockets_count(app_id).await
    }

    // ===== Channel Methods =====

    async fn get_channel_sockets_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        // For Redis mode, we only return local socket count
        self.local.get_channel_sockets_count(app_id, channel).await
    }

    async fn get_channels_with_sockets_count(
        &self,
        app_id: &str,
    ) -> Result<HashMap<String, usize>> {
        // For Redis mode, we only return local channels
        self.local.get_channels_with_sockets_count(app_id).await
    }

    async fn is_in_channel(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<bool> {
        self.local.is_in_channel(app_id, channel, socket_id).await
    }

    async fn send(
        &self,
        app_id: &str,
        channel: &str,
        message: &str,
        except_socket_id: Option<&str>,
    ) -> Result<()> {
        // Send to local sockets
        self.local
            .send(app_id, channel, message, except_socket_id)
            .await?;

        // Publish to Redis for distribution to other nodes
        let msg = RedisMessage::Broadcast {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            message: message.to_string(),
            except_socket_id: except_socket_id.map(|s| s.to_string()),
        };
        self.publish_to_redis(msg).await?;

        Ok(())
    }

    // ===== Presence Channel Methods =====

    async fn add_member(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: &str,
        member: PresenceMember,
    ) -> Result<()> {
        // Add to local adapter
        self.local
            .add_member(app_id, channel, socket_id, member.clone())
            .await?;

        // Publish to Redis for synchronization
        let msg = RedisMessage::AddMember {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            socket_id: socket_id.to_string(),
            member,
        };
        self.publish_to_redis(msg).await?;

        Ok(())
    }

    async fn remove_member(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<()> {
        // Remove from local adapter
        self.local.remove_member(app_id, channel, socket_id).await?;

        // Publish to Redis for synchronization
        let msg = RedisMessage::RemoveMember {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            socket_id: socket_id.to_string(),
        };
        self.publish_to_redis(msg).await?;

        Ok(())
    }

    async fn get_channel_members(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, PresenceMember>> {
        // For Redis mode, we only return local members
        // To get cluster-wide members, we would need to query all nodes
        self.local.get_channel_members(app_id, channel).await
    }

    async fn get_channel_members_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        // For Redis mode, we only return local member count
        self.local.get_channel_members_count(app_id, channel).await
    }

    // ===== User Tracking Methods =====

    async fn add_user(&self, app_id: &str, user_id: &str, socket_id: &str) -> Result<()> {
        self.local.add_user(app_id, user_id, socket_id).await
    }

    async fn remove_user(&self, app_id: &str, user_id: &str, socket_id: &str) -> Result<()> {
        self.local.remove_user(app_id, user_id, socket_id).await
    }

    async fn get_user_sockets(&self, app_id: &str, user_id: &str) -> Result<Vec<String>> {
        // For Redis mode, we only return local user sockets
        self.local.get_user_sockets(app_id, user_id).await
    }

    async fn terminate_user_connections(&self, app_id: &str, user_id: &str) -> Result<()> {
        // Terminate local connections
        self.local
            .terminate_user_connections(app_id, user_id)
            .await?;

        // Publish to Redis to terminate on other nodes
        let msg = RedisMessage::TerminateUser {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            user_id: user_id.to_string(),
        };
        self.publish_to_redis(msg).await?;

        Ok(())
    }

    // ===== Lifecycle Methods =====

    async fn disconnect(&self) -> Result<()> {
        // Disconnect local adapter
        self.local.disconnect().await
    }

    // ===== Legacy/Helper Methods =====

    async fn add_to_channel(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: String,
    ) -> Result<usize> {
        self.local.add_to_channel(app_id, channel, socket_id).await
    }

    async fn remove_from_channel(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: &str,
    ) -> Result<usize> {
        self.local
            .remove_from_channel(app_id, channel, socket_id)
            .await
    }

    async fn get_sockets(&self, app_id: &str) -> Result<HashMap<String, Socket>> {
        self.local.get_sockets(app_id).await
    }

    async fn get_channels(&self, app_id: &str) -> Result<HashMap<String, HashSet<String>>> {
        self.local.get_channels(app_id).await
    }

    async fn get_channel_sockets(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, Socket>> {
        self.local.get_channel_sockets(app_id, channel).await
    }

    async fn clear_namespace(&self, namespace_id: &str) -> Result<()> {
        self.local.clear_namespace(namespace_id).await
    }

    async fn clear_namespaces(&self) -> Result<()> {
        self.local.clear_namespaces().await
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
