use crate::adapters::Adapter;
use crate::app::App;
use crate::channels::{ChannelManager, JoinResponse, PusherMessage};
use crate::namespace::Socket;
use async_trait::async_trait;
use std::sync::Arc;

/// PublicChannelManager handles subscriptions to public channels
///
/// Public channels are the simplest channel type and do not require authentication.
/// Any client can subscribe to a public channel without providing credentials.
///
/// Requirements: 7.1
pub struct PublicChannelManager {
    adapter: Arc<dyn Adapter>,
}

impl PublicChannelManager {
    pub fn new(adapter: Arc<dyn Adapter>) -> Self {
        Self { adapter }
    }

    /// Join a public channel
    ///
    /// This method validates the channel name and adds the socket to the channel
    /// via the adapter. Public channels do not require authentication.
    ///
    /// # Arguments
    /// * `app` - The application the socket belongs to
    /// * `socket` - The socket attempting to join
    /// * `channel` - The channel name to join
    /// * `_message` - The subscription message (not used for public channels)
    ///
    /// # Returns
    /// A JoinResponse indicating success or failure
    ///
    /// Requirements: 7.1
    pub async fn join(
        &self,
        app: &App,
        socket: &Socket,
        channel: &str,
        _message: Option<PusherMessage>,
    ) -> JoinResponse {
        // Validate channel name is not a restricted type
        // Public channels cannot start with "private-", "presence-", or "private-encrypted-"
        if channel.starts_with("private-")
            || channel.starts_with("presence-")
            || channel.starts_with("private-encrypted-")
        {
            return JoinResponse {
                success: false,
                error_code: Some(4009),
                error_message: Some(format!("Channel '{}' requires authentication", channel)),
                auth_error: true,
                type_: Some("AuthError".to_string()),
            };
        }

        // Add socket to channel via adapter
        match self
            .adapter
            .add_to_channel(&app.id, channel, socket.id.clone())
            .await
        {
            Ok(_) => JoinResponse {
                success: true,
                error_code: None,
                error_message: None,
                auth_error: false,
                type_: None,
            },
            Err(e) => JoinResponse {
                success: false,
                error_code: Some(4302),
                error_message: Some(format!("Failed to join channel: {}", e)),
                auth_error: false,
                type_: Some("ServerError".to_string()),
            },
        }
    }

    /// Leave a public channel
    ///
    /// This method removes the socket from the channel via the adapter.
    ///
    /// # Arguments
    /// * `app_id` - The application ID
    /// * `socket_id` - The socket ID to remove
    /// * `channel` - The channel name to leave
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// Requirements: 7.1
    pub async fn leave(
        &self,
        app_id: &str,
        socket_id: &str,
        channel: &str,
    ) -> crate::error::Result<()> {
        // Remove socket from channel via adapter
        self.adapter
            .remove_from_channel(app_id, channel, socket_id)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl ChannelManager for PublicChannelManager {
    async fn join(
        &self,
        app: &App,
        socket: &Socket,
        channel: &str,
        message: Option<PusherMessage>,
    ) -> JoinResponse {
        self.join(app, socket, channel, message).await
    }
}
