/// Test suite for error handling and error code mapping
/// Validates Requirements 13.1-13.9
use soketi_rs::error::PusherError;

#[test]
fn test_requirement_13_1_invalid_app_key_error_code() {
    // Requirement 13.1: WHEN an app key is invalid, THE Pusher_Server SHALL return error code 4001
    let error = PusherError::AppNotFound("invalid_key".to_string());
    assert_eq!(
        error.to_pusher_code(),
        4001,
        "Invalid app key should return error code 4001"
    );
}

#[test]
fn test_requirement_13_2_app_disabled_error_code() {
    // Requirement 13.2: WHEN an app is disabled, THE Pusher_Server SHALL return error code 4003
    let error = PusherError::AppDisabled("disabled_app".to_string());
    assert_eq!(
        error.to_pusher_code(),
        4003,
        "Disabled app should return error code 4003"
    );
}

#[test]
fn test_requirement_13_3_connection_limit_error_code() {
    // Requirement 13.3: WHEN connection limit is reached, THE Pusher_Server SHALL return error code 4100
    let error = PusherError::ConnectionLimitReached;
    assert_eq!(
        error.to_pusher_code(),
        4100,
        "Connection limit reached should return error code 4100"
    );
}

#[test]
fn test_requirement_13_4_authentication_failed_error_code() {
    // Requirement 13.4: WHEN authentication fails, THE Pusher_Server SHALL return error code 4009
    let error = PusherError::AuthenticationFailed("invalid signature".to_string());
    assert_eq!(
        error.to_pusher_code(),
        4009,
        "Authentication failure should return error code 4009"
    );
}

#[test]
fn test_requirement_13_5_server_closing_error_code() {
    // Requirement 13.5: WHEN the server is closing, THE Pusher_Server SHALL return error code 4200
    let error = PusherError::ServerClosing;
    assert_eq!(
        error.to_pusher_code(),
        4200,
        "Server closing should return error code 4200"
    );
}

#[test]
fn test_requirement_13_6_connection_timeout_error_code() {
    // Requirement 13.6: WHEN a connection times out, THE Pusher_Server SHALL return error code 4201
    let error = PusherError::ConnectionTimeout;
    assert_eq!(
        error.to_pusher_code(),
        4201,
        "Connection timeout should return error code 4201"
    );
}

#[test]
fn test_requirement_13_7_client_messages_disabled_error_code() {
    // Requirement 13.7: WHEN client messages are disabled, THE Pusher_Server SHALL return error code 4301
    let error = PusherError::ClientMessagesDisabled;
    assert_eq!(
        error.to_pusher_code(),
        4301,
        "Client messages disabled should return error code 4301"
    );
}

#[test]
fn test_requirement_13_8_server_error_code() {
    // Requirement 13.8: WHEN a server error occurs, THE Pusher_Server SHALL return error code 4302
    let error = PusherError::ServerError("internal error".to_string());
    assert_eq!(
        error.to_pusher_code(),
        4302,
        "Server error should return error code 4302"
    );
}

#[test]
fn test_requirement_13_9_descriptive_error_messages() {
    // Requirement 13.9: THE Pusher_Server SHALL include descriptive error messages with all error codes

    // Test AppNotFound
    let error = PusherError::AppNotFound("test_app".to_string());
    let message = error.to_error_message(None);
    assert_eq!(message.event, "pusher:error");
    assert_eq!(message.data.code, 4001);
    assert!(
        message.data.message.contains("test_app"),
        "Error message should contain app identifier"
    );
    assert!(
        !message.data.message.is_empty(),
        "Error message should not be empty"
    );

    // Test AppDisabled
    let error = PusherError::AppDisabled("disabled_app".to_string());
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4003);
    assert!(
        message.data.message.contains("disabled_app"),
        "Error message should contain app identifier"
    );

    // Test ConnectionLimitReached
    let error = PusherError::ConnectionLimitReached;
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4100);
    assert!(
        !message.data.message.is_empty(),
        "Error message should not be empty"
    );

    // Test AuthenticationFailed
    let error = PusherError::AuthenticationFailed("invalid signature".to_string());
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4009);
    assert!(
        message.data.message.contains("invalid signature"),
        "Error message should contain failure reason"
    );

    // Test ServerClosing
    let error = PusherError::ServerClosing;
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4200);
    assert!(
        !message.data.message.is_empty(),
        "Error message should not be empty"
    );

    // Test ConnectionTimeout
    let error = PusherError::ConnectionTimeout;
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4201);
    assert!(
        !message.data.message.is_empty(),
        "Error message should not be empty"
    );

    // Test ClientMessagesDisabled
    let error = PusherError::ClientMessagesDisabled;
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4301);
    assert!(
        !message.data.message.is_empty(),
        "Error message should not be empty"
    );

    // Test ServerError
    let error = PusherError::ServerError("internal error".to_string());
    let message = error.to_error_message(None);
    assert_eq!(message.data.code, 4302);
    assert!(
        message.data.message.contains("internal error"),
        "Error message should contain error details"
    );
}

#[test]
fn test_error_message_with_channel() {
    // Test that error messages can include channel information
    let error = PusherError::AuthenticationFailed("invalid auth".to_string());
    let message = error.to_error_message(Some("private-test-channel"));

    assert_eq!(message.event, "pusher:error");
    assert_eq!(message.data.code, 4009);
    assert_eq!(message.channel, Some("private-test-channel".to_string()));
}

#[test]
fn test_error_message_without_channel() {
    // Test that error messages work without channel information
    let error = PusherError::ConnectionLimitReached;
    let message = error.to_error_message(None);

    assert_eq!(message.event, "pusher:error");
    assert_eq!(message.data.code, 4100);
    assert_eq!(message.channel, None);
}

#[test]
fn test_user_authentication_timeout_error_code() {
    // UserAuthenticationTimeout should also map to 4009 (authentication error)
    let error = PusherError::UserAuthenticationTimeout;
    assert_eq!(
        error.to_pusher_code(),
        4009,
        "User authentication timeout should return error code 4009"
    );
}

#[test]
fn test_generic_errors_map_to_4302() {
    // Test that generic/internal errors map to 4302
    let errors = vec![
        PusherError::AdapterError("adapter failed".to_string()),
        PusherError::DatabaseError("db connection failed".to_string()),
        PusherError::RedisError("redis timeout".to_string()),
        PusherError::IoError("io error".to_string()),
        PusherError::SerializationError("json parse error".to_string()),
        PusherError::ValidationError("invalid input".to_string()),
        PusherError::WebhookError("webhook failed".to_string()),
        PusherError::QueueError("queue error".to_string()),
        PusherError::InvalidMessage("malformed message".to_string()),
        PusherError::ChannelError("channel error".to_string()),
        PusherError::RateLimitExceeded,
    ];

    for error in errors {
        assert_eq!(
            error.to_pusher_code(),
            4302,
            "Generic error {:?} should map to 4302",
            error
        );
    }
}

#[test]
fn test_all_error_variants_have_codes() {
    // Ensure all error variants can be converted to error codes without panicking
    let errors = vec![
        PusherError::AppNotFound("test".to_string()),
        PusherError::AppDisabled("test".to_string()),
        PusherError::ConnectionLimitReached,
        PusherError::AuthenticationFailed("test".to_string()),
        PusherError::RateLimitExceeded,
        PusherError::InvalidMessage("test".to_string()),
        PusherError::ChannelError("test".to_string()),
        PusherError::ServerClosing,
        PusherError::ConnectionTimeout,
        PusherError::UserAuthenticationTimeout,
        PusherError::ClientMessagesDisabled,
        PusherError::ServerError("test".to_string()),
        PusherError::AdapterError("test".to_string()),
        PusherError::DatabaseError("test".to_string()),
        PusherError::RedisError("test".to_string()),
        PusherError::IoError("test".to_string()),
        PusherError::SerializationError("test".to_string()),
        PusherError::ValidationError("test".to_string()),
        PusherError::WebhookError("test".to_string()),
        PusherError::QueueError("test".to_string()),
    ];

    for error in errors {
        let code = error.to_pusher_code();
        assert!(
            code >= 4000 && code < 5000,
            "Error code {} should be in the 4xxx range",
            code
        );
    }
}

#[test]
fn test_all_error_variants_have_messages() {
    // Ensure all error variants can be converted to error messages without panicking
    let errors = vec![
        PusherError::AppNotFound("test".to_string()),
        PusherError::AppDisabled("test".to_string()),
        PusherError::ConnectionLimitReached,
        PusherError::AuthenticationFailed("test".to_string()),
        PusherError::RateLimitExceeded,
        PusherError::InvalidMessage("test".to_string()),
        PusherError::ChannelError("test".to_string()),
        PusherError::ServerClosing,
        PusherError::ConnectionTimeout,
        PusherError::UserAuthenticationTimeout,
        PusherError::ClientMessagesDisabled,
        PusherError::ServerError("test".to_string()),
        PusherError::AdapterError("test".to_string()),
        PusherError::DatabaseError("test".to_string()),
        PusherError::RedisError("test".to_string()),
        PusherError::IoError("test".to_string()),
        PusherError::SerializationError("test".to_string()),
        PusherError::ValidationError("test".to_string()),
        PusherError::WebhookError("test".to_string()),
        PusherError::QueueError("test".to_string()),
    ];

    for error in errors {
        let message = error.to_error_message(None);
        assert_eq!(message.event, "pusher:error");
        assert!(
            !message.data.message.is_empty(),
            "Error message should not be empty for {:?}",
            error
        );
        assert!(
            message.data.code >= 4000 && message.data.code < 5000,
            "Error code should be in the 4xxx range"
        );
    }
}

#[test]
fn test_error_message_serialization() {
    // Test that error messages can be serialized to JSON
    let error = PusherError::AppNotFound("test_app".to_string());
    let message = error.to_error_message(Some("test-channel"));

    let json = serde_json::to_string(&message).expect("Should serialize to JSON");
    assert!(json.contains("pusher:error"));
    assert!(json.contains("4001"));
    assert!(json.contains("test-channel"));
}
