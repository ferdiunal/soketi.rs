use crate::adapters::Adapter;
use crate::app::App;
use crate::channels::private::PrivateChannelManager;
use crate::channels::{ChannelManager, JoinResponse, PusherMessage};
use crate::namespace::Socket;
use async_trait::async_trait;
use hex;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use std::sync::Arc;

/// PresenceChannelManager handles subscriptions to presence channels
///
/// Presence channels (channels starting with "presence-") track member information
/// and broadcast member_added and member_removed events when members join or leave.
///
/// Requirements: 7.4, 7.6, 7.7, 7.8
pub struct PresenceChannelManager {
    #[allow(dead_code)]
    private_manager: PrivateChannelManager,
    adapter: Arc<dyn Adapter>,
}

impl PresenceChannelManager {
    pub fn new(adapter: Arc<dyn Adapter>) -> Self {
        Self {
            private_manager: PrivateChannelManager::new(adapter.clone()),
            adapter,
        }
    }

    /// Leave a presence channel
    ///
    /// This method removes the socket from the channel and broadcasts a member_removed event
    /// to all remaining subscribers.
    ///
    /// # Arguments
    /// * `app` - The application
    /// * `socket_id` - The socket ID to remove
    /// * `channel` - The channel name to leave
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// Requirements: 7.8
    pub async fn leave(
        &self,
        app: &App,
        socket_id: &str,
        channel: &str,
    ) -> crate::error::Result<()> {
        // Get the namespace to find the user_id for this socket before removing
        let user_id = if let Some(ns) = self.adapter.get_namespace(&app.id) {
            ns.presence_socket_to_user
                .get(channel)
                .and_then(|socket_to_user| {
                    socket_to_user
                        .get(socket_id)
                        .map(|entry| entry.value().clone())
                })
        } else {
            None
        };

        // Remove member from adapter (this also handles the socket_to_user mapping)
        self.adapter
            .remove_member(&app.id, channel, socket_id)
            .await?;

        // Remove socket from channel
        self.adapter
            .remove_from_channel(&app.id, channel, socket_id)
            .await?;

        // Broadcast member_removed event if we found the user_id
        // Only broadcast if the user has no other sockets in this channel
        if let Some(user_id) = user_id {
            // Check if the user still has other sockets in this channel
            let user_still_present = if let Some(ns) = self.adapter.get_namespace(&app.id) {
                ns.presence_socket_to_user
                    .get(channel)
                    .map(|socket_to_user| {
                        socket_to_user.iter().any(|entry| entry.value() == &user_id)
                    })
                    .unwrap_or(false)
            } else {
                false
            };

            // Only broadcast member_removed if the user is completely gone from the channel
            if !user_still_present {
                let member_removed_event = json!({
                    "event": "pusher_internal:member_removed",
                    "channel": channel,
                    "data": json!({
                        "user_id": user_id
                    }).to_string()
                })
                .to_string();

                // Send to all remaining subscribers
                let _ = self
                    .adapter
                    .send(&app.id, channel, &member_removed_event, None)
                    .await;
            }
        }

        Ok(())
    }

    /// Parse and validate channel_data from the subscription message
    ///
    /// # Arguments
    /// * `app` - The application (for size limit validation)
    /// * `channel_data_str` - The channel_data JSON string
    ///
    /// # Returns
    /// Result containing the parsed user_id and user_info, or an error
    ///
    /// Requirements: 7.6
    fn parse_and_validate_channel_data(
        &self,
        app: &App,
        channel_data_str: &str,
    ) -> Result<(String, Value), JoinResponse> {
        // Check member size limit (maxPresenceMemberSizeInKb)
        if let Some(max_size_kb) = app.max_presence_member_size_in_kb {
            let size_bytes = channel_data_str.len();
            let size_kb = size_bytes as f64 / 1024.0;
            if size_kb > max_size_kb {
                return Err(JoinResponse {
                    success: false,
                    error_code: Some(4301),
                    error_message: Some(format!(
                        "Presence member data exceeds maximum size of {} KB",
                        max_size_kb
                    )),
                    auth_error: false,
                    type_: Some("LimitReached".to_string()),
                });
            }
        }

        // Parse channel_data JSON
        let channel_data: Value = match serde_json::from_str(channel_data_str) {
            Ok(data) => data,
            Err(_) => {
                return Err(JoinResponse {
                    success: false,
                    error_code: Some(4009),
                    error_message: Some("Invalid channel_data JSON".to_string()),
                    auth_error: true,
                    type_: Some("AuthError".to_string()),
                });
            }
        };

        // Extract user_id (required)
        let user_id = match channel_data.get("user_id").and_then(|v| v.as_str()) {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => {
                return Err(JoinResponse {
                    success: false,
                    error_code: Some(4009),
                    error_message: Some(
                        "channel_data must contain a non-empty user_id".to_string(),
                    ),
                    auth_error: true,
                    type_: Some("AuthError".to_string()),
                });
            }
        };

        // Extract user_info (optional)
        let user_info = channel_data
            .get("user_info")
            .cloned()
            .unwrap_or(Value::Null);

        Ok((user_id, user_info))
    }

    // Logic duplicate from private manager because overriding methods in Rust struct composition is manual.
    // Ideally extract common logic. For now, reimplement signature check helper or expose it.
    // PrivateChannelManager doesn't expose helpers publicly.
    // I can make them public in PrivateChannelManager or copy. Copying for speed (no refactor of private.rs).
    fn sign(&self, secret: &str, data: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn signature_is_valid(
        &self,
        app: &App,
        socket_id: &str,
        message: &PusherMessage,
        signature_to_check: &str,
    ) -> bool {
        let expected = self.get_expected_signature(app, socket_id, message);
        expected == signature_to_check
    }

    fn get_expected_signature(
        &self,
        app: &App,
        socket_id: &str,
        message: &PusherMessage,
    ) -> String {
        let data_to_sign = self.get_data_to_sign_for_signature(socket_id, message);
        let signature = self.sign(&app.secret, &data_to_sign);
        format!("{}:{}", app.key, signature)
    }

    fn get_data_to_sign_for_signature(&self, socket_id: &str, message: &PusherMessage) -> String {
        let channel = message.channel.as_deref().unwrap_or("");
        let channel_data = message
            .data
            .as_ref()
            .and_then(|d| d.get("channel_data"))
            .and_then(|v| v.as_str())
            .unwrap_or("{}");
        format!("{}:{}:{}", socket_id, channel, channel_data)
    }
}

#[async_trait]
impl ChannelManager for PresenceChannelManager {
    /// Join a presence channel
    ///
    /// This method:
    /// 1. Validates the member count limit
    /// 2. Validates the auth signature (including channel_data)
    /// 3. Parses and validates the channel_data
    /// 4. Adds the socket to the channel
    /// 5. Adds the member to the adapter
    /// 6. Broadcasts a member_added event to all subscribers
    ///
    /// # Arguments
    /// * `app` - The application the socket belongs to
    /// * `socket` - The socket attempting to join
    /// * `channel` - The channel name to join (must start with "presence-")
    /// * `message` - The subscription message containing auth and channel_data
    ///
    /// # Returns
    /// A JoinResponse indicating success or failure
    ///
    /// Requirements: 7.4, 7.6, 7.7
    async fn join(
        &self,
        app: &App,
        socket: &Socket,
        channel: &str,
        message: Option<PusherMessage>,
    ) -> JoinResponse {
        let message = match message {
            Some(m) => m,
            None => {
                return JoinResponse {
                    success: false,
                    error_code: Some(4009),
                    error_message: Some("Message check failed.".to_string()),
                    auth_error: true,
                    type_: Some("AuthError".to_string()),
                };
            }
        };

        // Check members limit
        let count = self
            .adapter
            .get_channel_members_count(&app.id, channel)
            .await
            .unwrap_or(0);
        if let Some(limit) = app.max_presence_members_per_channel
            && count >= limit as usize
        {
            return JoinResponse {
                success: false,
                error_code: Some(4100),
                error_message: Some(
                    "The maximum members per presence channel limit was reached".to_string(),
                ),
                auth_error: false,
                type_: Some("LimitReached".to_string()),
            };
        }

        // Auth check
        let auth = message
            .data
            .as_ref()
            .and_then(|d| d.get("auth"))
            .and_then(|v| v.as_str());

        let passed_signature = match auth {
            Some(s) => s,
            None => {
                return JoinResponse {
                    success: false,
                    error_code: Some(4009),
                    error_message: Some("The connection is unauthorized.".to_string()),
                    auth_error: true,
                    type_: Some("AuthError".to_string()),
                };
            }
        };

        if !self.signature_is_valid(app, &socket.id, &message, passed_signature) {
            return JoinResponse {
                success: false,
                error_code: Some(4009),
                error_message: Some("The connection is unauthorized.".to_string()),
                auth_error: true,
                type_: Some("AuthError".to_string()),
            };
        }

        // Parse and validate channel_data
        let channel_data_str = message
            .data
            .as_ref()
            .and_then(|d| d.get("channel_data"))
            .and_then(|v| v.as_str())
            .unwrap_or("{}");

        let (user_id, user_info) = match self.parse_and_validate_channel_data(app, channel_data_str)
        {
            Ok(data) => data,
            Err(error_response) => return error_response,
        };

        // Add socket to channel via adapter (don't use private_manager.join because it will verify signature differently)
        match self
            .adapter
            .add_to_channel(&app.id, channel, socket.id.clone())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return JoinResponse {
                    success: false,
                    error_code: Some(4302),
                    error_message: Some(format!("Failed to join channel: {}", e)),
                    auth_error: false,
                    type_: Some("ServerError".to_string()),
                };
            }
        }

        // Add member to adapter
        let member = crate::app::PresenceMember {
            user_id: user_id.clone(),
            user_info: user_info.clone(),
        };

        if let Err(e) = self
            .adapter
            .add_member(&app.id, channel, &socket.id, member)
            .await
        {
            // If adding member fails, we should remove the socket from the channel
            let _ = self
                .adapter
                .remove_from_channel(&app.id, channel, &socket.id)
                .await;
            return JoinResponse {
                success: false,
                error_code: Some(4302),
                error_message: Some(format!("Failed to add member: {}", e)),
                auth_error: false,
                type_: Some("ServerError".to_string()),
            };
        }

        // Broadcast member_added event to all subscribers (excluding the new member)
        // Requirements: 7.7
        let member_added_event = json!({
            "event": "pusher_internal:member_added",
            "channel": channel,
            "data": json!({
                "user_id": user_id,
                "user_info": user_info
            }).to_string()
        })
        .to_string();

        // Send to all subscribers except the one who just joined
        let _ = self
            .adapter
            .send(&app.id, channel, &member_added_event, Some(&socket.id))
            .await;

        JoinResponse {
            success: true,
            error_code: None,
            error_message: None,
            auth_error: false,
            type_: None,
        }
    }
}
