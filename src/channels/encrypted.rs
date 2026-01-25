use crate::adapters::Adapter;
use crate::app::App;
use crate::channels::private::PrivateChannelManager;
use crate::channels::{ChannelManager, JoinResponse, PusherMessage};
use crate::namespace::Socket;
use async_trait::async_trait;
use std::sync::Arc;

/// EncryptedPrivateChannelManager handles subscriptions to encrypted private channels
///
/// Encrypted private channels (channels starting with "private-encrypted-") provide
/// end-to-end encryption for messages. The encryption is handled entirely on the
/// client side - the server never sees the decrypted message content.
///
/// From the server's perspective, encrypted private channels work exactly like
/// regular private channels:
/// - They require HMAC-SHA256 signature authentication
/// - The server validates the auth signature using the same mechanism as private channels
/// - The server routes encrypted messages without decrypting them
/// - The server has no access to the encryption keys
///
/// The encryption/decryption is performed by the Pusher client libraries using
/// a shared secret that is never transmitted to the server. This provides true
/// end-to-end encryption where only the clients can read the message content.
///
/// Requirements: 7.3
pub struct EncryptedPrivateChannelManager {
    private_manager: PrivateChannelManager,
}

impl EncryptedPrivateChannelManager {
    /// Create a new EncryptedPrivateChannelManager
    ///
    /// # Arguments
    /// * `adapter` - The adapter to use for channel management
    ///
    /// # Returns
    /// A new EncryptedPrivateChannelManager instance
    pub fn new(adapter: Arc<dyn Adapter>) -> Self {
        Self {
            private_manager: PrivateChannelManager::new(adapter),
        }
    }
}

#[async_trait]
impl ChannelManager for EncryptedPrivateChannelManager {
    /// Join an encrypted private channel
    ///
    /// This method delegates to the PrivateChannelManager since encrypted channels
    /// use the same authentication mechanism as private channels. The only difference
    /// is that clients encrypt/decrypt messages on their end - the server just routes
    /// the encrypted payloads without ever seeing the plaintext.
    ///
    /// # Arguments
    /// * `app` - The application the socket belongs to
    /// * `socket` - The socket attempting to join
    /// * `channel` - The channel name to join (must start with "private-encrypted-")
    /// * `message` - The subscription message containing the auth signature
    ///
    /// # Returns
    /// A JoinResponse indicating success or failure
    ///
    /// Requirements: 7.3, 7.5
    async fn join(
        &self,
        app: &App,
        socket: &Socket,
        channel: &str,
        message: Option<PusherMessage>,
    ) -> JoinResponse {
        // Encrypted private channels use the same server-side authentication as private channels.
        // The encryption is handled entirely client-side using a shared secret that never
        // reaches the server. The server simply validates the auth signature and routes
        // encrypted messages without decrypting them.
        self.private_manager
            .join(app, socket, channel, message)
            .await
    }
}
