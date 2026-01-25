use soketi_rs::app::*;
use soketi_rs::config::*;
use soketi_rs::error::*;

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
fn test_app_creation() {
    let app = App::new(
        "test_id".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    assert_eq!(app.id, "test_id");
    assert_eq!(app.key, "test_key");
    assert_eq!(app.secret, "test_secret");
    assert_eq!(app.enabled, true);
    assert_eq!(app.enable_client_messages, false);
}

#[test]
fn test_app_webhook_checks() {
    let mut app = App::new(
        "test_id".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );

    // Initially no webhooks
    assert_eq!(app.has_client_event_webhooks(), false);
    assert_eq!(app.has_channel_occupied_webhooks(), false);

    // Add a webhook
    app.webhooks.push(Webhook {
        url: Some("http://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string(), "channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    });

    assert_eq!(app.has_client_event_webhooks(), true);
    assert_eq!(app.has_channel_occupied_webhooks(), true);
    assert_eq!(app.has_member_added_webhooks(), false);
}

#[test]
fn test_presence_member_serialization() {
    let member = PresenceMember {
        user_id: "user123".to_string(),
        user_info: serde_json::json!({"name": "John Doe"}),
    };

    let json = serde_json::to_string(&member).unwrap();
    assert!(json.contains("user123"));
    assert!(json.contains("John Doe"));

    let deserialized: PresenceMember = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, "user123");
}

#[test]
fn test_user_serialization() {
    let user = User {
        id: "user456".to_string(),
        data: serde_json::json!({"email": "user@example.com", "role": "admin"}),
    };

    let json = serde_json::to_string(&user).unwrap();
    assert!(json.contains("user456"));
    assert!(json.contains("user@example.com"));

    let deserialized: User = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "user456");
}

#[test]
fn test_server_config_defaults() {
    let config = ServerConfig::default();

    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 6001);
    assert_eq!(config.debug, false);
    assert_eq!(config.mode, ServerMode::Full);
    assert_eq!(config.shutdown_grace_period_ms, 3000);
}

#[test]
fn test_adapter_config_defaults() {
    let config = AdapterConfig::default();

    assert_eq!(config.driver, AdapterDriver::Local);
    assert_eq!(config.redis.host, "127.0.0.1");
    assert_eq!(config.redis.port, 6379);
}

#[test]
fn test_cache_config_defaults() {
    let config = CacheConfig::default();

    assert_eq!(config.driver, CacheDriver::Memory);
}

#[test]
fn test_rate_limiter_config_defaults() {
    let config = RateLimiterConfig::default();

    assert_eq!(config.driver, RateLimiterDriver::Local);
}

#[test]
fn test_queue_config_defaults() {
    let config = QueueConfig::default();

    assert_eq!(config.driver, QueueDriver::Sync);
}

#[test]
fn test_metrics_config_defaults() {
    let config = MetricsConfig::default();

    assert_eq!(config.enabled, false);
    assert_eq!(config.driver, MetricsDriver::Prometheus);
    assert_eq!(config.port, 9601);
}

#[test]
fn test_channel_limits_defaults() {
    let limits = ChannelLimits::default();

    assert_eq!(limits.max_name_length, 200);
    assert_eq!(limits.cache_ttl_seconds, 3600);
}

#[test]
fn test_event_limits_defaults() {
    let limits = EventLimits::default();

    assert_eq!(limits.max_channels_at_once, 100);
    assert_eq!(limits.max_name_length, 200);
    assert_eq!(limits.max_payload_in_kb, 100.0);
    assert_eq!(limits.max_batch_size, 10);
}

#[test]
fn test_presence_limits_defaults() {
    let limits = PresenceLimits::default();

    assert_eq!(limits.max_members_per_channel, 100);
    assert_eq!(limits.max_member_size_in_kb, 2.0);
}
