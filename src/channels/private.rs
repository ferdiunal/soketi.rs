use crate::adapters::Adapter;
use crate::app::App;
use crate::channels::{ChannelManager, JoinResponse, PusherMessage};
use crate::namespace::Socket;
use async_trait::async_trait;
use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

/// PrivateChannelManager handles subscriptions to private channels
///
/// Private channels require authentication via HMAC-SHA256 signatures.
/// The client must provide a valid auth signature that proves they have
/// permission to subscribe to the channel.
///
/// Requirements: 7.2, 7.5
pub struct PrivateChannelManager {
    adapter: Arc<dyn Adapter>,
}

impl PrivateChannelManager {
    pub fn new(adapter: Arc<dyn Adapter>) -> Self {
        Self { adapter }
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
        format!("{}:{}", socket_id, channel)
    }

    fn sign(&self, secret: &str, data: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

#[async_trait]
impl ChannelManager for PrivateChannelManager {
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

        // After successful authentication, add socket to channel via adapter
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
}
