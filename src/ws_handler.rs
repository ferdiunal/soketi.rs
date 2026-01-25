use crate::channels::encrypted::EncryptedPrivateChannelManager;
use crate::channels::presence::PresenceChannelManager;
use crate::channels::private::PrivateChannelManager;
use crate::channels::public::PublicChannelManager;
use crate::error::{PusherError, Result};
use crate::namespace::Socket;
use crate::pusher::{ConnectionEstablishedData, PusherMessage};
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

/// WsHandler manages WebSocket connections and Pusher protocol message handling
///
/// This struct is responsible for:
/// - Managing WebSocket connection lifecycle
/// - Routing Pusher protocol messages to appropriate handlers
/// - Managing channel subscriptions through channel managers
/// - Handling authentication and user signin
///
/// Requirements: 2.1
pub struct WsHandler {
    /// Shared application state containing adapters, managers, and configuration
    pub state: Arc<AppState>,

    /// Manager for public channel subscriptions
    pub public_channel_manager: PublicChannelManager,

    /// Manager for private channel subscriptions
    pub private_channel_manager: PrivateChannelManager,

    /// Manager for encrypted private channel subscriptions
    pub encrypted_private_channel_manager: EncryptedPrivateChannelManager,

    /// Manager for presence channel subscriptions
    pub presence_channel_manager: PresenceChannelManager,
}

impl WsHandler {
    /// Create a new WsHandler with the given application state
    ///
    /// This initializes all channel managers with the adapter from the state.
    ///
    /// # Arguments
    /// * `state` - Shared application state
    ///
    /// # Returns
    /// A new WsHandler instance
    ///
    /// Requirements: 2.1
    pub fn new(state: Arc<AppState>) -> Self {
        // Cast LocalAdapter to dyn Adapter for channel managers
        let adapter: Arc<dyn crate::adapters::Adapter> = state.adapter.clone();

        Self {
            state,
            public_channel_manager: PublicChannelManager::new(adapter.clone()),
            private_channel_manager: PrivateChannelManager::new(adapter.clone()),
            encrypted_private_channel_manager: EncryptedPrivateChannelManager::new(adapter.clone()),
            presence_channel_manager: PresenceChannelManager::new(adapter.clone()),
        }
    }

    /// Handle a new WebSocket connection
    ///
    /// This method implements the connection handling logic:
    /// 1. Generate unique Socket_ID
    /// 2. Validate app key and check if app is enabled
    /// 3. Check connection limits
    /// 4. Send pusher:connection_established
    /// 5. Add socket to adapter
    /// 6. Set up user authentication timeout if enabled
    ///
    /// # Arguments
    /// * `socket` - The WebSocket connection
    /// * `app_key` - The application key from the connection URL
    ///
    /// # Requirements
    /// - 2.1: Generate unique Socket_ID and send pusher:connection_established
    /// - 12.1: Validate app key
    /// - 5.9: Check if app is enabled
    /// - 13.3: Check connection limits
    /// - 16.1: Set up user authentication timeout if enabled
    pub async fn handle_connection(&self, socket: WebSocket, app_key: String) {
        info!("New connection attempt for app_key: {}", app_key);

        // Check if server is closing - reject new connections
        if self
            .state
            .closing
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            error!(
                "Server is closing, rejecting new connection for app_key: {}",
                app_key
            );
            self.send_error_and_close(socket, PusherError::ServerClosing)
                .await;
            return;
        }

        // Step 1: Generate unique Socket_ID
        // Format: {random1}.{random2} to match Pusher protocol
        let socket_id = self.generate_socket_id();
        info!("Generated socket_id: {}", socket_id);

        // Step 2: Validate app key and get app configuration
        let app = match self.state.app_manager.find_by_key(&app_key).await {
            Ok(Some(app)) => app,
            Ok(None) => {
                error!("App not found for key: {}", app_key);
                self.send_error_and_close(socket, PusherError::AppNotFound(app_key))
                    .await;
                return;
            }
            Err(e) => {
                error!("Error looking up app: {}", e);
                self.send_error_and_close(socket, PusherError::ServerError(e.to_string()))
                    .await;
                return;
            }
        };

        // Step 3: Check if app is enabled
        if !app.enabled {
            error!("App is disabled: {}", app.id);
            self.send_error_and_close(socket, PusherError::AppDisabled(app.id.clone()))
                .await;
            return;
        }

        // Step 4: Check connection limits
        if let Some(max_connections) = app.max_connections {
            match self.state.adapter.get_sockets_count(&app.id).await {
                Ok(current_count) => {
                    if current_count >= max_connections as usize {
                        error!(
                            "Connection limit reached for app: {} ({}/{})",
                            app.id, current_count, max_connections
                        );
                        self.send_error_and_close(socket, PusherError::ConnectionLimitReached)
                            .await;
                        return;
                    }
                }
                Err(e) => {
                    error!("Error checking connection count: {}", e);
                    self.send_error_and_close(socket, PusherError::ServerError(e.to_string()))
                        .await;
                    return;
                }
            }
        }

        // Split the WebSocket into sender and receiver
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Step 5: Send pusher:connection_established
        let conn_data = ConnectionEstablishedData {
            socket_id: socket_id.clone(),
            activity_timeout: 120, // 120 seconds as per Pusher protocol
        };

        let conn_msg = PusherMessage::new("pusher:connection_established".to_string())
            .with_data(serde_json::to_value(&conn_data).unwrap());

        if let Err(e) = self.send_message(&mut ws_sender, &conn_msg).await {
            error!("Failed to send connection_established: {}", e);
            return;
        }

        info!("Connection established for socket: {}", socket_id);

        // Create a channel for sending messages to the client
        let (tx, mut rx) = mpsc::channel::<Message>(100);

        // Step 6: Add socket to adapter
        let socket_info = Socket {
            id: socket_id.clone(),
            sender: tx.clone(),
        };

        if let Err(e) = self.state.adapter.add_socket(&app.id, socket_info).await {
            error!("Failed to add socket to adapter: {}", e);
            self.send_error_and_close_split(ws_sender, PusherError::ServerError(e.to_string()))
                .await;
            return;
        }

        // Step 7: Set up user authentication timeout if enabled
        let auth_timeout_handle = if app.enable_user_authentication {
            let socket_id_clone = socket_id.clone();
            let app_id_clone = app.id.clone();
            let adapter_clone = self.state.adapter.clone();
            let tx_clone = tx.clone();

            Some(tokio::spawn(async move {
                // Default timeout is 30 seconds (can be made configurable)
                tokio::time::sleep(Duration::from_secs(30)).await;

                // Check if user has authenticated
                // For now, we'll send an error and close the connection
                // In a full implementation, we'd check if the socket has an associated user
                warn!(
                    "User authentication timeout for socket: {}",
                    socket_id_clone
                );

                let error_msg = PusherError::UserAuthenticationTimeout.to_error_message(None);
                let json = serde_json::to_string(&error_msg).unwrap();
                let _ = tx_clone.send(Message::Text(json.into())).await;

                // Remove socket from adapter
                let _ = adapter_clone
                    .remove_socket(&app_id_clone, &socket_id_clone)
                    .await;
            }))
        } else {
            None
        };

        // Spawn task to forward messages from channel to WebSocket
        let mut send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if ws_sender.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Main message receiving loop
        let socket_id_clone = socket_id.clone();
        let app_id_clone = app.id.clone();
        let app_clone = app.clone();
        let adapter_clone = self.state.adapter.clone();
        let tx_clone = tx.clone();
        let state_clone = self.state.clone();

        let mut recv_task = tokio::spawn(async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        info!("Received message from {}: {}", socket_id_clone, text);

                        // Parse the Pusher protocol message
                        match serde_json::from_str::<PusherMessage>(&text) {
                            Ok(pusher_msg) => {
                                // Route message to appropriate handler based on event type
                                let result = Self::route_message(
                                    &state_clone,
                                    &socket_id_clone,
                                    &app_clone,
                                    &tx_clone,
                                    pusher_msg,
                                )
                                .await;

                                if let Err(e) = result {
                                    error!("Error handling message: {}", e);
                                    // Send error to client
                                    let error_msg = e.to_error_message(None);
                                    let json =
                                        serde_json::to_string(&error_msg).unwrap_or_default();
                                    let _ = tx_clone.send(Message::Text(json.into())).await;
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse message from {}: {}", socket_id_clone, e);
                                let error =
                                    PusherError::InvalidMessage(format!("Invalid JSON: {}", e));
                                let error_msg = error.to_error_message(None);
                                let json = serde_json::to_string(&error_msg).unwrap_or_default();
                                let _ = tx_clone.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Client closed connection: {}", socket_id_clone);
                        break;
                    }
                    Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                        // WebSocket-level ping/pong, handled automatically
                    }
                    Ok(_) => {
                        // Ignore other message types
                    }
                    Err(e) => {
                        error!("WebSocket error for {}: {}", socket_id_clone, e);
                        break;
                    }
                }
            }

            // Connection close handling - Requirements: 2.7
            // This cleanup is performed when the connection is closed for any reason:
            // - Client initiated close
            // - Server error
            // - Network error
            info!(
                "Connection closed, performing cleanup for socket: {}",
                socket_id_clone
            );

            // Step 1: Get all channels the socket is subscribed to
            // We need to unsubscribe from all channels and handle presence channel cleanup
            let channels_to_cleanup = if let Some(ns) = adapter_clone.get_namespace(&app_id_clone) {
                // Find all channels this socket is subscribed to
                ns.channels
                    .iter()
                    .filter(|entry| entry.value().contains(&socket_id_clone))
                    .map(|entry| entry.key().clone())
                    .collect::<Vec<String>>()
            } else {
                Vec::new()
            };

            // Step 2: Unsubscribe from all channels (especially presence channels need special handling)
            for channel in &channels_to_cleanup {
                if channel.starts_with("presence-") {
                    // For presence channels, we need to:
                    // 1. Remove the member from the presence list
                    // 2. Broadcast member_removed event to other subscribers
                    info!(
                        "Cleaning up presence channel: {} for socket: {}",
                        channel, socket_id_clone
                    );

                    // Get the user_id for this socket in this presence channel before removing
                    let user_id = if let Some(ns) = adapter_clone.get_namespace(&app_id_clone) {
                        ns.presence_socket_to_user
                            .get(channel)
                            .and_then(|socket_to_user| {
                                socket_to_user
                                    .get(&socket_id_clone)
                                    .map(|entry| entry.value().clone())
                            })
                    } else {
                        None
                    };

                    // Remove member from presence channel
                    if let Err(e) = adapter_clone
                        .remove_member(&app_id_clone, channel, &socket_id_clone)
                        .await
                    {
                        error!(
                            "Error removing member from presence channel {}: {}",
                            channel, e
                        );
                    }

                    // Broadcast member_removed event if we have a user_id
                    if let Some(uid) = user_id {
                        let member_removed_msg = crate::pusher::PusherMessage::new(
                            "pusher_internal:member_removed".to_string(),
                        )
                        .with_channel(channel.clone())
                        .with_data(serde_json::json!({
                            "user_id": uid
                        }));

                        if let Ok(json) = serde_json::to_string(&member_removed_msg) {
                            // Send to all subscribers except the leaving socket
                            let _ = adapter_clone
                                .send(&app_id_clone, channel, &json, Some(&socket_id_clone))
                                .await;
                        }
                    }
                }
            }

            // Step 3: Check if socket has an authenticated user and remove user association
            // Get the user_id associated with this socket (if any)
            let user_id = if let Some(ns) = adapter_clone.get_namespace(&app_id_clone) {
                // Find user_id by checking which user has this socket
                ns.users
                    .iter()
                    .find(|entry| entry.value().contains(&socket_id_clone))
                    .map(|entry| entry.key().clone())
            } else {
                None
            };

            // Remove user association if authenticated
            if let Some(uid) = user_id {
                info!(
                    "Removing user association for user: {} socket: {}",
                    uid, socket_id_clone
                );
                if let Err(e) = adapter_clone
                    .remove_user(&app_id_clone, &uid, &socket_id_clone)
                    .await
                {
                    error!("Error removing user from adapter: {}", e);
                }
            }

            // Step 4: Remove socket from adapter
            // This will also clean up any remaining channel subscriptions
            info!("Removing socket from adapter: {}", socket_id_clone);
            if let Err(e) = adapter_clone
                .remove_socket(&app_id_clone, &socket_id_clone)
                .await
            {
                error!("Error removing socket from adapter: {}", e);
            }

            info!("Cleanup completed for socket: {}", socket_id_clone);
        });

        // Wait for either task to complete
        tokio::select! {
            _ = &mut recv_task => {
                send_task.abort();
            }
            _ = &mut send_task => {
                recv_task.abort();
            }
        }

        // Cancel auth timeout if it exists
        if let Some(handle) = auth_timeout_handle {
            handle.abort();
        }

        info!("Connection closed for socket: {}", socket_id);
    }

    /// Generate a unique Socket_ID in Pusher format
    ///
    /// Format: {random1}.{random2}
    /// Each part is a random number to ensure uniqueness
    ///
    /// Requirements: 2.1
    pub fn generate_socket_id(&self) -> String {
        // Generate two random numbers and format them as Pusher socket ID
        // Using UUID for randomness, taking modulo to keep numbers reasonable
        let part1 = Uuid::new_v4().as_u128() % 1_000_000_000;
        let part2 = Uuid::new_v4().as_u128() % 1_000_000_000;
        format!("{}.{}", part1, part2)
    }

    /// Route a Pusher protocol message to the appropriate handler
    ///
    /// This method implements message routing based on the event type:
    /// - pusher:ping -> handle_ping
    /// - pusher:subscribe -> handle_subscribe
    /// - pusher:unsubscribe -> handle_unsubscribe
    /// - pusher:signin -> handle_signin
    /// - client-* events -> handle_client_event
    ///
    /// # Arguments
    /// * `state` - Shared application state
    /// * `socket_id` - The socket ID
    /// * `app` - The application configuration
    /// * `sender` - Channel sender for sending messages to the client
    /// * `message` - The Pusher protocol message to route
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// Requirements: 2.2, 2.3, 2.4, 2.5, 2.6
    #[cfg_attr(test, allow(dead_code))]
    pub async fn route_message(
        state: &Arc<AppState>,
        socket_id: &str,
        app: &crate::app::App,
        sender: &mpsc::Sender<Message>,
        message: PusherMessage,
    ) -> Result<()> {
        info!(
            "Routing message: event={}, channel={:?}",
            message.event, message.channel
        );

        match message.event.as_str() {
            "pusher:ping" => Self::handle_ping(state, socket_id, sender).await,
            "pusher:subscribe" => {
                Self::handle_subscribe(state, socket_id, app, sender, message).await
            }
            "pusher:unsubscribe" => {
                Self::handle_unsubscribe(state, socket_id, app, sender, message).await
            }
            "pusher:signin" => Self::handle_signin(state, socket_id, app, sender, message).await,
            event if event.starts_with("client-") => {
                Self::handle_client_event(state, socket_id, app, sender, message).await
            }
            _ => {
                warn!("Unknown event type: {}", message.event);
                Err(PusherError::InvalidMessage(format!(
                    "Unknown event: {}",
                    message.event
                )))
            }
        }
    }

    /// Handle pusher:ping message
    ///
    /// Responds with pusher:pong and checks if server is closing.
    /// If the server is closing, returns an error to trigger disconnection.
    ///
    /// # Arguments
    /// * `state` - Shared application state
    /// * `socket_id` - The socket ID
    /// * `sender` - Channel sender for sending messages to the client
    ///
    /// # Returns
    /// Result indicating success or error (ServerClosing if server is shutting down)
    ///
    /// Requirements: 2.2
    async fn handle_ping(
        state: &Arc<AppState>,
        socket_id: &str,
        sender: &mpsc::Sender<Message>,
    ) -> Result<()> {
        info!("Handling ping from socket: {}", socket_id);

        // Check if server is closing
        if state.closing.load(std::sync::atomic::Ordering::Relaxed) {
            warn!(
                "Server is closing, rejecting ping from socket: {}",
                socket_id
            );
            return Err(PusherError::ServerClosing);
        }

        // Send pusher:pong response
        let pong_msg = PusherMessage::new("pusher:pong".to_string());
        let json = serde_json::to_string(&pong_msg)?;

        sender
            .send(Message::Text(json.into()))
            .await
            .map_err(|e| PusherError::IoError(format!("Failed to send pong: {}", e)))?;

        Ok(())
    }

    /// Handle pusher:subscribe message
    ///
    /// Validates the subscription and routes to the appropriate channel manager.
    /// This method:
    /// 1. Extracts and validates the channel name
    /// 2. Validates channel name length against configured limits
    /// 3. Determines the channel type (public, private, encrypted, presence)
    /// 4. Routes to the appropriate channel manager
    /// 5. Sends subscription_succeeded or subscription_error response
    /// 6. For presence channels, includes presence data in the response
    /// 7. For cache-enabled channels, sends cache miss or cached event
    ///
    /// # Arguments
    /// * `state` - Shared application state
    /// * `socket_id` - The socket ID
    /// * `app` - The application configuration
    /// * `sender` - Channel sender for sending messages to the client
    /// * `message` - The subscribe message
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// Requirements: 2.3, 7.1, 7.2, 7.3, 7.4, 7.9, 7.10
    async fn handle_subscribe(
        state: &Arc<AppState>,
        socket_id: &str,
        app: &crate::app::App,
        sender: &mpsc::Sender<Message>,
        message: PusherMessage,
    ) -> Result<()> {
        // Step 1: Extract channel name from message
        let channel = match &message.channel {
            Some(ch) => ch,
            None => {
                error!("Subscribe message missing channel field");
                return Err(PusherError::InvalidMessage(
                    "Channel name is required".to_string(),
                ));
            }
        };

        info!(
            "Handling subscribe from socket: {} for channel: {}",
            socket_id, channel
        );

        // Step 2: Validate channel name length
        // Requirements: 7.9
        let max_channel_length = app.max_channel_name_length.unwrap_or(200);
        if channel.len() as u64 > max_channel_length {
            error!(
                "Channel name exceeds maximum length: {} > {}",
                channel.len(),
                max_channel_length
            );
            let error_msg = PusherError::ValidationError(format!(
                "Channel name exceeds maximum length of {} characters",
                max_channel_length
            ))
            .to_error_message(Some(channel));
            let json = serde_json::to_string(&error_msg)?;
            sender
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| PusherError::IoError(format!("Failed to send error: {}", e)))?;
            return Ok(());
        }

        // Step 3: Create Socket info for channel manager
        let socket_info = crate::namespace::Socket {
            id: socket_id.to_string(),
            sender: sender.clone(),
        };

        // Convert pusher::PusherMessage to channels::PusherMessage
        let channel_message = crate::channels::PusherMessage {
            event: message.event.clone(),
            data: message.data.clone(),
            channel: message.channel.clone(),
            socket_id: Some(socket_id.to_string()),
        };

        // Step 4: Determine channel type and route to appropriate manager
        // Requirements: 7.1, 7.2, 7.3, 7.4
        let join_response = if channel.starts_with("presence-") {
            // Presence channel
            use crate::channels::ChannelManager;
            let presence_manager =
                crate::channels::presence::PresenceChannelManager::new(state.adapter.clone());
            presence_manager
                .join(app, &socket_info, channel, Some(channel_message.clone()))
                .await
        } else if channel.starts_with("private-encrypted-") {
            // Encrypted private channel
            use crate::channels::ChannelManager;
            let encrypted_manager = crate::channels::encrypted::EncryptedPrivateChannelManager::new(
                state.adapter.clone(),
            );
            encrypted_manager
                .join(app, &socket_info, channel, Some(channel_message.clone()))
                .await
        } else if channel.starts_with("private-") {
            // Private channel
            use crate::channels::ChannelManager;
            let private_manager =
                crate::channels::private::PrivateChannelManager::new(state.adapter.clone());
            private_manager
                .join(app, &socket_info, channel, Some(channel_message.clone()))
                .await
        } else {
            // Public channel
            let public_manager =
                crate::channels::public::PublicChannelManager::new(state.adapter.clone());
            public_manager
                .join(app, &socket_info, channel, Some(channel_message.clone()))
                .await
        };

        // Step 5: Send response based on join result
        if join_response.success {
            // Subscription succeeded
            info!(
                "Subscription succeeded for socket: {} on channel: {}",
                socket_id, channel
            );

            // For presence channels, include presence data
            // Requirements: 7.4
            let subscription_data = if channel.starts_with("presence-") {
                // Get current members from adapter
                match state.adapter.get_channel_members(&app.id, channel).await {
                    Ok(members) => {
                        // Build presence data
                        let ids: Vec<String> = members.keys().cloned().collect();
                        let count = ids.len();

                        // Build hash of user_id -> user_info
                        let mut hash = serde_json::Map::new();
                        for (user_id, member) in members {
                            hash.insert(user_id, member.user_info);
                        }

                        let presence_data = crate::pusher::PresenceData {
                            ids,
                            hash: serde_json::Value::Object(hash),
                            count,
                        };

                        Some(crate::pusher::SubscriptionSucceededData {
                            presence: Some(presence_data),
                        })
                    }
                    Err(e) => {
                        warn!("Failed to get presence members: {}", e);
                        Some(crate::pusher::SubscriptionSucceededData { presence: None })
                    }
                }
            } else {
                None
            };

            // Build subscription_succeeded message
            let success_msg = if let Some(data) = subscription_data {
                crate::pusher::PusherMessage::new(
                    "pusher_internal:subscription_succeeded".to_string(),
                )
                .with_channel(channel.clone())
                .with_data(serde_json::to_value(&data)?)
            } else {
                crate::pusher::PusherMessage::new(
                    "pusher_internal:subscription_succeeded".to_string(),
                )
                .with_channel(channel.clone())
            };

            let json = serde_json::to_string(&success_msg)?;
            sender
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| PusherError::IoError(format!("Failed to send success: {}", e)))?;

            // TODO: Step 6: For cache-enabled channels, send cache miss or cached event
            // Requirements: 7.10
            // This will be implemented when cache functionality is added
        } else {
            // Subscription failed
            error!(
                "Subscription failed for socket: {} on channel: {}: {:?}",
                socket_id, channel, join_response.error_message
            );

            // Send subscription_error
            let error_data = serde_json::json!({
                "type": join_response.type_.unwrap_or_else(|| "Error".to_string()),
                "error": join_response.error_message.unwrap_or_else(|| "Subscription failed".to_string()),
                "status": join_response.error_code.unwrap_or(4302),
            });

            let error_msg =
                crate::pusher::PusherMessage::new("pusher:subscription_error".to_string())
                    .with_channel(channel.clone())
                    .with_data(error_data);

            let json = serde_json::to_string(&error_msg)?;
            sender
                .send(Message::Text(json.into()))
                .await
                .map_err(|e| PusherError::IoError(format!("Failed to send error: {}", e)))?;
        }

        Ok(())
    }

    /// Handle pusher:unsubscribe message
    ///
    /// Removes the connection from the channel.
    /// This method:
    /// 1. Extracts and validates the channel name
    /// 2. Determines the channel type (public, private, encrypted, presence)
    /// 3. Calls the appropriate channel manager's leave method
    /// 4. Updates socket state by removing from adapter
    ///
    /// # Arguments
    /// * `state` - Shared application state
    /// * `socket_id` - The socket ID
    /// * `app` - The application configuration
    /// * `sender` - Channel sender for sending messages to the client
    /// * `message` - The unsubscribe message
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// Requirements: 2.4
    async fn handle_unsubscribe(
        state: &Arc<AppState>,
        socket_id: &str,
        app: &crate::app::App,
        _sender: &mpsc::Sender<Message>,
        message: PusherMessage,
    ) -> Result<()> {
        // Step 1: Extract channel name from message
        let channel = match &message.channel {
            Some(ch) => ch,
            None => {
                error!("Unsubscribe message missing channel field");
                return Err(PusherError::InvalidMessage(
                    "Channel name is required".to_string(),
                ));
            }
        };

        info!(
            "Handling unsubscribe from socket: {} for channel: {}",
            socket_id, channel
        );

        // Step 2: Determine channel type and call appropriate channel manager's leave method
        // Requirements: 2.4
        if channel.starts_with("presence-") {
            // Presence channel - use PresenceChannelManager
            let presence_manager =
                crate::channels::presence::PresenceChannelManager::new(state.adapter.clone());
            presence_manager.leave(app, socket_id, channel).await?;
        } else if channel.starts_with("private-encrypted-") || channel.starts_with("private-") {
            // Private or encrypted private channel - use PublicChannelManager's leave
            // (private channels don't need special leave logic, just remove from adapter)
            let public_manager =
                crate::channels::public::PublicChannelManager::new(state.adapter.clone());
            public_manager.leave(&app.id, socket_id, channel).await?;
        } else {
            // Public channel - use PublicChannelManager
            let public_manager =
                crate::channels::public::PublicChannelManager::new(state.adapter.clone());
            public_manager.leave(&app.id, socket_id, channel).await?;
        }

        info!(
            "Successfully unsubscribed socket: {} from channel: {}",
            socket_id, channel
        );

        Ok(())
    }

    /// Handle client event message
    ///
    /// Validates and broadcasts client events if enabled.
    /// This method:
    /// 1. Validates client messages are enabled for the app
    /// 2. Validates event name length against configured limits
    /// 3. Validates payload size against configured limits
    /// 4. Validates socket is subscribed to the channel
    /// 5. Checks rate limits for client events
    /// 6. Broadcasts event to channel (excluding sender)
    /// 7. Includes user_id for presence channels
    /// 8. Sends webhook if configured
    ///
    /// # Arguments
    /// * `state` - Shared application state
    /// * `socket_id` - The socket ID
    /// * `app` - The application configuration
    /// * `sender` - Channel sender for sending messages to the client
    /// * `message` - The client event message
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// Requirements: 2.5, 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7
    async fn handle_client_event(
        state: &Arc<AppState>,
        socket_id: &str,
        app: &crate::app::App,
        _sender: &mpsc::Sender<Message>,
        message: PusherMessage,
    ) -> Result<()> {
        info!(
            "Handling client event from socket: {} event: {} channel: {:?}",
            socket_id, message.event, message.channel
        );

        // Step 1: Validate client messages are enabled
        // Requirements: 15.2
        if !app.enable_client_messages {
            error!("Client messages are disabled for app: {}", app.id);
            return Err(PusherError::ClientMessagesDisabled);
        }

        // Step 2: Extract channel name from message
        let channel = match &message.channel {
            Some(ch) => ch,
            None => {
                error!("Client event missing channel field");
                return Err(PusherError::InvalidMessage(
                    "Channel name is required for client events".to_string(),
                ));
            }
        };

        // Step 3: Validate event name length
        // Requirements: 15.4
        let max_event_name_length = app.max_event_name_length.unwrap_or(200);
        if message.event.len() as u64 > max_event_name_length {
            error!(
                "Event name exceeds maximum length: {} > {}",
                message.event.len(),
                max_event_name_length
            );
            return Err(PusherError::ValidationError(format!(
                "Event name exceeds maximum length of {} characters",
                max_event_name_length
            )));
        }

        // Step 4: Validate payload size
        // Requirements: 15.5
        if let Some(data) = &message.data {
            let payload_str = serde_json::to_string(data)?;
            let max_payload_kb = app.max_event_payload_in_kb.unwrap_or(100.0);
            let payload_size_kb = payload_str.len() as f64 / 1024.0;

            if payload_size_kb > max_payload_kb {
                error!(
                    "Payload size exceeds maximum: {:.2} KB > {:.2} KB",
                    payload_size_kb, max_payload_kb
                );
                return Err(PusherError::ValidationError(format!(
                    "Payload size {:.2} KB exceeds maximum of {:.2} KB",
                    payload_size_kb, max_payload_kb
                )));
            }
        }

        // Step 5: Validate socket is subscribed to channel
        // Requirements: 15.3
        let is_subscribed = state
            .adapter
            .is_in_channel(&app.id, channel, socket_id)
            .await?;
        if !is_subscribed {
            error!(
                "Socket {} is not subscribed to channel {}",
                socket_id, channel
            );
            return Err(PusherError::ValidationError(format!(
                "Socket is not subscribed to channel {}",
                channel
            )));
        }

        // Step 6: Check rate limits
        // Requirements: 2.5
        let rate_limit_response = state
            .rate_limiter
            .consume_frontend_event_points(1, app, socket_id)
            .await?;

        if !rate_limit_response.can_continue {
            error!("Rate limit exceeded for socket: {}", socket_id);
            return Err(PusherError::RateLimitExceeded);
        }

        // Step 7: Get user_id for presence channels
        // Requirements: 15.6
        let user_id = if channel.starts_with("presence-") {
            // Get the namespace to find the user_id for this socket
            if let Some(ns) = state.adapter.get_namespace(&app.id) {
                ns.presence_socket_to_user
                    .get(channel)
                    .and_then(|socket_to_user| {
                        socket_to_user
                            .get(socket_id)
                            .map(|entry| entry.value().clone())
                    })
            } else {
                None
            }
        } else {
            None
        };

        // Step 8: Build the broadcast message
        // Include user_id for presence channels (Requirements: 15.6)
        let broadcast_data = if let Some(uid) = &user_id {
            // For presence channels, wrap the data with user_id
            let mut data_obj = serde_json::Map::new();
            if let Some(data) = &message.data {
                data_obj.insert("data".to_string(), data.clone());
            }
            data_obj.insert(
                "user_id".to_string(),
                serde_json::Value::String(uid.clone()),
            );
            Some(serde_json::Value::Object(data_obj))
        } else {
            message.data.clone()
        };

        let broadcast_msg = crate::pusher::PusherMessage {
            event: message.event.clone(),
            data: broadcast_data.clone(),
            channel: Some(channel.clone()),
        };

        let json = serde_json::to_string(&broadcast_msg)?;

        // Step 9: Broadcast event to channel (excluding sender)
        // Requirements: 15.7
        state
            .adapter
            .send(&app.id, channel, &json, Some(socket_id))
            .await?;

        info!(
            "Client event broadcast to channel: {} from socket: {}",
            channel, socket_id
        );

        // Step 10: Send webhook if configured
        // Requirements: 2.5
        if app.has_client_event_webhooks() {
            let data_value = message.data.clone().unwrap_or(serde_json::Value::Null);
            state
                .webhook_sender
                .send_client_event(
                    app,
                    channel,
                    &message.event,
                    data_value,
                    Some(socket_id),
                    user_id.as_deref(),
                )
                .await;
        }

        Ok(())
    }

    /// Handle pusher:signin message
    ///
    /// Validates user authentication and associates the user with the connection.
    /// This method:
    /// 1. Extracts user_data and auth from the message
    /// 2. Validates the auth signature
    /// 3. Parses user_data to extract user_id
    /// 4. Associates the user with the socket in the adapter
    /// 5. Sends pusher:signin_success or returns authentication error
    ///
    /// # Arguments
    /// * `state` - Shared application state
    /// * `socket_id` - The socket ID
    /// * `app` - The application configuration
    /// * `sender` - Channel sender for sending messages to the client
    /// * `message` - The signin message
    ///
    /// # Returns
    /// Result indicating success or error
    ///
    /// Requirements: 2.6, 16.2, 16.3, 16.4, 16.5
    async fn handle_signin(
        state: &Arc<AppState>,
        socket_id: &str,
        app: &crate::app::App,
        sender: &mpsc::Sender<Message>,
        message: PusherMessage,
    ) -> Result<()> {
        info!(
            "Handling signin from socket: {} data: {:?}",
            socket_id, message.data
        );

        // Step 1: Extract user_data and auth from message data
        // Requirements: 16.2
        let data = match &message.data {
            Some(d) => d,
            None => {
                error!("Signin message missing data field");
                return Err(PusherError::InvalidMessage(
                    "Signin message requires data field".to_string(),
                ));
            }
        };

        let user_data_str = match data.get("user_data").and_then(|v| v.as_str()) {
            Some(ud) => ud,
            None => {
                error!("Signin message missing user_data field");
                return Err(PusherError::InvalidMessage(
                    "Signin message requires user_data field".to_string(),
                ));
            }
        };

        let auth = match data.get("auth").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => {
                error!("Signin message missing auth field");
                return Err(PusherError::InvalidMessage(
                    "Signin message requires auth field".to_string(),
                ));
            }
        };

        // Step 2: Validate the auth signature
        // Requirements: 16.2, 16.4
        if !crate::auth::verify_user_auth(auth, &app.secret, socket_id, user_data_str) {
            error!(
                "Invalid user authentication signature for socket: {}",
                socket_id
            );
            return Err(PusherError::AuthenticationFailed(
                "Invalid user authentication signature".to_string(),
            ));
        }

        info!(
            "User authentication signature verified for socket: {}",
            socket_id
        );

        // Step 3: Parse user_data to extract user_id
        // Requirements: 16.3
        let user_data_json: serde_json::Value = match serde_json::from_str(user_data_str) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to parse user_data JSON: {}", e);
                return Err(PusherError::InvalidMessage(format!(
                    "Invalid user_data JSON: {}",
                    e
                )));
            }
        };

        let user_id = match user_data_json.get("id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                error!("user_data missing required 'id' field");
                return Err(PusherError::InvalidMessage(
                    "user_data must contain 'id' field".to_string(),
                ));
            }
        };

        info!("Extracted user_id: {} for socket: {}", user_id, socket_id);

        // Step 4: Associate user with socket in adapter
        // Requirements: 16.3, 16.5
        state.adapter.add_user(&app.id, user_id, socket_id).await?;

        info!("User {} associated with socket: {}", user_id, socket_id);

        // Step 5: Send pusher:signin_success
        // Requirements: 16.3
        let signin_success_msg =
            crate::pusher::PusherMessage::new("pusher:signin_success".to_string())
                .with_data(user_data_json);

        let json = serde_json::to_string(&signin_success_msg)?;
        sender
            .send(Message::Text(json.into()))
            .await
            .map_err(|e| PusherError::IoError(format!("Failed to send signin_success: {}", e)))?;

        info!("Sent pusher:signin_success to socket: {}", socket_id);

        Ok(())
    }

    /// Send a Pusher message over the WebSocket
    async fn send_message<S>(&self, sender: &mut S, message: &PusherMessage) -> Result<()>
    where
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        let json = serde_json::to_string(message)?;
        sender
            .send(Message::Text(json.into()))
            .await
            .map_err(|e| PusherError::IoError(e.to_string()))
    }

    /// Send an error message and close the connection
    async fn send_error_and_close(&self, socket: WebSocket, error: PusherError) {
        let (sender, _) = socket.split();
        self.send_error_and_close_split(sender, error).await;
    }

    /// Send an error message and close the connection (with split sender)
    async fn send_error_and_close_split<S>(&self, mut sender: S, error: PusherError)
    where
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Display,
    {
        let error_msg = error.to_error_message(None);
        let json = serde_json::to_string(&error_msg).unwrap_or_default();
        let _ = sender.send(Message::Text(json.into())).await;
        let _ = sender.send(Message::Close(None)).await;
    }

    /// Close all local sockets with server closing error
    ///
    /// This method is called during graceful shutdown to:
    /// 1. Send pusher:error with code 4200 to all connected sockets
    /// 2. Close all connections
    /// 3. Clean up all resources
    ///
    /// The method iterates through all apps and their sockets, sending the
    /// shutdown signal to each one. This allows clients to reconnect gracefully.
    ///
    /// # Requirements
    /// - 2.10: Send pusher:error with code 4200 to all sockets and close connections
    /// - 1.4: Signal existing connections to reconnect during shutdown
    pub async fn close_all_local_sockets(&self) {
        info!("Closing all local sockets");

        // Create the error message that will be sent to all sockets
        let error = PusherError::ServerClosing;
        let error_msg = error.to_error_message(None);
        let json = match serde_json::to_string(&error_msg) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize error message: {}", e);
                return;
            }
        };

        // We need to get all sockets across all apps
        // The adapter doesn't have a method to get all apps, so we'll need to
        // iterate through the namespaces if the adapter supports it

        // For LocalAdapter and ClusterAdapter, we can access the namespaces directly
        // For other adapters, this would need to be implemented differently

        use crate::adapters::cluster::ClusterAdapter;
        use crate::adapters::local::LocalAdapter;

        // Try to get the local adapter (either directly or from ClusterAdapter)
        let local_adapter_opt = if let Some(local_adapter) =
            self.state.adapter.as_any().downcast_ref::<LocalAdapter>()
        {
            Some(local_adapter)
        } else {
            self.state
                .adapter
                .as_any()
                .downcast_ref::<ClusterAdapter>()
                .map(|cluster_adapter| cluster_adapter.get_local_adapter())
        };

        if let Some(local_adapter) = local_adapter_opt {
            // Get all namespaces
            let namespaces = local_adapter.get_all_namespaces();

            // Iterate through each namespace (app)
            for namespace_entry in namespaces.iter() {
                let app_id = namespace_entry.key();
                let namespace = namespace_entry.value();

                info!("Closing sockets for app: {}", app_id);

                // Get all sockets in this namespace
                let sockets: Vec<Socket> = namespace
                    .sockets
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect();

                info!(
                    "Found {} sockets to close for app: {}",
                    sockets.len(),
                    app_id
                );

                // Send error message to each socket
                for socket in sockets {
                    info!("Sending server closing error to socket: {}", socket.id);

                    // Send the error message
                    if let Err(e) = socket.sender.send(Message::Text(json.clone().into())).await {
                        warn!(
                            "Failed to send closing error to socket {}: {}",
                            socket.id, e
                        );
                    }

                    // Send close frame
                    if let Err(e) = socket.sender.send(Message::Close(None)).await {
                        warn!("Failed to send close frame to socket {}: {}", socket.id, e);
                    }
                }
            }

            info!("All local sockets have been signaled to close");
        } else {
            // For other adapter types, we would need a different approach
            // This could be implemented by adding a method to the Adapter trait
            warn!(
                "close_all_local_sockets is only implemented for LocalAdapter and ClusterAdapter"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Comprehensive tests for WsHandler are in the tests/ directory
    // including ws_handler_connection_test.rs, ws_handler_subscribe_test.rs, etc.
}
