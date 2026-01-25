use serde::{Deserialize, Serialize};

/// Core Pusher protocol message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PusherMessage {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

impl PusherMessage {
    pub fn new(event: String) -> Self {
        Self {
            event,
            data: None,
            channel: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_channel(mut self, channel: String) -> Self {
        self.channel = Some(channel);
        self
    }
}

/// Connection established message data
#[derive(Debug, Serialize)]
pub struct ConnectionEstablishedData {
    pub socket_id: String,
    pub activity_timeout: u64,
}

/// Subscription succeeded message data
#[derive(Debug, Serialize)]
pub struct SubscriptionSucceededData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<PresenceData>,
}

/// Presence data for presence channels
#[derive(Debug, Serialize)]
pub struct PresenceData {
    pub ids: Vec<String>,
    pub hash: serde_json::Value,
    pub count: usize,
}

/// Member added/removed event data
#[derive(Debug, Serialize)]
pub struct MemberEventData {
    pub user_id: String,
    pub user_info: serde_json::Value,
}
