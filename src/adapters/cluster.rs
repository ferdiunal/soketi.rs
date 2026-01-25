use crate::adapters::{Adapter, local::LocalAdapter};
use crate::app::PresenceMember;
use crate::config::ClusterAdapterConfig;
use crate::error::Result;
use crate::namespace::Socket;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, broadcast};
use tokio::time::interval;
use uuid::Uuid;

/// ClusterAdapter provides multi-instance deployment support using UDP discovery
///
/// This adapter extends LocalAdapter with cluster-wide message broadcasting and
/// node discovery capabilities. It uses UDP multicast for node discovery and
/// message distribution across cluster nodes.
///
/// Requirements: 4.2, 4.5, 4.6, 17.1, 17.2, 17.3, 17.4, 17.5
#[derive(Clone)]
pub struct ClusterAdapter {
    /// Local adapter for managing local socket connections
    local: LocalAdapter,

    /// Node discovery manager
    discovery: Arc<Mutex<Discovery>>,

    /// Broadcast channel for cluster messages
    broadcast_tx: broadcast::Sender<ClusterMessage>,

    /// Configuration (kept for future use)
    #[allow(dead_code)]
    config: ClusterAdapterConfig,
}

/// Discovery manages UDP-based node discovery and master election
struct Discovery {
    /// This node's unique identifier
    node_id: String,

    /// This node's address
    local_address: SocketAddr,

    /// Map of discovered nodes (node_id -> NodeInfo)
    nodes: HashMap<String, NodeInfo>,

    /// Current master node ID
    master_id: Option<String>,

    /// Whether this node is the master
    is_master: bool,

    /// UDP socket for discovery
    socket: Arc<UdpSocket>,

    /// Configuration
    config: ClusterAdapterConfig,
}

/// Information about a discovered node
#[derive(Debug, Clone)]
struct NodeInfo {
    node_id: String,
    address: SocketAddr,
    last_seen: SystemTime,
    weight: u64, // Used for master election
}

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum DiscoveryMessageType {
    Hello,
    Heartbeat,
    Goodbye,
}

/// Discovery message sent over UDP
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    message_type: DiscoveryMessageType,
    node_id: String,
    address: String,
    weight: u64,
    timestamp: u64,
}

/// Cluster message types for adapter operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterMessage {
    /// Broadcast a message to a channel
    Broadcast {
        app_id: String,
        channel: String,
        message: String,
        except_socket_id: Option<String>,
    },

    /// Add a member to a presence channel
    AddMember {
        app_id: String,
        channel: String,
        socket_id: String,
        member: PresenceMember,
    },

    /// Remove a member from a presence channel
    RemoveMember {
        app_id: String,
        channel: String,
        socket_id: String,
    },

    /// Terminate user connections
    TerminateUser { app_id: String, user_id: String },
}

impl ClusterAdapter {
    /// Create a new ClusterAdapter
    ///
    /// # Arguments
    ///
    /// * `config` - Cluster adapter configuration
    pub async fn new(config: ClusterAdapterConfig) -> Result<Self> {
        let local = LocalAdapter::new();

        // Create broadcast channel for cluster messages
        let (broadcast_tx, _) = broadcast::channel(1000);

        // Create discovery manager
        let node_id = Uuid::new_v4().to_string();
        let discovery = Discovery::new(node_id, config.clone()).await?;

        Ok(Self {
            local,
            discovery: Arc::new(Mutex::new(discovery)),
            broadcast_tx,
            config,
        })
    }

    /// Get the local adapter (for accessing namespaces)
    pub fn get_local_adapter(&self) -> &LocalAdapter {
        &self.local
    }

    /// Start the discovery and message handling tasks
    pub async fn start(&self) -> Result<()> {
        // Start discovery heartbeat task
        let discovery = Arc::clone(&self.discovery);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                if let Ok(mut disc) = discovery.try_lock() {
                    let _ = disc.send_heartbeat().await;
                    disc.check_node_timeouts();
                    disc.check_master_election();
                }
            }
        });

        // Start discovery message receiver task
        let discovery = Arc::clone(&self.discovery);
        tokio::spawn(async move {
            loop {
                if let Ok(mut disc) = discovery.try_lock()
                    && let Ok(_) = disc.receive_discovery_message().await
                {
                    // Message processed
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        // Start cluster message receiver task
        let mut rx = self.broadcast_tx.subscribe();
        let local = self.local.clone();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let _ = Self::handle_cluster_message(&local, msg).await;
            }
        });

        // Send initial hello message
        let mut disc = self.discovery.lock().await;
        disc.send_hello().await?;

        Ok(())
    }

    /// Handle a cluster message received from another node
    async fn handle_cluster_message(local: &LocalAdapter, msg: ClusterMessage) -> Result<()> {
        match msg {
            ClusterMessage::Broadcast {
                app_id,
                channel,
                message,
                except_socket_id,
            } => {
                local
                    .send(&app_id, &channel, &message, except_socket_id.as_deref())
                    .await?;
            }
            ClusterMessage::AddMember {
                app_id,
                channel,
                socket_id,
                member,
            } => {
                local
                    .add_member(&app_id, &channel, &socket_id, member)
                    .await?;
            }
            ClusterMessage::RemoveMember {
                app_id,
                channel,
                socket_id,
            } => {
                local.remove_member(&app_id, &channel, &socket_id).await?;
            }
            ClusterMessage::TerminateUser { app_id, user_id } => {
                local.terminate_user_connections(&app_id, &user_id).await?;
            }
        }
        Ok(())
    }

    /// Broadcast a cluster message to all nodes
    async fn broadcast_cluster_message(&self, msg: ClusterMessage) -> Result<()> {
        // Send to local broadcast channel
        let _ = self.broadcast_tx.send(msg.clone());

        // Send to other nodes via UDP
        let discovery = self.discovery.lock().await;
        let serialized = serde_json::to_string(&msg)?;

        for node in discovery.nodes.values() {
            if node.node_id != discovery.node_id {
                let _ = discovery
                    .socket
                    .send_to(serialized.as_bytes(), node.address)
                    .await;
            }
        }

        Ok(())
    }

    /// Check if this node is the master
    pub async fn is_master(&self) -> bool {
        let discovery = self.discovery.lock().await;
        discovery.is_master
    }

    /// Get the current master node ID
    pub async fn get_master_id(&self) -> Option<String> {
        let discovery = self.discovery.lock().await;
        discovery.master_id.clone()
    }

    /// Get all discovered nodes
    pub async fn get_nodes(&self) -> Vec<String> {
        let discovery = self.discovery.lock().await;
        discovery.nodes.keys().cloned().collect()
    }
}

impl Discovery {
    /// Create a new Discovery instance
    async fn new(node_id: String, config: ClusterAdapterConfig) -> Result<Self> {
        // Bind to any address on the specified port
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);

        let socket = UdpSocket::bind(socket_addr).await?;

        // Set socket options for multicast
        socket.set_broadcast(true)?;

        // Join multicast group
        let multicast_addr: Ipv4Addr = config.multicast_address.parse().map_err(|_| {
            crate::error::PusherError::AdapterError("Invalid multicast address".to_string())
        })?;

        // Try to join multicast group, but don't fail if it doesn't work
        // This allows the adapter to work in environments where multicast is not available
        if let Err(e) = socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED) {
            tracing::warn!(
                "Failed to join multicast group: {}. Cluster discovery may not work properly.",
                e
            );
        }

        let local_address = socket.local_addr()?;

        Ok(Self {
            node_id: node_id.clone(),
            local_address,
            nodes: HashMap::new(),
            master_id: None,
            is_master: false,
            socket: Arc::new(socket),
            config,
        })
    }

    /// Send a hello message to announce this node
    async fn send_hello(&mut self) -> Result<()> {
        let msg = DiscoveryMessage {
            message_type: DiscoveryMessageType::Hello,
            node_id: self.node_id.clone(),
            address: self.local_address.to_string(),
            weight: self.calculate_weight(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.send_discovery_message(msg).await
    }

    /// Send a heartbeat message
    async fn send_heartbeat(&mut self) -> Result<()> {
        let msg = DiscoveryMessage {
            message_type: DiscoveryMessageType::Heartbeat,
            node_id: self.node_id.clone(),
            address: self.local_address.to_string(),
            weight: self.calculate_weight(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.send_discovery_message(msg).await
    }

    /// Send a goodbye message when shutting down
    async fn send_goodbye(&mut self) -> Result<()> {
        let msg = DiscoveryMessage {
            message_type: DiscoveryMessageType::Goodbye,
            node_id: self.node_id.clone(),
            address: self.local_address.to_string(),
            weight: self.calculate_weight(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.send_discovery_message(msg).await
    }

    /// Send a discovery message to the multicast group
    async fn send_discovery_message(&self, msg: DiscoveryMessage) -> Result<()> {
        let serialized = serde_json::to_string(&msg)?;
        let multicast_addr: Ipv4Addr = self.config.multicast_address.parse().map_err(|_| {
            crate::error::PusherError::AdapterError("Invalid multicast address".to_string())
        })?;
        let target = SocketAddr::new(IpAddr::V4(multicast_addr), self.config.port);

        // Try to send, but don't fail if it doesn't work
        // This allows the adapter to work in environments where multicast is not available
        match self.socket.send_to(serialized.as_bytes(), target).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::debug!(
                    "Failed to send discovery message: {}. This is expected if multicast is not available.",
                    e
                );
                Ok(())
            }
        }
    }

    /// Receive and process a discovery message
    async fn receive_discovery_message(&mut self) -> Result<()> {
        let mut buf = vec![0u8; 65536];

        if let Ok(Ok((len, addr))) =
            tokio::time::timeout(Duration::from_millis(100), self.socket.recv_from(&mut buf)).await
        {
            let data = &buf[..len];
            if let Ok(msg) = serde_json::from_slice::<DiscoveryMessage>(data) {
                self.handle_discovery_message(msg, addr).await?;
            }
        }

        Ok(())
    }

    /// Handle a received discovery message
    async fn handle_discovery_message(
        &mut self,
        msg: DiscoveryMessage,
        addr: SocketAddr,
    ) -> Result<()> {
        // Ignore messages from self
        if msg.node_id == self.node_id {
            return Ok(());
        }

        match msg.message_type {
            DiscoveryMessageType::Hello | DiscoveryMessageType::Heartbeat => {
                // Parse the address from the message
                let node_addr = msg.address.parse::<SocketAddr>().unwrap_or(addr);

                let node_info = NodeInfo {
                    node_id: msg.node_id.clone(),
                    address: node_addr,
                    last_seen: SystemTime::now(),
                    weight: msg.weight,
                };

                let is_new = !self.nodes.contains_key(&msg.node_id);
                self.nodes.insert(msg.node_id.clone(), node_info);

                if is_new {
                    tracing::info!("New node discovered: {}", msg.node_id);
                    // Trigger master election
                    self.check_master_election();
                }
            }
            DiscoveryMessageType::Goodbye => {
                if self.nodes.remove(&msg.node_id).is_some() {
                    tracing::info!("Node left: {}", msg.node_id);
                    // Trigger master election if the master left
                    if self.master_id.as_ref() == Some(&msg.node_id) {
                        self.check_master_election();
                    }
                }
            }
        }

        Ok(())
    }

    /// Check for node timeouts and remove stale nodes
    fn check_node_timeouts(&mut self) {
        let timeout = Duration::from_secs(15);
        let now = SystemTime::now();

        let mut to_remove = Vec::new();
        for (node_id, node_info) in &self.nodes {
            if let Ok(elapsed) = now.duration_since(node_info.last_seen)
                && elapsed > timeout
            {
                to_remove.push(node_id.clone());
            }
        }

        for node_id in to_remove {
            self.nodes.remove(&node_id);
            tracing::info!("Node timed out: {}", node_id);

            // Trigger master election if the master timed out
            if self.master_id.as_ref() == Some(&node_id) {
                self.check_master_election();
            }
        }
    }

    /// Perform master election
    /// The node with the highest weight becomes the master
    /// In case of a tie, the node with the lexicographically smallest ID wins
    fn check_master_election(&mut self) {
        let mut candidates: Vec<(String, u64)> = self
            .nodes
            .iter()
            .map(|(id, info)| (id.clone(), info.weight))
            .collect();

        // Add self to candidates
        candidates.push((self.node_id.clone(), self.calculate_weight()));

        // Sort by weight (descending), then by node_id (ascending)
        candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        if let Some((new_master_id, _)) = candidates.first() {
            let old_master_id = self.master_id.clone();
            let new_is_master = new_master_id == &self.node_id;

            if old_master_id.as_ref() != Some(new_master_id) {
                tracing::info!("Master election: {} is now master", new_master_id);

                if self.is_master && !new_is_master {
                    tracing::info!("This node was demoted from master");
                } else if !self.is_master && new_is_master {
                    tracing::info!("This node was promoted to master");
                }

                self.master_id = Some(new_master_id.clone());
                self.is_master = new_is_master;
            }
        }
    }

    /// Calculate this node's weight for master election
    /// Higher weight = higher priority to become master
    fn calculate_weight(&self) -> u64 {
        // Use a combination of factors:
        // - Uptime (nodes that have been up longer have higher weight)
        // - Node ID hash (for deterministic tie-breaking)

        // Simple weight calculation
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[async_trait]
impl Adapter for ClusterAdapter {
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
        // For cluster mode, we only return local socket count
        // To get cluster-wide count, we would need to query all nodes
        self.local.get_sockets_count(app_id).await
    }

    // ===== Channel Methods =====

    async fn get_channel_sockets_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        // For cluster mode, we only return local socket count
        self.local.get_channel_sockets_count(app_id, channel).await
    }

    async fn get_channels_with_sockets_count(
        &self,
        app_id: &str,
    ) -> Result<HashMap<String, usize>> {
        // For cluster mode, we only return local channels
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

        // Broadcast to cluster
        let msg = ClusterMessage::Broadcast {
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            message: message.to_string(),
            except_socket_id: except_socket_id.map(|s| s.to_string()),
        };
        self.broadcast_cluster_message(msg).await?;

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

        // Broadcast to cluster
        let msg = ClusterMessage::AddMember {
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            socket_id: socket_id.to_string(),
            member,
        };
        self.broadcast_cluster_message(msg).await?;

        Ok(())
    }

    async fn remove_member(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<()> {
        // Remove from local adapter
        self.local.remove_member(app_id, channel, socket_id).await?;

        // Broadcast to cluster
        let msg = ClusterMessage::RemoveMember {
            app_id: app_id.to_string(),
            channel: channel.to_string(),
            socket_id: socket_id.to_string(),
        };
        self.broadcast_cluster_message(msg).await?;

        Ok(())
    }

    async fn get_channel_members(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, PresenceMember>> {
        // For cluster mode, we only return local members
        // To get cluster-wide members, we would need to query all nodes
        self.local.get_channel_members(app_id, channel).await
    }

    async fn get_channel_members_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        // For cluster mode, we only return local member count
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
        // For cluster mode, we only return local user sockets
        self.local.get_user_sockets(app_id, user_id).await
    }

    async fn terminate_user_connections(&self, app_id: &str, user_id: &str) -> Result<()> {
        // Terminate local connections
        self.local
            .terminate_user_connections(app_id, user_id)
            .await?;

        // Broadcast to cluster
        let msg = ClusterMessage::TerminateUser {
            app_id: app_id.to_string(),
            user_id: user_id.to_string(),
        };
        self.broadcast_cluster_message(msg).await?;

        Ok(())
    }

    // ===== Lifecycle Methods =====

    async fn disconnect(&self) -> Result<()> {
        // Send goodbye message
        let mut discovery = self.discovery.lock().await;
        let _ = discovery.send_goodbye().await;

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
