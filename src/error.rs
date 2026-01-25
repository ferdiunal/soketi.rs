use serde::{Deserialize, Serialize};

/// Main error type for the Pusher server
///
/// This enum represents all possible errors that can occur in the Pusher server.
/// Each error variant maps to a specific Pusher protocol error code.
///
/// # Error Codes
///
/// The Pusher protocol defines specific error codes for different error conditions:
/// - 4001: App not found
/// - 4003: App disabled
/// - 4009: Authentication failed
/// - 4100: Connection limit reached
/// - 4200: Server closing
/// - 4201: Connection timeout
/// - 4301: Client messages disabled
/// - 4302: Generic server error
///
/// # Examples
///
/// ## Creating and using errors
/// ```
/// use soketi_rs::error::PusherError;
///
/// let error = PusherError::AppNotFound("app-123".to_string());
/// assert_eq!(error.to_pusher_code(), 4001);
///
/// let error_msg = error.to_error_message(Some("test-channel"));
/// assert_eq!(error_msg.event, "pusher:error");
/// assert_eq!(error_msg.data.code, 4001);
/// ```
///
/// **Validates: Requirements 13.1-13.9**
#[derive(Debug, thiserror::Error)]
pub enum PusherError {
    #[error("App not found: {0}")]
    AppNotFound(String),

    #[error("App disabled: {0}")]
    AppDisabled(String),

    #[error("Connection limit reached")]
    ConnectionLimitReached,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Server closing")]
    ServerClosing,

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("User authentication timeout")]
    UserAuthenticationTimeout,

    #[error("Client messages disabled")]
    ClientMessagesDisabled,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Adapter error: {0}")]
    AdapterError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Webhook error: {0}")]
    WebhookError(String),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl PusherError {
    /// Convert error to Pusher protocol error code
    /// Based on Pusher protocol specification
    pub fn to_pusher_code(&self) -> u16 {
        match self {
            PusherError::AppNotFound(_) => 4001,
            PusherError::AppDisabled(_) => 4003,
            PusherError::AuthenticationFailed(_) => 4009,
            PusherError::ConnectionLimitReached => 4100,
            PusherError::ServerClosing => 4200,
            PusherError::ConnectionTimeout => 4201,
            PusherError::ClientMessagesDisabled => 4301,
            PusherError::ServerError(_) => 4302,
            PusherError::UserAuthenticationTimeout => 4009,
            _ => 4302, // Generic server error
        }
    }

    /// Convert error to Pusher protocol error message
    pub fn to_error_message(&self, channel: Option<&str>) -> PusherErrorMessage {
        PusherErrorMessage {
            event: "pusher:error".to_string(),
            data: PusherErrorData {
                code: self.to_pusher_code(),
                message: self.to_string(),
            },
            channel: channel.map(|s| s.to_string()),
        }
    }
}

/// Pusher error message structure for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PusherErrorMessage {
    pub event: String,
    pub data: PusherErrorData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PusherErrorData {
    pub code: u16,
    pub message: String,
}

// Implement From traits for common error types
impl From<std::io::Error> for PusherError {
    fn from(err: std::io::Error) -> Self {
        PusherError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for PusherError {
    fn from(err: serde_json::Error) -> Self {
        PusherError::SerializationError(err.to_string())
    }
}

impl From<redis::RedisError> for PusherError {
    fn from(err: redis::RedisError) -> Self {
        PusherError::RedisError(err.to_string())
    }
}

impl From<sqlx::Error> for PusherError {
    fn from(err: sqlx::Error) -> Self {
        PusherError::DatabaseError(err.to_string())
    }
}

impl From<prometheus::Error> for PusherError {
    fn from(err: prometheus::Error) -> Self {
        PusherError::ServerError(format!("Prometheus error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for PusherError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        PusherError::SerializationError(format!("UTF-8 conversion error: {}", err))
    }
}

/// Result type alias for Pusher operations
pub type Result<T> = std::result::Result<T, PusherError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            PusherError::AppNotFound("test".to_string()).to_pusher_code(),
            4001
        );
        assert_eq!(
            PusherError::AppDisabled("test".to_string()).to_pusher_code(),
            4003
        );
        assert_eq!(
            PusherError::AuthenticationFailed("test".to_string()).to_pusher_code(),
            4009
        );
        assert_eq!(PusherError::ConnectionLimitReached.to_pusher_code(), 4100);
        assert_eq!(PusherError::ServerClosing.to_pusher_code(), 4200);
        assert_eq!(PusherError::ConnectionTimeout.to_pusher_code(), 4201);
        assert_eq!(PusherError::ClientMessagesDisabled.to_pusher_code(), 4301);
        assert_eq!(
            PusherError::ServerError("test".to_string()).to_pusher_code(),
            4302
        );
    }

    #[test]
    fn test_error_message_generation() {
        let error = PusherError::AppNotFound("test_app".to_string());
        let message = error.to_error_message(Some("test-channel"));

        assert_eq!(message.event, "pusher:error");
        assert_eq!(message.data.code, 4001);
        assert!(message.data.message.contains("test_app"));
        assert_eq!(message.channel, Some("test-channel".to_string()));
    }

    #[test]
    fn test_error_message_without_channel() {
        let error = PusherError::ConnectionLimitReached;
        let message = error.to_error_message(None);

        assert_eq!(message.event, "pusher:error");
        assert_eq!(message.data.code, 4100);
        assert_eq!(message.channel, None);
    }

    #[test]
    fn test_error_from_conversions() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());

        let pusher_error: PusherError = json_error.unwrap_err().into();
        assert!(matches!(pusher_error, PusherError::SerializationError(_)));
    }
}
