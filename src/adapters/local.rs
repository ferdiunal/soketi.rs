use crate::adapters::Adapter;
use crate::app::PresenceMember;
use crate::error::Result;
use crate::namespace::{Namespace, Socket};
use async_trait::async_trait;
use axum::extract::ws::Message;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct LocalAdapter {
    pub namespaces: Arc<DashMap<String, Namespace>>,
}

impl Default for LocalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalAdapter {
    pub fn new() -> Self {
        Self {
            namespaces: Arc::new(DashMap::new()),
        }
    }

    /// Get all namespaces
    /// This is used for operations that need to iterate over all apps
    pub fn get_all_namespaces(&self) -> &DashMap<String, Namespace> {
        &self.namespaces
    }
}

#[async_trait]
impl Adapter for LocalAdapter {
    async fn init(&self) -> Result<()> {
        // Initialization logic if any
        Ok(())
    }

    fn get_namespace(&self, app_id: &str) -> Option<Namespace> {
        self.namespaces.get(app_id).map(|ns| ns.clone())
    }

    // ===== Socket Management Methods =====

    async fn add_socket(&self, app_id: &str, socket: Socket) -> Result<()> {
        let ns = self
            .namespaces
            .entry(app_id.to_string())
            .or_insert_with(|| Namespace::new(app_id.to_string()));
        ns.add_socket(socket).await;
        Ok(())
    }

    async fn remove_socket(&self, app_id: &str, socket_id: &str) -> Result<()> {
        if let Some(ns) = self.namespaces.get(app_id) {
            ns.remove_socket(socket_id).await;
        }
        Ok(())
    }

    async fn get_sockets_count(&self, app_id: &str) -> Result<usize> {
        if let Some(ns) = self.namespaces.get(app_id) {
            Ok(ns.sockets.len())
        } else {
            Ok(0)
        }
    }

    // ===== Channel Methods =====

    async fn get_channel_sockets_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        if let Some(ns) = self.namespaces.get(app_id) {
            if let Some(c) = ns.channels.get(channel) {
                Ok(c.len())
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    async fn get_channels_with_sockets_count(
        &self,
        app_id: &str,
    ) -> Result<HashMap<String, usize>> {
        if let Some(ns) = self.namespaces.get(app_id) {
            Ok(ns
                .channels
                .iter()
                .map(|k| (k.key().clone(), k.value().len()))
                .collect())
        } else {
            Ok(HashMap::new())
        }
    }

    async fn is_in_channel(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<bool> {
        if let Some(ns) = self.namespaces.get(app_id) {
            if let Some(c) = ns.channels.get(channel) {
                Ok(c.contains(socket_id))
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    async fn send(
        &self,
        app_id: &str,
        channel: &str,
        message: &str,
        except_socket_id: Option<&str>,
    ) -> Result<()> {
        tracing::debug!("LocalAdapter::send - app_id: {}, channel: {}", app_id, channel);
        
        if let Some(ns) = self.namespaces.get(app_id) {
            tracing::debug!("Found namespace for app: {}", app_id);
            
            if let Some(channel_sockets) = ns.channels.get(channel) {
                tracing::debug!("Found {} sockets in channel: {}", channel_sockets.len(), channel);
                
                // TODO: Optimize parallel sending
                for socket_id in channel_sockets.iter() {
                    let socket_id = socket_id.key();
                    if let Some(except) = except_socket_id
                        && socket_id == except
                    {
                        tracing::debug!("Skipping socket (except): {}", socket_id);
                        continue;
                    }

                    if let Some(socket) = ns.sockets.get(socket_id) {
                        tracing::debug!("Sending to socket: {}", socket_id);
                        let _ = socket
                            .sender
                            .send(Message::Text(message.to_string().into()))
                            .await;
                    } else {
                        tracing::debug!("Socket not found in sockets map: {}", socket_id);
                    }
                }
            } else {
                tracing::debug!("No sockets found in channel: {}", channel);
            }
        } else {
            tracing::debug!("Namespace not found for app: {}", app_id);
        }
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
        let ns = self
            .namespaces
            .entry(app_id.to_string())
            .or_insert_with(|| Namespace::new(app_id.to_string()));

        // Add member to presence tracking
        let channel_presence = ns.presence.entry(channel.to_string()).or_default();
        channel_presence.insert(member.user_id.clone(), member.clone());

        // Track socket_id to user_id mapping for this channel
        let socket_to_user = ns
            .presence_socket_to_user
            .entry(channel.to_string())
            .or_default();
        socket_to_user.insert(socket_id.to_string(), member.user_id.clone());

        Ok(())
    }

    async fn remove_member(&self, app_id: &str, channel: &str, socket_id: &str) -> Result<()> {
        if let Some(ns) = self.namespaces.get(app_id) {
            // Get the user_id for this socket_id in this channel
            if let Some(socket_to_user) = ns.presence_socket_to_user.get(channel) {
                if let Some(user_id_entry) = socket_to_user.get(socket_id) {
                    let user_id = user_id_entry.value().clone();

                    // Remove the socket_id to user_id mapping
                    drop(user_id_entry);
                    socket_to_user.remove(socket_id);

                    // Check if this user has any other sockets in this channel
                    let has_other_sockets =
                        socket_to_user.iter().any(|entry| entry.value() == &user_id);

                    // Only remove the member if they have no other sockets in this channel
                    if !has_other_sockets && let Some(channel_presence) = ns.presence.get(channel) {
                        channel_presence.remove(&user_id);

                        // Clean up empty presence map
                        if channel_presence.is_empty() {
                            drop(channel_presence);
                            ns.presence.remove(channel);
                        }
                    }
                }

                // Clean up empty socket_to_user map
                if socket_to_user.is_empty() {
                    drop(socket_to_user);
                    ns.presence_socket_to_user.remove(channel);
                }
            }
        }
        Ok(())
    }

    async fn get_channel_members(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, PresenceMember>> {
        if let Some(ns) = self.namespaces.get(app_id) {
            if let Some(channel_presence) = ns.presence.get(channel) {
                Ok(channel_presence
                    .iter()
                    .map(|entry| (entry.key().clone(), entry.value().clone()))
                    .collect())
            } else {
                Ok(HashMap::new())
            }
        } else {
            Ok(HashMap::new())
        }
    }

    async fn get_channel_members_count(&self, app_id: &str, channel: &str) -> Result<usize> {
        if let Some(ns) = self.namespaces.get(app_id) {
            if let Some(channel_presence) = ns.presence.get(channel) {
                Ok(channel_presence.len())
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    // ===== User Tracking Methods =====

    async fn add_user(&self, app_id: &str, user_id: &str, socket_id: &str) -> Result<()> {
        let ns = self
            .namespaces
            .entry(app_id.to_string())
            .or_insert_with(|| Namespace::new(app_id.to_string()));
        let entry = ns.users.entry(user_id.to_string()).or_default();
        entry.insert(socket_id.to_string());
        Ok(())
    }

    async fn remove_user(&self, app_id: &str, user_id: &str, socket_id: &str) -> Result<()> {
        if let Some(ns) = self.namespaces.get(app_id) {
            let mut empty = false;
            if let Some(entry) = ns.users.get_mut(user_id) {
                entry.remove(socket_id);
                empty = entry.is_empty();
            }
            if empty {
                ns.users.remove(user_id);
            }
        }
        Ok(())
    }

    async fn get_user_sockets(&self, app_id: &str, user_id: &str) -> Result<Vec<String>> {
        if let Some(ns) = self.namespaces.get(app_id) {
            if let Some(ids) = ns.users.get(user_id) {
                Ok(ids.iter().map(|id| id.key().clone()).collect())
            } else {
                Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }

    async fn terminate_user_connections(&self, app_id: &str, user_id: &str) -> Result<()> {
        if let Some(ns) = self.namespaces.get(app_id)
            && let Some(user_sockets) = ns.users.get(user_id)
        {
            for socket_id in user_sockets.iter() {
                if let Some(socket) = ns.sockets.get(socket_id.key()) {
                    // Send error and close
                    let msg = serde_json::json!({
                        "event": "pusher:error",
                        "data": {
                            "code": 4009,
                            "message": "You got disconnected by the app."
                        }
                    })
                    .to_string();
                    let _ = socket.sender.send(Message::Text(msg.into())).await;
                    // In a real implementation we might signal the closer
                }
            }
        }
        Ok(())
    }

    // ===== Lifecycle Methods =====

    async fn disconnect(&self) -> Result<()> {
        self.namespaces.clear();
        Ok(())
    }

    // ===== Legacy/Helper Methods =====

    async fn add_to_channel(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: String,
    ) -> Result<usize> {
        let ns = self
            .namespaces
            .entry(app_id.to_string())
            .or_insert_with(|| Namespace::new(app_id.to_string()));
        Ok(ns.add_to_channel(channel, socket_id).await)
    }

    async fn remove_from_channel(
        &self,
        app_id: &str,
        channel: &str,
        socket_id: &str,
    ) -> Result<usize> {
        if let Some(ns) = self.namespaces.get(app_id) {
            Ok(ns.remove_from_channel(channel, socket_id).await)
        } else {
            Ok(0)
        }
    }

    async fn get_sockets(&self, app_id: &str) -> Result<HashMap<String, Socket>> {
        if let Some(ns) = self.namespaces.get(app_id) {
            Ok(ns
                .sockets
                .iter()
                .map(|k| (k.key().clone(), k.value().clone()))
                .collect())
        } else {
            Ok(HashMap::new())
        }
    }

    async fn get_channels(&self, app_id: &str) -> Result<HashMap<String, HashSet<String>>> {
        if let Some(ns) = self.namespaces.get(app_id) {
            Ok(ns
                .channels
                .iter()
                .map(|k| {
                    (
                        k.key().clone(),
                        k.value().iter().map(|s| s.clone()).collect(),
                    )
                })
                .collect())
        } else {
            Ok(HashMap::new())
        }
    }

    async fn get_channel_sockets(
        &self,
        app_id: &str,
        channel: &str,
    ) -> Result<HashMap<String, Socket>> {
        if let Some(ns) = self.namespaces.get(app_id) {
            if let Some(socket_ids) = ns.channels.get(channel) {
                let mut sockets = HashMap::new();
                for id in socket_ids.iter() {
                    if let Some(socket) = ns.sockets.get(id.key()) {
                        sockets.insert(id.key().clone(), socket.clone());
                    }
                }
                Ok(sockets)
            } else {
                Ok(HashMap::new())
            }
        } else {
            Ok(HashMap::new())
        }
    }

    async fn clear_namespace(&self, namespace_id: &str) -> Result<()> {
        self.namespaces.remove(namespace_id);
        Ok(())
    }

    async fn clear_namespaces(&self) -> Result<()> {
        self.namespaces.clear();
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
