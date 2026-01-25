use crate::app::App;
use crate::config::ServerConfig;
use crate::error::{PusherError, Result};
use serde_json::Value;

/// Validates channel name length against configured limits
///
/// Checks both app-specific and global limits. App-specific limits take precedence.
///
/// # Arguments
/// * `channel_name` - The channel name to validate
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(PusherError::ValidationError)` if channel name exceeds limits
pub fn validate_channel_name_length(
    channel_name: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    let max_length = app
        .and_then(|a| a.max_channel_name_length)
        .unwrap_or(config.channel_limits.max_name_length);

    if channel_name.len() as u64 > max_length {
        return Err(PusherError::ValidationError(format!(
            "Channel name exceeds maximum length of {} characters",
            max_length
        )));
    }

    Ok(())
}

/// Validates event name length against configured limits
///
/// Checks both app-specific and global limits. App-specific limits take precedence.
///
/// # Arguments
/// * `event_name` - The event name to validate
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(PusherError::ValidationError)` if event name exceeds limits
pub fn validate_event_name_length(
    event_name: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    let max_length = app
        .and_then(|a| a.max_event_name_length)
        .unwrap_or(config.event_limits.max_name_length);

    if event_name.len() as u64 > max_length {
        return Err(PusherError::ValidationError(format!(
            "Event name exceeds maximum length of {} characters",
            max_length
        )));
    }

    Ok(())
}

/// Validates payload size against configured limits
///
/// Checks both app-specific and global limits. App-specific limits take precedence.
/// The payload is measured in kilobytes.
///
/// # Arguments
/// * `payload` - The payload string to validate
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(PusherError::ValidationError)` if payload exceeds size limits
pub fn validate_payload_size(
    payload: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    let max_size_kb = app
        .and_then(|a| a.max_event_payload_in_kb)
        .unwrap_or(config.event_limits.max_payload_in_kb);

    let payload_size_kb = payload.len() as f64 / 1024.0;

    if payload_size_kb > max_size_kb {
        return Err(PusherError::ValidationError(format!(
            "Payload size {:.2} KB exceeds maximum of {:.2} KB",
            payload_size_kb, max_size_kb
        )));
    }

    Ok(())
}

/// Validates batch size against configured limits
///
/// Checks both app-specific and global limits. App-specific limits take precedence.
///
/// # Arguments
/// * `batch_size` - The number of events in the batch
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(PusherError::ValidationError)` if batch size exceeds limits
pub fn validate_batch_size(
    batch_size: usize,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    let max_batch_size = app
        .and_then(|a| a.max_event_batch_size)
        .unwrap_or(config.event_limits.max_batch_size);

    if batch_size as u64 > max_batch_size {
        return Err(PusherError::ValidationError(format!(
            "Batch size {} exceeds maximum of {}",
            batch_size, max_batch_size
        )));
    }

    Ok(())
}

/// Validates channel count against configured limits
///
/// Checks both app-specific and global limits. App-specific limits take precedence.
/// This validates the number of channels an event can be sent to at once.
///
/// # Arguments
/// * `channel_count` - The number of channels
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(PusherError::ValidationError)` if channel count exceeds limits
pub fn validate_channel_count(
    channel_count: usize,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    let max_channels = app
        .and_then(|a| a.max_event_channels_at_once)
        .unwrap_or(config.event_limits.max_channels_at_once);

    if channel_count as u64 > max_channels {
        return Err(PusherError::ValidationError(format!(
            "Channel count {} exceeds maximum of {}",
            channel_count, max_channels
        )));
    }

    Ok(())
}

/// Validates JSON structure
///
/// Attempts to parse the string as JSON to ensure it's valid.
///
/// # Arguments
/// * `json_str` - The JSON string to validate
///
/// # Returns
/// * `Ok(Value)` if JSON is valid, returning the parsed value
/// * `Err(PusherError::ValidationError)` if JSON is malformed
pub fn validate_json_structure(json_str: &str) -> Result<Value> {
    serde_json::from_str(json_str)
        .map_err(|e| PusherError::ValidationError(format!("Invalid JSON structure: {}", e)))
}

/// Validates that required fields are present in a JSON value
///
/// # Arguments
/// * `value` - The JSON value to check
/// * `required_fields` - Slice of field names that must be present
///
/// # Returns
/// * `Ok(())` if all required fields are present
/// * `Err(PusherError::ValidationError)` if any required field is missing
pub fn validate_required_fields(value: &Value, required_fields: &[&str]) -> Result<()> {
    if let Some(obj) = value.as_object() {
        for field in required_fields {
            if !obj.contains_key(*field) {
                return Err(PusherError::ValidationError(format!(
                    "Missing required field: {}",
                    field
                )));
            }
        }
        Ok(())
    } else {
        Err(PusherError::ValidationError(
            "Expected JSON object".to_string(),
        ))
    }
}

/// Validates a complete event message
///
/// This is a convenience function that validates multiple aspects of an event:
/// - Event name length
/// - Payload size
/// - JSON structure (if payload is provided)
///
/// # Arguments
/// * `event_name` - The event name
/// * `payload` - Optional payload string
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if all validations pass
/// * `Err(PusherError::ValidationError)` if any validation fails
pub fn validate_event(
    event_name: &str,
    payload: Option<&str>,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    validate_event_name_length(event_name, app, config)?;

    if let Some(payload_str) = payload {
        validate_payload_size(payload_str, app, config)?;
        validate_json_structure(payload_str)?;
    }

    Ok(())
}

/// Validates a batch of events
///
/// This validates:
/// - Batch size
/// - Each individual event in the batch
/// - Channel count across all events
///
/// # Arguments
/// * `events` - Slice of (event_name, channels, payload) tuples
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if all validations pass
/// * `Err(PusherError::ValidationError)` if any validation fails
pub fn validate_batch_events(
    events: &[(&str, &[String], Option<&str>)],
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    // Validate batch size
    validate_batch_size(events.len(), app, config)?;

    // Validate each event
    for (event_name, channels, payload) in events {
        validate_event(event_name, *payload, app, config)?;
        validate_channel_count(channels.len(), app, config)?;

        // Validate each channel name
        for channel in *channels {
            validate_channel_name_length(channel, app, config)?;
        }
    }

    Ok(())
}

/// Validates a subscription request
///
/// This validates:
/// - Channel name length
/// - Required fields in the subscription data
///
/// # Arguments
/// * `channel_name` - The channel to subscribe to
/// * `auth_data` - Optional authentication data as JSON value
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if all validations pass
/// * `Err(PusherError::ValidationError)` if any validation fails
pub fn validate_subscription(
    channel_name: &str,
    auth_data: Option<&Value>,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    validate_channel_name_length(channel_name, app, config)?;

    // For private and presence channels, validate auth field
    if (channel_name.starts_with("private-") || channel_name.starts_with("presence-"))
        && let Some(data) = auth_data
    {
        validate_required_fields(data, &["auth"])?;

        // For presence channels, also require channel_data
        if channel_name.starts_with("presence-") {
            validate_required_fields(data, &["channel_data"])?;
        }
    }

    Ok(())
}

/// Validates presence member data
///
/// Checks the size of the presence member data against configured limits.
///
/// # Arguments
/// * `member_data` - The member data as a JSON string
/// * `app` - Optional app with per-app limits
/// * `config` - Server configuration with global limits
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(PusherError::ValidationError)` if member data exceeds size limits
pub fn validate_presence_member_size(
    member_data: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()> {
    let max_size_kb = app
        .and_then(|a| a.max_presence_member_size_in_kb)
        .unwrap_or(config.presence.max_member_size_in_kb);

    let data_size_kb = member_data.len() as f64 / 1024.0;

    if data_size_kb > max_size_kb {
        return Err(PusherError::ValidationError(format!(
            "Presence member data size {:.2} KB exceeds maximum of {:.2} KB",
            data_size_kb, max_size_kb
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::config::ServerConfig;

    fn create_test_config() -> ServerConfig {
        ServerConfig::default()
    }

    fn create_test_app() -> App {
        App::new(
            "test_id".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
        )
    }

    #[test]
    fn test_validate_channel_name_length_success() {
        let config = create_test_config();
        let result = validate_channel_name_length("test-channel", None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_channel_name_length_exceeds_limit() {
        let config = create_test_config();
        let long_name = "a".repeat(201); // Default limit is 200
        let result = validate_channel_name_length(&long_name, None, &config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PusherError::ValidationError(_)
        ));
    }

    #[test]
    fn test_validate_channel_name_length_with_app_limit() {
        let config = create_test_config();
        let mut app = create_test_app();
        app.max_channel_name_length = Some(10);

        let result = validate_channel_name_length("short", Some(&app), &config);
        assert!(result.is_ok());

        let result = validate_channel_name_length("this-is-too-long", Some(&app), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_event_name_length_success() {
        let config = create_test_config();
        let result = validate_event_name_length("test-event", None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_name_length_exceeds_limit() {
        let config = create_test_config();
        let long_name = "a".repeat(201); // Default limit is 200
        let result = validate_event_name_length(&long_name, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_payload_size_success() {
        let config = create_test_config();
        let payload = r#"{"message": "Hello, World!"}"#;
        let result = validate_payload_size(payload, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_payload_size_exceeds_limit() {
        let config = create_test_config();
        // Create a payload larger than 100 KB (default limit)
        let large_payload = "a".repeat(110 * 1024);
        let result = validate_payload_size(&large_payload, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_payload_size_with_app_limit() {
        let config = create_test_config();
        let mut app = create_test_app();
        app.max_event_payload_in_kb = Some(1.0); // 1 KB limit

        let small_payload = "small";
        let result = validate_payload_size(small_payload, Some(&app), &config);
        assert!(result.is_ok());

        let large_payload = "a".repeat(2048); // 2 KB
        let result = validate_payload_size(&large_payload, Some(&app), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_batch_size_success() {
        let config = create_test_config();
        let result = validate_batch_size(5, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_batch_size_exceeds_limit() {
        let config = create_test_config();
        let result = validate_batch_size(11, None, &config); // Default limit is 10
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_channel_count_success() {
        let config = create_test_config();
        let result = validate_channel_count(50, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_channel_count_exceeds_limit() {
        let config = create_test_config();
        let result = validate_channel_count(101, None, &config); // Default limit is 100
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_structure_success() {
        let json = r#"{"key": "value", "number": 42}"#;
        let result = validate_json_structure(json);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_object());
    }

    #[test]
    fn test_validate_json_structure_invalid() {
        let invalid_json = r#"{"key": "value", invalid}"#;
        let result = validate_json_structure(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required_fields_success() {
        let json = serde_json::json!({
            "name": "test",
            "value": 42,
            "enabled": true
        });

        let result = validate_required_fields(&json, &["name", "value"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_fields_missing() {
        let json = serde_json::json!({
            "name": "test"
        });

        let result = validate_required_fields(&json, &["name", "value"]);
        assert!(result.is_err());

        if let Err(PusherError::ValidationError(msg)) = result {
            assert!(msg.contains("value"));
        }
    }

    #[test]
    fn test_validate_required_fields_not_object() {
        let json = serde_json::json!([1, 2, 3]);

        let result = validate_required_fields(&json, &["name"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_event_success() {
        let config = create_test_config();
        let payload = r#"{"message": "test"}"#;

        let result = validate_event("test-event", Some(payload), None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_event_invalid_json() {
        let config = create_test_config();
        let invalid_payload = r#"{"message": invalid}"#;

        let result = validate_event("test-event", Some(invalid_payload), None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_event_no_payload() {
        let config = create_test_config();

        let result = validate_event("test-event", None, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_batch_events_success() {
        let config = create_test_config();
        let channels1 = vec!["channel1".to_string()];
        let channels2 = vec!["channel2".to_string()];
        let payload = r#"{"data": "test"}"#;

        let events = vec![
            ("event1", channels1.as_slice(), Some(payload)),
            ("event2", channels2.as_slice(), Some(payload)),
        ];

        let result = validate_batch_events(&events, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_batch_events_too_many() {
        let config = create_test_config();
        let channels = vec!["channel".to_string()];
        let payload = r#"{"data": "test"}"#;

        // Create 11 events (exceeds default limit of 10)
        let events: Vec<_> = (0..11)
            .map(|i| {
                (
                    format!("event{}", i).leak() as &str,
                    channels.as_slice(),
                    Some(payload),
                )
            })
            .collect();

        let result = validate_batch_events(&events, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_batch_events_too_many_channels() {
        let config = create_test_config();
        // Create 101 channels (exceeds default limit of 100)
        let channels: Vec<String> = (0..101).map(|i| format!("channel{}", i)).collect();
        let payload = r#"{"data": "test"}"#;

        let events = vec![("event", channels.as_slice(), Some(payload))];

        let result = validate_batch_events(&events, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_subscription_public_channel() {
        let config = create_test_config();
        let result = validate_subscription("public-channel", None, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_subscription_private_channel_with_auth() {
        let config = create_test_config();
        let auth_data = serde_json::json!({
            "auth": "test_key:signature"
        });

        let result = validate_subscription("private-channel", Some(&auth_data), None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_subscription_presence_channel_with_data() {
        let config = create_test_config();
        let auth_data = serde_json::json!({
            "auth": "test_key:signature",
            "channel_data": r#"{"user_id": "123"}"#
        });

        let result = validate_subscription("presence-channel", Some(&auth_data), None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_subscription_presence_channel_missing_channel_data() {
        let config = create_test_config();
        let auth_data = serde_json::json!({
            "auth": "test_key:signature"
        });

        let result = validate_subscription("presence-channel", Some(&auth_data), None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_presence_member_size_success() {
        let config = create_test_config();
        let member_data = r#"{"user_id": "123", "user_info": {"name": "Test"}}"#;

        let result = validate_presence_member_size(member_data, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_presence_member_size_exceeds_limit() {
        let config = create_test_config();
        // Create data larger than 2 KB (default limit)
        let large_data = format!(r#"{{"user_id": "123", "data": "{}"}}"#, "a".repeat(3000));

        let result = validate_presence_member_size(&large_data, None, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_presence_member_size_with_app_limit() {
        let config = create_test_config();
        let mut app = create_test_app();
        app.max_presence_member_size_in_kb = Some(0.5); // 0.5 KB limit

        let small_data = r#"{"user_id": "123"}"#;
        let result = validate_presence_member_size(small_data, Some(&app), &config);
        assert!(result.is_ok());

        let large_data = "a".repeat(600); // ~0.6 KB
        let result = validate_presence_member_size(&large_data, Some(&app), &config);
        assert!(result.is_err());
    }
}
