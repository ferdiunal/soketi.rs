use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application configuration and credentials
///
/// Represents a Pusher application with its credentials, limits, and webhook configuration.
/// Applications are managed by AppManager implementations and can be stored in:
/// - Static configuration (ArrayAppManager)
/// - DynamoDB (DynamoDbAppManager)
/// - MySQL (MysqlAppManager)
/// - PostgreSQL (PostgresAppManager)
///
/// # Examples
///
/// ## Creating a basic app
/// ```
/// use soketi_rs::app::App;
///
/// let app = App::new(
///     "app-123".to_string(),
///     "app-key-123".to_string(),
///     "app-secret-123".to_string(),
/// );
///
/// assert_eq!(app.id, "app-123");
/// assert_eq!(app.key, "app-key-123");
/// assert!(app.enabled);
/// ```
///
/// ## Creating an app with limits
/// ```
/// use soketi_rs::app::App;
///
/// let mut app = App::new(
///     "app-123".to_string(),
///     "app-key-123".to_string(),
///     "app-secret-123".to_string(),
/// );
///
/// app.max_connections = Some(1000);
/// app.enable_client_messages = true;
/// app.max_backend_events_per_second = Some(100);
/// ```
///
/// **Validates: Requirements 5.1-5.9, 12.1-12.7**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    /// Unique application identifier
    pub id: String,
    /// Public application key (used for authentication)
    pub key: String,
    /// Secret key for signing (never exposed to clients)
    pub secret: String,
    /// Maximum number of concurrent connections (None = unlimited)
    #[serde(default)]
    pub max_connections: Option<u64>,
    /// Whether client-to-client messages are enabled
    #[serde(default)]
    pub enable_client_messages: bool,
    /// Whether the application is enabled (disabled apps reject all connections)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Maximum backend events per second (None = unlimited)
    #[serde(default)]
    pub max_backend_events_per_second: Option<u64>,
    /// Maximum client events per second (None = unlimited)
    #[serde(default)]
    pub max_client_events_per_second: Option<u64>,
    /// Maximum read requests per second (None = unlimited)
    #[serde(default)]
    pub max_read_requests_per_second: Option<u64>,
    /// Webhook configurations for this app
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
    /// Maximum members per presence channel (None = use global default)
    #[serde(default)]
    pub max_presence_members_per_channel: Option<u64>,
    /// Maximum presence member size in KB (None = use global default)
    #[serde(default)]
    pub max_presence_member_size_in_kb: Option<f64>,
    /// Maximum channel name length (None = use global default)
    #[serde(default)]
    pub max_channel_name_length: Option<u64>,
    /// Maximum channels per event (None = use global default)
    #[serde(default)]
    pub max_event_channels_at_once: Option<u64>,
    /// Maximum event name length (None = use global default)
    #[serde(default)]
    pub max_event_name_length: Option<u64>,
    /// Maximum event payload size in KB (None = use global default)
    #[serde(default)]
    pub max_event_payload_in_kb: Option<f64>,
    /// Maximum batch size (None = use global default)
    #[serde(default)]
    pub max_event_batch_size: Option<u64>,
    /// Whether user authentication is required
    #[serde(default)]
    pub enable_user_authentication: bool,
}

fn default_true() -> bool {
    true
}

/// Webhook configuration for an application
///
/// Webhooks allow applications to receive notifications about events:
/// - channel_occupied: Channel transitions from 0 to 1+ subscribers
/// - channel_vacated: Channel transitions from 1+ to 0 subscribers
/// - member_added: Member joins a presence channel
/// - member_removed: Member leaves a presence channel
/// - client_event: Client sends an event
/// - cache_miss: Cache-enabled channel has no cached data
///
/// Webhooks can be sent to HTTP URLs or AWS Lambda functions.
///
/// # Examples
///
/// ## HTTP webhook
/// ```
/// use soketi_rs::app::Webhook;
///
/// let webhook = Webhook {
///     url: Some("https://example.com/webhooks".to_string()),
///     lambda_function: None,
///     event_types: vec!["channel_occupied".to_string(), "channel_vacated".to_string()],
///     headers: None,
///     filter: None,
///     lambda: None,
/// };
/// ```
///
/// ## Lambda webhook
/// ```
/// use soketi_rs::app::{Webhook, LambdaConfig};
///
/// let webhook = Webhook {
///     url: None,
///     lambda_function: Some("arn:aws:lambda:us-east-1:123456789:function:pusher-webhook".to_string()),
///     event_types: vec!["member_added".to_string()],
///     headers: None,
///     filter: None,
///     lambda: Some(LambdaConfig {
///         client_context: None,
///         async_invocation: true,
///     }),
/// };
/// ```
///
/// **Validates: Requirements 10.1-10.10**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// HTTP URL to send webhook to (mutually exclusive with lambda_function)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// AWS Lambda function ARN (mutually exclusive with url)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lambda_function: Option<String>,
    /// Event types to trigger this webhook
    pub event_types: Vec<String>,
    /// Custom HTTP headers to include in webhook requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Channel name filter for webhook events
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<WebhookFilter>,
    /// Lambda-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lambda: Option<LambdaConfig>,
}

/// Webhook channel name filter
///
/// Allows filtering webhook events by channel name prefix or suffix.
///
/// # Examples
///
/// ```
/// use soketi_rs::app::WebhookFilter;
///
/// // Only trigger for private channels
/// let filter = WebhookFilter {
///     channel_name_starts_with: Some("private-".to_string()),
///     channel_name_ends_with: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFilter {
    /// Only trigger webhook if channel name starts with this prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name_starts_with: Option<String>,
    /// Only trigger webhook if channel name ends with this suffix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_name_ends_with: Option<String>,
}

/// AWS Lambda webhook configuration
///
/// **Validates: Requirements 10.9**
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LambdaConfig {
    /// Client context to pass to Lambda function
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_context: Option<serde_json::Value>,
    /// Whether to invoke Lambda asynchronously (default: false)
    #[serde(default)]
    pub async_invocation: bool,
}

/// Presence member information
///
/// Contains user information for presence channel members.
///
/// # Examples
///
/// ```
/// use soketi_rs::app::PresenceMember;
/// use serde_json::json;
///
/// let member = PresenceMember {
///     user_id: "user-123".to_string(),
///     user_info: json!({
///         "name": "John Doe",
///         "email": "john@example.com"
///     }),
/// };
/// ```
///
/// **Validates: Requirements 7.4, 7.6**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMember {
    /// Unique user identifier
    pub user_id: String,
    /// Additional user information (arbitrary JSON)
    pub user_info: serde_json::Value,
}

/// User information for authenticated connections
///
/// Used with pusher:signin for user authentication.
///
/// # Examples
///
/// ```
/// use soketi_rs::app::User;
/// use serde_json::json;
///
/// let user = User {
///     id: "user-123".to_string(),
///     data: json!({
///         "name": "John Doe",
///         "role": "admin"
///     }),
/// };
/// ```
///
/// **Validates: Requirements 16.1-16.6**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: String,
    /// Additional user data (arbitrary JSON)
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl App {
    /// Create a new App with basic configuration
    ///
    /// Creates an app with the given ID, key, and secret.
    /// All optional fields are set to their defaults:
    /// - enabled: true
    /// - enable_client_messages: false
    /// - All limits: None (use global defaults)
    ///
    /// # Arguments
    /// * `id` - Unique application identifier
    /// * `key` - Public application key
    /// * `secret` - Secret key for signing
    ///
    /// # Examples
    ///
    /// ```
    /// use soketi_rs::app::App;
    ///
    /// let app = App::new(
    ///     "app-123".to_string(),
    ///     "key-123".to_string(),
    ///     "secret-123".to_string(),
    /// );
    ///
    /// assert!(app.enabled);
    /// assert!(!app.enable_client_messages);
    /// ```
    pub fn new(id: String, key: String, secret: String) -> Self {
        Self {
            id,
            key,
            secret,
            max_connections: None,
            enable_client_messages: false,
            enabled: true,
            max_backend_events_per_second: None,
            max_client_events_per_second: None,
            max_read_requests_per_second: None,
            webhooks: vec![],
            max_presence_members_per_channel: None,
            max_presence_member_size_in_kb: None,
            max_channel_name_length: None,
            max_event_channels_at_once: None,
            max_event_name_length: None,
            max_event_payload_in_kb: None,
            max_event_batch_size: None,
            enable_user_authentication: false,
        }
    }

    /// Check if the app has webhooks configured for client events
    ///
    /// # Returns
    /// `true` if any webhook is configured for "client_event" events
    pub fn has_client_event_webhooks(&self) -> bool {
        self.webhooks
            .iter()
            .any(|w| w.event_types.contains(&"client_event".to_string()))
    }

    /// Check if the app has webhooks configured for channel occupied events
    ///
    /// # Returns
    /// `true` if any webhook is configured for "channel_occupied" events
    pub fn has_channel_occupied_webhooks(&self) -> bool {
        self.webhooks
            .iter()
            .any(|w| w.event_types.contains(&"channel_occupied".to_string()))
    }

    /// Check if the app has webhooks configured for channel vacated events
    ///
    /// # Returns
    /// `true` if any webhook is configured for "channel_vacated" events
    pub fn has_channel_vacated_webhooks(&self) -> bool {
        self.webhooks
            .iter()
            .any(|w| w.event_types.contains(&"channel_vacated".to_string()))
    }

    /// Check if the app has webhooks configured for member added events
    ///
    /// # Returns
    /// `true` if any webhook is configured for "member_added" events
    pub fn has_member_added_webhooks(&self) -> bool {
        self.webhooks
            .iter()
            .any(|w| w.event_types.contains(&"member_added".to_string()))
    }

    /// Check if the app has webhooks configured for member removed events
    ///
    /// # Returns
    /// `true` if any webhook is configured for "member_removed" events
    pub fn has_member_removed_webhooks(&self) -> bool {
        self.webhooks
            .iter()
            .any(|w| w.event_types.contains(&"member_removed".to_string()))
    }

    /// Check if the app has webhooks configured for cache miss events
    ///
    /// # Returns
    /// `true` if any webhook is configured for "cache_miss" events
    pub fn has_cache_missed_webhooks(&self) -> bool {
        self.webhooks
            .iter()
            .any(|w| w.event_types.contains(&"cache_miss".to_string()))
    }
}
