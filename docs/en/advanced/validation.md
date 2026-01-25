# Message Validation

This document describes the message validation functionality implemented in the Pusher server.

## Overview

The validation module provides comprehensive validation functions for all message types and data structures used in the Pusher protocol. It ensures that messages conform to configured limits and requirements before processing.

## Features

The validation module validates:

1. **Channel name length** - Ensures channel names don't exceed configured limits
2. **Event name length** - Ensures event names don't exceed configured limits
3. **Payload size** - Ensures message payloads don't exceed size limits (in KB)
4. **Batch size** - Ensures batch event requests don't exceed the maximum batch size
5. **Channel count** - Ensures events aren't sent to too many channels at once
6. **JSON structure** - Validates that payloads are valid JSON
7. **Required fields** - Validates that required fields are present in messages
8. **Presence member size** - Validates presence member data size

## Configuration

Validation limits can be configured at two levels:

### Global Limits (ServerConfig)

Global limits apply to all apps unless overridden by app-specific limits:

```rust
pub struct ServerConfig {
    pub channel_limits: ChannelLimits,
    pub event_limits: EventLimits,
    pub presence: PresenceLimits,
    // ... other fields
}

pub struct ChannelLimits {
    pub max_name_length: u64,        // Default: 200
    pub cache_ttl_seconds: u64,
}

pub struct EventLimits {
    pub max_channels_at_once: u64,   // Default: 100
    pub max_name_length: u64,        // Default: 200
    pub max_payload_in_kb: f64,      // Default: 100.0
    pub max_batch_size: u64,         // Default: 10
}

pub struct PresenceLimits {
    pub max_members_per_channel: u64,    // Default: 100
    pub max_member_size_in_kb: f64,      // Default: 2.0
}
```

### App-Specific Limits

Apps can override global limits with their own values:

```rust
pub struct App {
    pub max_channel_name_length: Option<u64>,
    pub max_event_name_length: Option<u64>,
    pub max_event_payload_in_kb: Option<f64>,
    pub max_event_batch_size: Option<u64>,
    pub max_event_channels_at_once: Option<u64>,
    pub max_presence_member_size_in_kb: Option<f64>,
    // ... other fields
}
```

When an app-specific limit is set, it takes precedence over the global limit.

## API Reference

### validate_channel_name_length

Validates that a channel name doesn't exceed the configured maximum length.

```rust
pub fn validate_channel_name_length(
    channel_name: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Parameters:**
- `channel_name` - The channel name to validate
- `app` - Optional app with per-app limits
- `config` - Server configuration with global limits

**Returns:**
- `Ok(())` if validation passes
- `Err(PusherError::ValidationError)` if channel name exceeds limits

**Example:**
```rust
let config = ServerConfig::default();
validate_channel_name_length("my-channel", None, &config)?;
```

### validate_event_name_length

Validates that an event name doesn't exceed the configured maximum length.

```rust
pub fn validate_event_name_length(
    event_name: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
validate_event_name_length("user-login", Some(&app), &config)?;
```

### validate_payload_size

Validates that a payload doesn't exceed the configured maximum size in kilobytes.

```rust
pub fn validate_payload_size(
    payload: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
let payload = r#"{"message": "Hello, World!"}"#;
validate_payload_size(payload, Some(&app), &config)?;
```

### validate_batch_size

Validates that a batch doesn't exceed the configured maximum number of events.

```rust
pub fn validate_batch_size(
    batch_size: usize,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
validate_batch_size(5, Some(&app), &config)?;
```

### validate_channel_count

Validates that an event isn't being sent to too many channels at once.

```rust
pub fn validate_channel_count(
    channel_count: usize,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
validate_channel_count(10, Some(&app), &config)?;
```

### validate_json_structure

Validates that a string is valid JSON and returns the parsed value.

```rust
pub fn validate_json_structure(json_str: &str) -> Result<Value>
```

**Example:**
```rust
let json = r#"{"key": "value"}"#;
let value = validate_json_structure(json)?;
```

### validate_required_fields

Validates that all required fields are present in a JSON object.

```rust
pub fn validate_required_fields(value: &Value, required_fields: &[&str]) -> Result<()>
```

**Example:**
```rust
let data = serde_json::json!({
    "event": "test",
    "channel": "my-channel"
});
validate_required_fields(&data, &["event", "channel"])?;
```

### validate_event

Convenience function that validates a complete event (name, payload size, and JSON structure).

```rust
pub fn validate_event(
    event_name: &str,
    payload: Option<&str>,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
let payload = r#"{"user_id": "123"}"#;
validate_event("user-registered", Some(payload), Some(&app), &config)?;
```

### validate_batch_events

Validates a batch of events including batch size, individual events, and channel counts.

```rust
pub fn validate_batch_events(
    events: &[(&str, &[String], Option<&str>)],
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Parameters:**
- `events` - Slice of tuples containing (event_name, channels, payload)

**Example:**
```rust
let channels1 = vec!["channel-1".to_string()];
let channels2 = vec!["channel-2".to_string()];
let payload = r#"{"message": "test"}"#;

let events = vec![
    ("event-1", channels1.as_slice(), Some(payload)),
    ("event-2", channels2.as_slice(), Some(payload)),
];

validate_batch_events(&events, Some(&app), &config)?;
```

### validate_subscription

Validates a subscription request including channel name and required authentication fields.

```rust
pub fn validate_subscription(
    channel_name: &str,
    auth_data: Option<&Value>,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
// Public channel
validate_subscription("public-channel", None, Some(&app), &config)?;

// Private channel
let auth = serde_json::json!({"auth": "key:signature"});
validate_subscription("private-channel", Some(&auth), Some(&app), &config)?;

// Presence channel
let presence_auth = serde_json::json!({
    "auth": "key:signature",
    "channel_data": r#"{"user_id": "123"}"#
});
validate_subscription("presence-channel", Some(&presence_auth), Some(&app), &config)?;
```

### validate_presence_member_size

Validates that presence member data doesn't exceed the configured size limit.

```rust
pub fn validate_presence_member_size(
    member_data: &str,
    app: Option<&App>,
    config: &ServerConfig,
) -> Result<()>
```

**Example:**
```rust
let member_data = r#"{"user_id": "123", "user_info": {"name": "Test"}}"#;
validate_presence_member_size(member_data, Some(&app), &config)?;
```

## Error Handling

All validation functions return `Result<T>` where the error type is `PusherError::ValidationError`.

Validation errors include descriptive messages indicating:
- What was validated
- What the limit was
- What the actual value was

Example error messages:
- `"Channel name exceeds maximum length of 200 characters"`
- `"Payload size 150.00 KB exceeds maximum of 100.00 KB"`
- `"Batch size 15 exceeds maximum of 10"`
- `"Invalid JSON structure: unexpected token at line 1 column 5"`
- `"Missing required field: channel_data"`

## Usage in WebSocket Handler

The validation functions are used throughout the WebSocket handler to validate incoming messages:

```rust
// Validate subscription
validate_subscription(&channel_name, Some(&auth_data), Some(&app), &config)?;

// Validate client event
validate_event(&event_name, Some(&payload), Some(&app), &config)?;
validate_channel_name_length(&channel, Some(&app), &config)?;
```

## Usage in HTTP API Handler

The validation functions are used in HTTP API endpoints:

```rust
// Validate single event
validate_event(&event_name, Some(&data), Some(&app), &config)?;
validate_channel_count(channels.len(), Some(&app), &config)?;

// Validate batch events
let events: Vec<_> = batch.events.iter()
    .map(|e| (e.name.as_str(), e.channels.as_slice(), Some(e.data.as_str())))
    .collect();
validate_batch_events(&events, Some(&app), &config)?;
```

## Testing

The validation module includes comprehensive unit tests covering:

- Valid inputs that should pass validation
- Invalid inputs that should fail validation
- Edge cases (empty strings, boundary values)
- App-specific limits overriding global limits
- All error conditions

Run the tests with:
```bash
cargo test validation
```

Run the example with:
```bash
cargo run --example validation_example
```

## Requirements Mapping

This implementation satisfies the following requirements:

- **Requirement 19.1**: Event name length validation
- **Requirement 19.2**: Channel name length validation
- **Requirement 19.3**: Payload size validation
- **Requirement 19.4**: Batch size validation
- **Requirement 19.5**: Channel count validation
- **Requirement 19.6**: JSON structure validation
- **Requirement 19.7**: Required fields validation

## Performance Considerations

- All validation functions are designed to be fast and non-blocking
- String length checks are O(1) in Rust (strings store their length)
- JSON parsing is only performed when necessary
- No allocations are made unless validation fails (for error messages)

## Future Enhancements

Potential future enhancements:

1. **Async validation** - For validations that might need to query external services
2. **Custom validators** - Allow apps to register custom validation functions
3. **Validation caching** - Cache validation results for repeated checks
4. **Detailed metrics** - Track validation failures by type and reason
5. **Rate limiting** - Integrate with rate limiter to throttle validation failures
