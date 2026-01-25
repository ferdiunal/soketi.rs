use crate::app::PresenceMember;
use crate::error::Result;
use crate::namespace::{Namespace, Socket};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

pub mod cluster;
pub mod horizontal;
pub mod local;
pub mod nats;
pub mod redis;

/// Adapter trait for managing socket connections and message distribution
///
/// The Adapter is responsible for:
/// - Managing socket connections per app and channel
/// - Distributing messages to subscribed sockets
/// - Tracking presence channel members
/// - Managing user authentication and connection tracking
/// - Supporting both single-instance (local) and multi-instance (horizontal) deployments
///
/// Requirements: 4.7, 4.8, 4.9, 4.10
#[async_trait]
pub trait Adapter: Send + Sync {
    /// Initialize the adapter
    /// This is called during server startup to set up any required resources
    async fn init(&self) -> Result<()>;

    // ===== Socket Management Methods =====
    // Requirements: 4.7, 4.8

    /// Add a socket to the adapter for a specific app
    /// Returns Ok(()) on success
    async fn add_socket(&self, app_id: &str, socket: Socket) -> Result<()>;

    /// Remove a socket from the adapter
    /// This should clean up all channel subscriptions and presence memberships
    /// Returns Ok(()) on success
    async fn remove_socket(&self, app_id: &str, socket_id: &str) -> Result<()>;

    /// Get the total number of sockets connected to an app
    async fn get_sockets_count(&self, app_id: &str) -> Result<usize>;

    // ===== Channel Methods =====
    // Requirements: 4.7, 4.8

    /// Get the number of sockets subscribed to a specific channel
    async fn get_channel_sockets_count(&self, app_id: &str, channel: &str) -> Result<usize>;

    /// Get all channels with their socket counts for an app
    /// Returns a map of channel name to socket count
    async fn get_channels_with_sockets_count(&self, app_id: &str)
    -> Result<HashMap<String, usize>>;

    /// Check if a socket is subscribed to a channel
    async fn is_in_channel(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<bool>;

    /// Send a message to all sockets subscribed to a channel
    /// The except_socket_id parameter allows excluding a specific socket (e.g., the sender)
    /// Requirements: 4.5
    async fn send(
        &self,
        app_id: &str,
        channel: &str,
        message: &str,
        except_socket_id: Option<&str>,
    ) -> Result<()>;

    // ===== Presence Channel Methods =====
    // Requirements: 4.10

    /// Add a member to a presence channel
    /// This tracks the member information (user_id and user_info) for the channel
    async fn add_member(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: &str,
        member: PresenceMember,
    ) -> Result<()>;

    /// Remove a member from a presence channel
    async fn remove_member(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<()>;

    /// Get all members of a presence channel
    /// Returns a map of user_id to PresenceMember
    async fn get_channel_members(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, PresenceMember>>;

    /// Get the count of unique members in a presence channel
    async fn get_channel_members_count(&self, app_id: &str, channel: &str) -> Result<usize>;

    // ===== User Tracking Methods =====
    // Requirements: 4.9

    /// Associate a user with a socket
    /// This is used for user authentication tracking
    async fn add_user(&self, app_id: &str, user_id: &str, socket_id: &str) -> Result<()>;

    /// Remove a user association from a socket
    async fn remove_user(&self, app_id: &str, user_id: &str, socket_id: &str) -> Result<()>;

    /// Get all socket IDs associated with a user
    async fn get_user_sockets(&self, app_id: &str, user_id: &str) -> Result<Vec<String>>;

    /// Terminate all connections for a specific user
    /// This should disconnect all sockets associated with the user
    async fn terminate_user_connections(&self, app_id: &str, user_id: &str) -> Result<()>;

    // ===== Lifecycle Methods =====

    /// Disconnect and clean up all adapter resources
    /// This is called during graceful shutdown
    async fn disconnect(&self) -> Result<()>;

    // ===== Legacy/Helper Methods =====
    // These methods are kept for backward compatibility with existing code

    /// Get the namespace for an app (if it exists)
    fn get_namespace(&self, app_id: &str) -> Option<Namespace>;

    /// Add a socket to a channel
    /// Returns the new socket count for the channel
    async fn add_to_channel(&self, app_id: &str, channel: &str, socket_id: String)
    -> Result<usize>;

    /// Remove a socket from a channel
    /// Returns the remaining socket count for the channel
    async fn remove_from_channel(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: &str,
    ) -> Result<usize>;

    /// Get all sockets for an app
    async fn get_sockets(&self, app_id: &str) -> Result<HashMap<String, Socket>>;

    /// Get all channels for an app with their socket IDs
    async fn get_channels(&self, app_id: &str) -> Result<HashMap<String, HashSet<String>>>;

    /// Get all sockets subscribed to a specific channel
    async fn get_channel_sockets(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, Socket>>;

    /// Clear a specific namespace
    async fn clear_namespace(&self, namespace_id: &str) -> Result<()>;

    /// Clear all namespaces
    async fn clear_namespaces(&self) -> Result<()>;

    /// Get a reference to self as Any for downcasting
    /// This is used for adapter-specific operations like close_all_local_sockets
    fn as_any(&self) -> &dyn std::any::Any;
}
