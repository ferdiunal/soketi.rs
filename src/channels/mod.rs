use crate::app::App;
use crate::namespace::Socket;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResponse {
    pub success: bool,
    pub error_code: Option<u16>,
    pub error_message: Option<String>,
    pub auth_error: bool,
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PusherMessage {
    pub event: String,
    pub data: Option<Value>,
    pub channel: Option<String>,
    pub socket_id: Option<String>,
}

pub mod encrypted;
pub mod presence;
pub mod private;
pub mod public;

#[async_trait]
pub trait ChannelManager: Send + Sync {
    async fn join(
        &self,
        app: &App,
        socket: &Socket,
        channel: &str,
        message: Option<PusherMessage>,
    ) -> JoinResponse;
}
