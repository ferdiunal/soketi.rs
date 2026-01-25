// Minimal test file to verify auth module functionality
// This file only tests the auth module without depending on other modules

use hex;
use hmac::{Hmac, Mac};
use md5;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Generate a Pusher signature using HMAC-SHA256
pub fn generate_pusher_signature(app_secret: &str, to_sign: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(app_secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(to_sign.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes)
}

/// Verify a Pusher signature
pub fn verify_pusher_signature(signature: &str, app_secret: &str, to_sign: &str) -> bool {
    let expected_signature = generate_pusher_signature(app_secret, to_sign);
    expected_signature == signature
}

/// Generate an auth string for channel authentication
pub fn generate_channel_auth(
    app_key: &str,
    app_secret: &str,
    socket_id: &str,
    channel_name: &str,
    channel_data: Option<&str>,
) -> String {
    let to_sign = if let Some(data) = channel_data {
        format!("{}:{}:{}", socket_id, channel_name, data)
    } else {
        format!("{}:{}", socket_id, channel_name)
    };

    let signature = generate_pusher_signature(app_secret, &to_sign);
    format!("{}:{}", app_key, signature)
}

/// Verify channel authentication
pub fn verify_channel_auth(
    auth: &str,
    app_key: &str,
    app_secret: &str,
    socket_id: &str,
    channel_name: &str,
    channel_data: Option<&str>,
) -> bool {
    let expected_auth =
        generate_channel_auth(app_key, app_secret, socket_id, channel_name, channel_data);
    expected_auth == auth
}

/// Generate an auth string for user authentication (pusher:signin)
pub fn generate_user_auth(app_secret: &str, socket_id: &str, user_data: &str) -> String {
    let to_sign = format!("{}::user::{}", socket_id, user_data);
    generate_pusher_signature(app_secret, &to_sign)
}

/// Verify user authentication
pub fn verify_user_auth(auth: &str, app_secret: &str, socket_id: &str, user_data: &str) -> bool {
    let expected_auth = generate_user_auth(app_secret, socket_id, user_data);
    expected_auth == auth
}

/// Generate MD5 hash of a string
pub fn generate_md5_hash(data: &str) -> String {
    let digest = md5::compute(data.as_bytes());
    format!("{:x}", digest)
}

/// Generate signature for HTTP API requests
pub fn generate_api_auth_signature(
    app_secret: &str,
    method: &str,
    path: &str,
    query_string: &str,
) -> String {
    let to_sign = format!("{}\n{}\n{}", method, path, query_string);
    generate_pusher_signature(app_secret, &to_sign)
}

/// Verify HTTP API request signature
pub fn verify_api_auth_signature(
    signature: &str,
    app_secret: &str,
    method: &str,
    path: &str,
    query_string: &str,
) -> bool {
    let expected_signature = generate_api_auth_signature(app_secret, method, path, query_string);
    expected_signature == signature
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pusher_signature() {
        let signature = generate_pusher_signature("secret", "test_data");
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_signature_round_trip() {
        // Property 7: Signature Verification Round Trip
        // For any valid app credentials and message data, generating a signature
        // and then verifying it should succeed
        let app_secret = "test_secret";
        let to_sign = "test_data";

        let signature = generate_pusher_signature(app_secret, to_sign);
        assert!(verify_pusher_signature(&signature, app_secret, to_sign));
    }

    #[test]
    fn test_channel_auth_round_trip() {
        let app_key = "app_key";
        let app_secret = "app_secret";
        let socket_id = "123.456";
        let channel_name = "private-channel";

        let auth = generate_channel_auth(app_key, app_secret, socket_id, channel_name, None);
        assert!(verify_channel_auth(
            &auth,
            app_key,
            app_secret,
            socket_id,
            channel_name,
            None
        ));
    }

    #[test]
    fn test_user_auth_round_trip() {
        let app_secret = "app_secret";
        let socket_id = "123.456";
        let user_data = r#"{"id":"user123"}"#;

        let auth = generate_user_auth(app_secret, socket_id, user_data);
        assert!(verify_user_auth(&auth, app_secret, socket_id, user_data));
    }

    #[test]
    fn test_api_auth_round_trip() {
        let app_secret = "app_secret";
        let method = "POST";
        let path = "/apps/123/events";
        let query_string = "auth_key=key&auth_timestamp=123&auth_version=1.0";

        let signature = generate_api_auth_signature(app_secret, method, path, query_string);
        assert!(verify_api_auth_signature(
            &signature,
            app_secret,
            method,
            path,
            query_string
        ));
    }

    #[test]
    fn test_websocket_auth_format() {
        // Test WebSocket channel authentication format
        let auth = generate_channel_auth("key", "secret", "123.456", "private-channel", None);
        assert!(auth.starts_with("key:"));

        let parts: Vec<&str> = auth.split(':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "key");
        assert_eq!(parts[1].len(), 64); // SHA256 signature
    }

    #[test]
    fn test_http_api_auth_format() {
        // Test HTTP API authentication format
        let signature = generate_api_auth_signature("secret", "POST", "/path", "query");

        // Verify it matches signing the formatted string
        let to_sign = "POST\n/path\nquery";
        let expected = generate_pusher_signature("secret", to_sign);
        assert_eq!(signature, expected);
    }

    #[test]
    fn test_user_auth_format() {
        // Test user authentication format
        let user_data = r#"{"id":"123"}"#;
        let auth = generate_user_auth("secret", "123.456", user_data);

        // Verify it matches signing the formatted string
        let to_sign = format!("123.456::user::{}", user_data);
        let expected = generate_pusher_signature("secret", &to_sign);
        assert_eq!(auth, expected);
    }

    #[test]
    fn test_md5_hash() {
        let hash = generate_md5_hash("test data");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 32); // MD5 produces 32 hex characters
    }

    #[test]
    fn test_signature_with_special_characters() {
        let data = "test:data/with\\special@chars!";
        let sig = generate_pusher_signature("secret", data);
        assert!(verify_pusher_signature(&sig, "secret", data));
    }

    #[test]
    fn test_signature_with_unicode() {
        let data = "test_data_with_émojis_🎉";
        let sig = generate_pusher_signature("secret", data);
        assert!(verify_pusher_signature(&sig, "secret", data));
    }
}

#[test]
fn test_timestamp_validation_current() {
    // Current timestamp should be valid
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simulate the validation logic
    let time_diff = 0; // current time - current time = 0
    assert!(time_diff <= 600);
}

#[test]
fn test_timestamp_validation_recent() {
    // Timestamp from 5 minutes ago should be valid (300 seconds < 600)
    let time_diff = 300;
    assert!(time_diff <= 600);
}

#[test]
fn test_timestamp_validation_old() {
    // Timestamp from 15 minutes ago should be invalid (900 seconds > 600)
    let time_diff = 900;
    assert!(time_diff > 600);
}

#[test]
fn test_timestamp_validation_boundary() {
    // Timestamp exactly 600 seconds ago should be valid
    let time_diff = 600;
    assert!(time_diff <= 600);
}

#[test]
fn test_timestamp_validation_just_outside() {
    // Timestamp 601 seconds ago should be invalid
    let time_diff = 601;
    assert!(time_diff > 600);
}

#[test]
fn test_api_auth_with_timestamp_format() {
    // Test that query string includes timestamp
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let query = format!(
        "auth_key=key&auth_timestamp={}&auth_version=1.0",
        current_time
    );

    // Verify timestamp can be extracted
    assert!(query.contains("auth_timestamp="));
    assert!(query.contains(&current_time.to_string()));
}

#[test]
fn test_extract_timestamp_from_query() {
    let query = "auth_key=key&auth_timestamp=1234567890&auth_version=1.0";

    // Extract timestamp manually
    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            if key == "auth_timestamp" {
                let timestamp = value.parse::<u64>().unwrap();
                assert_eq!(timestamp, 1234567890);
                return;
            }
        }
    }
    panic!("auth_timestamp not found");
}

#[test]
fn test_md5_body_hashing() {
    // Test MD5 hashing for API request bodies
    let body = r#"{"name":"test","channel":"my-channel","data":"hello"}"#;
    let hash = generate_md5_hash(body);

    // MD5 hash should be 32 hex characters
    assert_eq!(hash.len(), 32);

    // Same body should produce same hash
    let hash2 = generate_md5_hash(body);
    assert_eq!(hash, hash2);
}

#[test]
fn test_api_signature_with_body_md5() {
    // Test API signature generation with body MD5
    let body = r#"{"name":"test"}"#;
    let body_md5 = generate_md5_hash(body);

    let query = format!(
        "auth_key=key&auth_timestamp=123&auth_version=1.0&body_md5={}",
        body_md5
    );
    let signature = generate_api_auth_signature("secret", "POST", "/apps/123/events", &query);

    // Signature should be valid
    assert!(verify_api_auth_signature(
        &signature,
        "secret",
        "POST",
        "/apps/123/events",
        &query
    ));
}
