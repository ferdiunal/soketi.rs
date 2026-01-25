use crate::app::PresenceMember;
use axum::extract::ws::Message;
use dashmap::{DashMap, DashSet};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Socket {
    pub id: String,
    pub sender: mpsc::Sender<Message>,
}

#[derive(Clone)]
pub struct Namespace {
    pub app_id: String,
    pub channels: Arc<DashMap<String, DashSet<String>>>, // channel -> socket_ids
    pub sockets: Arc<DashMap<String, Socket>>,           // socket_id -> Socket
    pub users: Arc<DashMap<String, DashSet<String>>>,    // user_id -> socket_ids
    pub presence: Arc<DashMap<String, DashMap<String, PresenceMember>>>, // channel -> (user_id -> PresenceMember)
    pub presence_socket_to_user: Arc<DashMap<String, DashMap<String, String>>>, // channel -> (socket_id -> user_id)
}

impl Namespace {
    pub fn new(app_id: String) -> Self {
        Self {
            app_id,
            channels: Arc::new(DashMap::new()),
            sockets: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
            presence: Arc::new(DashMap::new()),
            presence_socket_to_user: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_socket(&self, socket: Socket) {
        self.sockets.insert(socket.id.clone(), socket);
    }

    pub async fn remove_socket(&self, socket_id: &str) {
        // Remove from channels
        // Iterating over all channels is expensive.
        // Ideally we should track which channels a socket is in.
        // But for parity with TS implementation which iterates... wait, TS implementation iterates?
        // TS: removeFromChannel(wsId, [...this.channels.keys()]) -> removes from all channels.
        // Optimization: Socket should know its channels.
        // For now, let's keep it simple or follow TS.

        let channels_to_remove: Vec<String> = self
            .channels
            .iter()
            .filter(|entry| entry.value().contains(socket_id))
            .map(|entry| entry.key().clone())
            .collect();

        for channel in channels_to_remove {
            self.remove_from_channel(&channel, socket_id).await;
        }

        self.sockets.remove(socket_id);
    }

    pub async fn add_to_channel(&self, channel: &str, socket_id: String) -> usize {
        let entry = self.channels.entry(channel.to_string()).or_default();
        entry.insert(socket_id);
        entry.len()
    }

    pub async fn remove_from_channel(&self, channel: &str, socket_id: &str) -> usize {
        let mut count = 0;
        if let Some(entry) = self.channels.get_mut(channel) {
            entry.remove(socket_id);
            count = entry.len();
        }
        if count == 0 {
            self.channels.remove(channel);
        }
        count
    }

    pub async fn get_sockets(&self) -> DashMap<String, Socket> {
        // returning a clone of the map is expensive?
        // In TS it returns Promise<Map>.
        // Here getting direct access to DashMap via Arc is better.
        // But for API signature parity.. let's just return reference to inner map if possible or clone.
        // Since we expose Arc, we can just return Arc or clone the Arc.
        // But the trait might require HashMap snapshot.
        // Let's defer to adapter layer.
        (*self.sockets).clone()
    }
}
