use crate::adapters::{Adapter, local::LocalAdapter};
use crate::app::PresenceMember;
use crate::config::NatsAdapterConfig;
use crate::error::Result;
use crate::namespace::Socket;
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// NatsAdapter provides horizontal scaling using NATS messaging
///
/// This adapter extends LocalAdapter with NATS-based message broadcasting and
/// channel membership synchronization across multiple server instances.
///
/// Requirements: 4.4, 4.5, 4.6
#[derive(Clone)]
pub struct NatsAdapter {
    /// Local adapter for managing local socket connections
    local: LocalAdapter,

    /// NATS client for publishing and subscribing
    nats_client: async_nats::Client,

    /// Unique identifier for this server instance
    node_id: String,

    /// Subject prefix for NATS pub/sub
    subject_prefix: String,

    /// Configuration (kept for future use)
    #[allow(dead_code)]
    config: NatsAdapterConfig,
}

/// NATS message types for adapter operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NatsMessage {
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

impl NatsAdapter {
    /// Create a new NatsAdapter
    ///
    /// # Arguments
    ///
    /// * `config` - NATS adapter configuration
    pub async fn new(config: NatsAdapterConfig) -> Result<Self> {
        let local = LocalAdapter::new();

        // Build NATS connection options
        let mut connect_options = async_nats::ConnectOptions::new();

        // Set authentication if provided
        if let Some(user) = &config.user {
            if let Some(password) = &config.password {
                connect_options = connect_options.user_and_password(user.clone(), password.clone());
            }
        } else if let Some(token) = &config.token {
            connect_options = connect_options.token(token.clone());
        }

        // Set connection timeout
        connect_options =
            connect_options.connection_timeout(std::time::Duration::from_millis(config.timeout_ms));

        // Connect to NATS servers
        let servers: Vec<&str> = config.servers.iter().map(|s| s.as_str()).collect();
        let nats_client = connect_options.connect(&servers[..]).await.map_err(|e| {
            crate::error::PusherError::AdapterError(format!("Failed to connect to NATS: {}", e))
        })?;

        tracing::info!("Connected to NATS servers: {:?}", config.servers);

        let node_id = Uuid::new_v4().to_string();
        let subject_prefix = if config.prefix.is_empty() {
            "pusher-rs".to_string()
        } else {
            config.prefix.clone()
        };

        Ok(Self {
            local,
            nats_client,
            node_id,
            subject_prefix,
            config,
        })
    }

    /// Get the local adapter (for accessing namespaces)
    pub fn get_local_adapter(&self) -> &LocalAdapter {
        &self.local
    }

    /// Get the NATS subject name for pub/sub
    fn get_nats_subject(&self) -> String {
        format!("{}.adapter", self.subject_prefix)
    }

    /// Start the NATS subscription listener
    pub async fn start(&self) -> Result<()> {
        let nats_client = self.nats_client.clone();
        let local = self.local.clone();
        let node_id = self.node_id.clone();
        let nats_subject = self.get_nats_subject();

        tokio::spawn(async move {
            loop {
                match Self::subscribe_and_listen(
                    nats_client.clone(),
                    local.clone(),
                    node_id.clone(),
                    nats_subject.clone(),
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("NATS subscription listener stopped normally");
                        break;
                    }
                    Err(e) => {
                        tracing::error!(
                            "NATS subscription listener error: {}. Reconnecting in 5 seconds...",
                            e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Subscribe to NATS subject and listen for messages
    async fn subscribe_and_listen(
        nats_client: async_nats::Client,
        local: LocalAdapter,
        node_id: String,
        nats_subject: String,
    ) -> Result<()> {
        // Subscribe to the adapter subject
        let mut subscriber = nats_client
            .subscribe(nats_subject.clone())
            .await
            .map_err(|e| {
                crate::error::PusherError::AdapterError(format!(
                    "Failed to subscribe to NATS subject: {}",
                    e
                ))
            })?;

        tracing::info!("Subscribed to NATS subject: {}", nats_subject);

        // Listen for messages
        while let Some(message) = subscriber.next().await {
            let payload = String::from_utf8_lossy(&message.payload).to_string();

            // Parse and handle the message
            match serde_json::from_str::<NatsMessage>(&payload) {
                Ok(nats_msg) => {
                    // Ignore messages from self
                    if Self::is_from_self(&nats_msg, &node_id) {
                        continue;
                    }

                    if let Err(e) = Self::handle_nats_message(&local, nats_msg).await {
                        tracing::error!("Failed to handle NATS message: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse NATS message: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Check if a message is from this node
    fn is_from_self(msg: &NatsMessage, node_id: &str) -> bool {
        match msg {
            NatsMessage::Broadcast {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
            NatsMessage::AddMember {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
            NatsMessage::RemoveMember {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
            NatsMessage::TerminateUser {
                node_id: msg_node_id,
                ..
            } => msg_node_id == node_id,
        }
    }

    /// Handle a NATS message received from another node
    async fn handle_nats_message(local: &LocalAdapter, msg: NatsMessage) -> Result<()> {
        match msg {
            NatsMessage::Broadcast {
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
            NatsMessage::AddMember {
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
            NatsMessage::RemoveMember {
                app_id,
                channel,
                socket_id,
                ..
            } => {
                local.remove_member(&app_id, &channel, &socket_id).await?;
            }
            NatsMessage::TerminateUser {
                app_id, user_id, ..
            } => {
                local.terminate_user_connections(&app_id, &user_id).await?;
            }
        }
        Ok(())
    }

    /// Publish a message to NATS
    async fn publish_to_nats(&self, msg: NatsMessage) -> Result<()> {
        let nats_subject = self.get_nats_subject();
        let payload = serde_json::to_string(&msg).map_err(|e| {
            crate::error::PusherError::AdapterError(format!("Failed to serialize message: {}", e))
        })?;

        self.nats_client
            .publish(nats_subject, payload.into())
            .await
            .map_err(|e| {
                crate::error::PusherError::AdapterError(format!("Failed to publish to NATS: {}", e))
            })?;

        Ok(())
    }
}

#[async_trait]
impl Adapter for NatsAdapter {
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
        // For NATS mode, we only return local socket count
        // To get cluster-wide count, we would need to query all nodes
        self.local.get_sockets_count(app_id).await
    }

    // ===== Channel Methods =====

    async fn get_channel_sockets_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        // For NATS mode, we only return local socket count
        self.local.get_channel_sockets_count(app_id, channel).await
    }

    async fn get_channels_with_sockets_count(
        &self,
        app_id: &str,
    ) -> Result<HashMap<String, usize>> {
        // For NATS mode, we only return local channels
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

        // Publish to NATS for distribution to other nodes
        let msg = NatsMessage::Broadcast {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            message: message.to_string(),
            except_socket_id: except_socket_id.map(|s| s.to_string()),
        };
        self.publish_to_nats(msg).await?;

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

        // Publish to NATS for synchronization
        let msg = NatsMessage::AddMember {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            socket_id: socket_id.to_string(),
            member,
        };
        self.publish_to_nats(msg).await?;

        Ok(())
    }

    async fn remove_member(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<()> {
        // Remove from local adapter
        self.local.remove_member(app_id, channel, socket_id).await?;

        // Publish to NATS for synchronization
        let msg = NatsMessage::RemoveMember {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            socket_id: socket_id.to_string(),
        };
        self.publish_to_nats(msg).await?;

        Ok(())
    }

    async fn get_channel_members(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, PresenceMember>> {
        // For NATS mode, we only return local members
        // To get cluster-wide members, we would need to query all nodes
        self.local.get_channel_members(app_id, channel).await
    }

    async fn get_channel_members_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        // For NATS mode, we only return local member count
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
        // For NATS mode, we only return local user sockets
        self.local.get_user_sockets(app_id, user_id).await
    }

    async fn terminate_user_connections(&self, app_id: &str, user_id: &str) -> Result<()> {
        // Terminate local connections
        self.local
            .terminate_user_connections(app_id, user_id)
            .await?;

        // Publish to NATS to terminate on other nodes
        let msg = NatsMessage::TerminateUser {
            node_id: self.node_id.clone(),
            app_id: app_id.to_string(),
            user_id: user_id.to_string(),
        };
        self.publish_to_nats(msg).await?;

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
