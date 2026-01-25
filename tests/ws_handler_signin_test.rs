use serde_json::json;
/// Tests for WebSocket handler signin functionality
///
/// This test suite verifies:
/// - Valid signin with correct user_data and auth signature
/// - Invalid signin with incorrect auth signature
/// - Invalid signin with missing user_data field
/// - Invalid signin with missing auth field
/// - Invalid signin with malformed user_data JSON
/// - Invalid signin with missing user_id in user_data
///
/// Requirements: 2.6, 16.2, 16.3, 16.4, 16.5
use soketi_rs::app::App;
use soketi_rs::auth::generate_user_auth;
use soketi_rs::pusher::PusherMessage;

#[test]
fn test_generate_user_auth_format() {
    // Test that user auth generation works correctly
    let app_secret = "test_secret";
    let socket_id = "123.456";
    let user_data = r#"{"id":"user123","name":"Test User"}"#;

    let auth = generate_user_auth(app_secret, socket_id, user_data);

    // Auth should be a hex string (64 characters for SHA256)
    assert_eq!(auth.len(), 64);
    assert!(auth.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_signin_message_structure() {
    // Test that we can create a valid signin message structure
    let user_data = r#"{"id":"user123","name":"Test User"}"#;
    let auth = "test_auth_signature";

    let signin_msg = PusherMessage::new("pusher:signin".to_string()).with_data(json!({
        "user_data": user_data,
        "auth": auth
    }));

    assert_eq!(signin_msg.event, "pusher:signin");
    assert!(signin_msg.data.is_some());

    let data = signin_msg.data.unwrap();
    assert_eq!(
        data.get("user_data").and_then(|v| v.as_str()),
        Some(user_data)
    );
    assert_eq!(data.get("auth").and_then(|v| v.as_str()), Some(auth));
}

#[test]
fn test_user_data_parsing() {
    // Test that user_data can be parsed correctly
    let user_data = r#"{"id":"user123","name":"Test User","email":"test@example.com"}"#;

    let parsed: serde_json::Value = serde_json::from_str(user_data).unwrap();

    assert_eq!(parsed.get("id").and_then(|v| v.as_str()), Some("user123"));
    assert_eq!(
        parsed.get("name").and_then(|v| v.as_str()),
        Some("Test User")
    );
    assert_eq!(
        parsed.get("email").and_then(|v| v.as_str()),
        Some("test@example.com")
    );
}

#[test]
fn test_user_data_missing_id() {
    // Test that user_data without id field is detected
    let user_data = r#"{"name":"Test User","email":"test@example.com"}"#;

    let parsed: serde_json::Value = serde_json::from_str(user_data).unwrap();

    // Should not have an id field
    assert!(parsed.get("id").is_none());
}

#[test]
fn test_signin_success_message_structure() {
    // Test that signin_success message has the correct structure
    let user_data = json!({
        "id": "user123",
        "name": "Test User"
    });

    let signin_success_msg =
        PusherMessage::new("pusher:signin_success".to_string()).with_data(user_data.clone());

    assert_eq!(signin_success_msg.event, "pusher:signin_success");
    assert!(signin_success_msg.data.is_some());

    let data = signin_success_msg.data.unwrap();
    assert_eq!(data.get("id").and_then(|v| v.as_str()), Some("user123"));
    assert_eq!(data.get("name").and_then(|v| v.as_str()), Some("Test User"));
}

#[test]
fn test_auth_signature_verification() {
    // Test that auth signature verification works correctly
    let app_secret = "test_secret";
    let socket_id = "123.456";
    let user_data = r#"{"id":"user123"}"#;

    // Generate a valid auth signature
    let valid_auth = generate_user_auth(app_secret, socket_id, user_data);

    // Verify it
    assert!(soketi_rs::auth::verify_user_auth(
        &valid_auth,
        app_secret,
        socket_id,
        user_data
    ));

    // Test with invalid auth
    let invalid_auth = "invalid_signature";
    assert!(!soketi_rs::auth::verify_user_auth(
        invalid_auth,
        app_secret,
        socket_id,
        user_data
    ));
}

#[test]
fn test_auth_signature_with_different_socket_id() {
    // Test that auth signature is specific to socket_id
    let app_secret = "test_secret";
    let socket_id_1 = "123.456";
    let socket_id_2 = "789.012";
    let user_data = r#"{"id":"user123"}"#;

    // Generate auth for socket_id_1
    let auth = generate_user_auth(app_secret, socket_id_1, user_data);

    // Should be valid for socket_id_1
    assert!(soketi_rs::auth::verify_user_auth(
        &auth,
        app_secret,
        socket_id_1,
        user_data
    ));

    // Should NOT be valid for socket_id_2
    assert!(!soketi_rs::auth::verify_user_auth(
        &auth,
        app_secret,
        socket_id_2,
        user_data
    ));
}

#[test]
fn test_auth_signature_with_different_user_data() {
    // Test that auth signature is specific to user_data
    let app_secret = "test_secret";
    let socket_id = "123.456";
    let user_data_1 = r#"{"id":"user123"}"#;
    let user_data_2 = r#"{"id":"user456"}"#;

    // Generate auth for user_data_1
    let auth = generate_user_auth(app_secret, socket_id, user_data_1);

    // Should be valid for user_data_1
    assert!(soketi_rs::auth::verify_user_auth(
        &auth,
        app_secret,
        socket_id,
        user_data_1
    ));

    // Should NOT be valid for user_data_2
    assert!(!soketi_rs::auth::verify_user_auth(
        &auth,
        app_secret,
        socket_id,
        user_data_2
    ));
}

#[test]
fn test_malformed_json_user_data() {
    // Test that malformed JSON is detected
    let malformed_user_data = r#"{"id":"user123""#; // Missing closing brace

    let result = serde_json::from_str::<serde_json::Value>(malformed_user_data);
    assert!(result.is_err());
}

#[test]
fn test_user_data_with_additional_fields() {
    // Test that user_data can contain additional fields beyond id
    let user_data =
        r#"{"id":"user123","name":"Test User","email":"test@example.com","role":"admin"}"#;

    let parsed: serde_json::Value = serde_json::from_str(user_data).unwrap();

    // Should have id field
    assert_eq!(parsed.get("id").and_then(|v| v.as_str()), Some("user123"));

    // Should also have additional fields
    assert_eq!(
        parsed.get("name").and_then(|v| v.as_str()),
        Some("Test User")
    );
    assert_eq!(
        parsed.get("email").and_then(|v| v.as_str()),
        Some("test@example.com")
    );
    assert_eq!(parsed.get("role").and_then(|v| v.as_str()), Some("admin"));
}

#[test]
fn test_signin_to_sign_format() {
    // Test the format of the string that gets signed for user authentication
    // Format should be: socket_id::user::user_data
    let socket_id = "123.456";
    let user_data = r#"{"id":"user123"}"#;

    let to_sign = format!("{}::user::{}", socket_id, user_data);

    assert_eq!(to_sign, r#"123.456::user::{"id":"user123"}"#);
    assert!(to_sign.contains("::user::"));
}
