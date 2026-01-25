use soketi_rs::auth::*;

#[test]
fn test_generate_pusher_signature() {
    // Test basic signature generation
    let signature = generate_pusher_signature("secret", "test_data");
    assert!(!signature.is_empty());
    assert_eq!(signature.len(), 64); // SHA256 produces 64 hex characters
}

#[test]
fn test_signature_consistency() {
    // Same input should always produce same signature
    let sig1 = generate_pusher_signature("secret", "test_data");
    let sig2 = generate_pusher_signature("secret", "test_data");
    assert_eq!(sig1, sig2);
}

#[test]
fn test_signature_varies_with_secret() {
    // Different secrets should produce different signatures
    let sig1 = generate_pusher_signature("secret1", "test_data");
    let sig2 = generate_pusher_signature("secret2", "test_data");
    assert_ne!(sig1, sig2);
}

#[test]
fn test_signature_varies_with_data() {
    // Different data should produce different signatures
    let sig1 = generate_pusher_signature("secret", "test_data1");
    let sig2 = generate_pusher_signature("secret", "test_data2");
    assert_ne!(sig1, sig2);
}

#[test]
fn test_verify_pusher_signature_valid() {
    let to_sign = "test_data";
    let signature = generate_pusher_signature("secret", to_sign);
    assert!(verify_pusher_signature(&signature, "secret", to_sign));
}

#[test]
fn test_verify_pusher_signature_invalid() {
    let signature = generate_pusher_signature("secret", "test_data");
    assert!(!verify_pusher_signature(
        &signature,
        "wrong_secret",
        "test_data"
    ));
    assert!(!verify_pusher_signature(&signature, "secret", "wrong_data"));
    assert!(!verify_pusher_signature(
        "invalid_signature",
        "secret",
        "test_data"
    ));
}

#[test]
fn test_generate_channel_auth_private() {
    // Test private channel auth generation
    let auth = generate_channel_auth("app_key", "app_secret", "123.456", "private-channel", None);

    // Should be in format: app_key:signature
    assert!(auth.starts_with("app_key:"));
    let parts: Vec<&str> = auth.split(':').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "app_key");
    assert_eq!(parts[1].len(), 64); // SHA256 hex signature
}

#[test]
fn test_generate_channel_auth_presence() {
    // Test presence channel auth generation with channel_data
    let channel_data = r#"{"user_id":"123","user_info":{"name":"John"}}"#;
    let auth = generate_channel_auth(
        "app_key",
        "app_secret",
        "123.456",
        "presence-channel",
        Some(channel_data),
    );

    assert!(auth.starts_with("app_key:"));
    let parts: Vec<&str> = auth.split(':').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "app_key");
}

#[test]
fn test_verify_channel_auth_valid() {
    let auth = generate_channel_auth("app_key", "app_secret", "123.456", "private-channel", None);

    assert!(verify_channel_auth(
        &auth,
        "app_key",
        "app_secret",
        "123.456",
        "private-channel",
        None,
    ));
}

#[test]
fn test_verify_channel_auth_invalid() {
    let auth = generate_channel_auth("app_key", "app_secret", "123.456", "private-channel", None);

    // Wrong app key
    assert!(!verify_channel_auth(
        &auth,
        "wrong_key",
        "app_secret",
        "123.456",
        "private-channel",
        None,
    ));

    // Wrong socket ID
    assert!(!verify_channel_auth(
        &auth,
        "app_key",
        "app_secret",
        "789.012",
        "private-channel",
        None,
    ));

    // Wrong channel name
    assert!(!verify_channel_auth(
        &auth,
        "app_key",
        "app_secret",
        "123.456",
        "different-channel",
        None,
    ));
}

#[test]
fn test_verify_channel_auth_presence_with_data() {
    let channel_data = r#"{"user_id":"123","user_info":{"name":"John"}}"#;
    let auth = generate_channel_auth(
        "app_key",
        "app_secret",
        "123.456",
        "presence-channel",
        Some(channel_data),
    );

    assert!(verify_channel_auth(
        &auth,
        "app_key",
        "app_secret",
        "123.456",
        "presence-channel",
        Some(channel_data),
    ));

    // Should fail without channel_data
    assert!(!verify_channel_auth(
        &auth,
        "app_key",
        "app_secret",
        "123.456",
        "presence-channel",
        None,
    ));
}

#[test]
fn test_generate_user_auth() {
    let user_data = r#"{"id":"user123","name":"John Doe"}"#;
    let auth = generate_user_auth("app_secret", "123.456", user_data);

    assert!(!auth.is_empty());
    assert_eq!(auth.len(), 64); // SHA256 hex signature
}

#[test]
fn test_verify_user_auth_valid() {
    let user_data = r#"{"id":"user123","name":"John Doe"}"#;
    let auth = generate_user_auth("app_secret", "123.456", user_data);

    assert!(verify_user_auth(&auth, "app_secret", "123.456", user_data));
}

#[test]
fn test_verify_user_auth_invalid() {
    let user_data = r#"{"id":"user123","name":"John Doe"}"#;
    let auth = generate_user_auth("app_secret", "123.456", user_data);

    // Wrong secret
    assert!(!verify_user_auth(
        &auth,
        "wrong_secret",
        "123.456",
        user_data
    ));

    // Wrong socket ID
    assert!(!verify_user_auth(&auth, "app_secret", "789.012", user_data));

    // Wrong user data
    let different_data = r#"{"id":"user456","name":"Jane Doe"}"#;
    assert!(!verify_user_auth(
        &auth,
        "app_secret",
        "123.456",
        different_data
    ));
}

#[test]
fn test_generate_md5_hash() {
    let hash = generate_md5_hash("test data");
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 32); // MD5 produces 32 hex characters
}

#[test]
fn test_md5_hash_consistency() {
    let hash1 = generate_md5_hash("test data");
    let hash2 = generate_md5_hash("test data");
    assert_eq!(hash1, hash2);
}

#[test]
fn test_md5_hash_varies_with_data() {
    let hash1 = generate_md5_hash("test data 1");
    let hash2 = generate_md5_hash("test data 2");
    assert_ne!(hash1, hash2);
}

#[test]
fn test_generate_api_auth_signature() {
    let signature = generate_api_auth_signature(
        "app_secret",
        "POST",
        "/apps/123/events",
        "auth_key=key&auth_timestamp=123&auth_version=1.0",
    );

    assert!(!signature.is_empty());
    assert_eq!(signature.len(), 64); // SHA256 hex signature
}

#[test]
fn test_verify_api_auth_signature_valid() {
    let signature = generate_api_auth_signature(
        "app_secret",
        "POST",
        "/apps/123/events",
        "auth_key=key&auth_timestamp=123&auth_version=1.0",
    );

    assert!(verify_api_auth_signature(
        &signature,
        "app_secret",
        "POST",
        "/apps/123/events",
        "auth_key=key&auth_timestamp=123&auth_version=1.0",
    ));
}

#[test]
fn test_verify_api_auth_signature_invalid() {
    let signature = generate_api_auth_signature(
        "app_secret",
        "POST",
        "/apps/123/events",
        "auth_key=key&auth_timestamp=123&auth_version=1.0",
    );

    // Wrong method
    assert!(!verify_api_auth_signature(
        &signature,
        "app_secret",
        "GET",
        "/apps/123/events",
        "auth_key=key&auth_timestamp=123&auth_version=1.0",
    ));

    // Wrong path
    assert!(!verify_api_auth_signature(
        &signature,
        "app_secret",
        "POST",
        "/apps/456/events",
        "auth_key=key&auth_timestamp=123&auth_version=1.0",
    ));

    // Wrong query string
    assert!(!verify_api_auth_signature(
        &signature,
        "app_secret",
        "POST",
        "/apps/123/events",
        "auth_key=key&auth_timestamp=456&auth_version=1.0",
    ));
}

#[test]
fn test_api_signature_format() {
    // The signature should be generated from METHOD\nPATH\nQUERY_STRING
    let signature = generate_api_auth_signature("secret", "POST", "/path", "query");

    // Verify it's the same as signing the formatted string directly
    let to_sign = "POST\n/path\nquery";
    let expected = generate_pusher_signature("secret", to_sign);
    assert_eq!(signature, expected);
}

#[test]
fn test_channel_auth_format_private() {
    // For private channels: socket_id:channel_name
    let auth = generate_channel_auth("key", "secret", "123.456", "private-channel", None);

    let to_sign = "123.456:private-channel";
    let expected_sig = generate_pusher_signature("secret", to_sign);
    let expected_auth = format!("key:{}", expected_sig);
    assert_eq!(auth, expected_auth);
}

#[test]
fn test_channel_auth_format_presence() {
    // For presence channels: socket_id:channel_name:channel_data
    let channel_data = r#"{"user_id":"123"}"#;
    let auth = generate_channel_auth(
        "key",
        "secret",
        "123.456",
        "presence-channel",
        Some(channel_data),
    );

    let to_sign = format!("123.456:presence-channel:{}", channel_data);
    let expected_sig = generate_pusher_signature("secret", &to_sign);
    let expected_auth = format!("key:{}", expected_sig);
    assert_eq!(auth, expected_auth);
}

#[test]
fn test_user_auth_format() {
    // For user auth: socket_id::user::user_data
    let user_data = r#"{"id":"123"}"#;
    let auth = generate_user_auth("secret", "123.456", user_data);

    let to_sign = format!("123.456::user::{}", user_data);
    let expected = generate_pusher_signature("secret", &to_sign);
    assert_eq!(auth, expected);
}

#[test]
fn test_empty_strings() {
    // Test with empty strings (edge case)
    let sig = generate_pusher_signature("", "");
    assert!(!sig.is_empty());
    assert_eq!(sig.len(), 64);
}

#[test]
fn test_special_characters() {
    // Test with special characters
    let data = "test:data/with\\special@chars!";
    let sig = generate_pusher_signature("secret", data);
    assert!(verify_pusher_signature(&sig, "secret", data));
}

#[test]
fn test_unicode_characters() {
    // Test with unicode characters
    let data = "test_data_with_émojis_🎉";
    let sig = generate_pusher_signature("secret", data);
    assert!(verify_pusher_signature(&sig, "secret", data));
}

#[test]
fn test_long_strings() {
    // Test with long strings
    let long_data = "a".repeat(10000);
    let sig = generate_pusher_signature("secret", &long_data);
    assert!(verify_pusher_signature(&sig, "secret", &long_data));
}
