use hex;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use soketi_rs::app::{App, Webhook};
use soketi_rs::webhook_sender::WebhookSender;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// Helper function to create a test app with webhooks
fn create_test_app_with_webhooks(webhooks: Vec<Webhook>) -> App {
    App {
        id: "test-app-id".to_string(),
        key: "test-app-key".to_string(),
        secret: "test-app-secret".to_string(),
        max_connections: None,
        enable_client_messages: true,
        enabled: true,
        max_backend_events_per_second: None,
        max_client_events_per_second: None,
        max_read_requests_per_second: None,
        webhooks,
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

/// Helper function to compute HMAC-SHA256 signature
fn compute_hmac_signature(data: &str, secret: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tokio::test]
async fn test_webhook_signature_is_valid_hmac() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);
    let app_secret = app.secret.clone();

    // Set up mock with custom matcher to verify signature
    Mock::given(method("POST"))
        .and(path("/"))
        .and(move |req: &Request| {
            // Get the signature from headers
            let signature_header = req.headers.get("X-Pusher-Signature");
            if signature_header.is_none() {
                return false;
            }

            let signature = signature_header.unwrap().to_str().unwrap();

            // Get the body
            let body = std::str::from_utf8(&req.body).unwrap();

            // Compute expected signature
            let expected_signature = compute_hmac_signature(body, &app_secret);

            // Verify signature matches
            signature == expected_signature
        })
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    sender.send_channel_occupied(&app, "test-channel").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_signature_changes_with_payload() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);
    let app_secret = app.secret.clone();

    // Set up mock to capture signatures
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(move |req: &Request| {
            let signature = req
                .headers
                .get("X-Pusher-Signature")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let body = std::str::from_utf8(&req.body).unwrap();
            let expected_signature = compute_hmac_signature(body, &app_secret);

            // Verify signature is correct
            assert_eq!(
                signature, expected_signature,
                "Signature mismatch for payload: {}",
                body
            );

            ResponseTemplate::new(200)
        })
        .expect(2)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send two different webhooks
    sender.send_channel_occupied(&app, "test-channel-1").await;
    sender.send_channel_vacated(&app, "test-channel-2").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_signature_with_different_secrets() {
    // Test that different app secrets produce different signatures

    let app1 = create_test_app_with_webhooks(vec![]);
    let mut app2 = app1.clone();
    app2.secret = "different-secret".to_string();

    let payload =
        r#"{"time_ms":1234567890,"events":[{"name":"channel_occupied","channel":"test"}]}"#;

    let sig1 = compute_hmac_signature(payload, &app1.secret);
    let sig2 = compute_hmac_signature(payload, &app2.secret);

    assert_ne!(
        sig1, sig2,
        "Different secrets should produce different signatures"
    );
}

#[tokio::test]
async fn test_webhook_signature_format() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);

    // Set up mock to verify signature format
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(move |req: &Request| {
            let signature = req
                .headers
                .get("X-Pusher-Signature")
                .unwrap()
                .to_str()
                .unwrap();

            // Verify signature is a valid hex string
            assert!(
                signature.len() == 64,
                "HMAC-SHA256 signature should be 64 hex characters"
            );
            assert!(
                signature.chars().all(|c| c.is_ascii_hexdigit()),
                "Signature should only contain hex characters"
            );

            ResponseTemplate::new(200)
        })
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    sender.send_channel_occupied(&app, "test-channel").await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_signature_with_special_characters() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);
    let app_secret = app.secret.clone();

    // Set up mock with custom matcher to verify signature
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(move |req: &Request| {
            let signature = req
                .headers
                .get("X-Pusher-Signature")
                .unwrap()
                .to_str()
                .unwrap();

            let body = std::str::from_utf8(&req.body).unwrap();
            let expected_signature = compute_hmac_signature(body, &app_secret);

            assert_eq!(
                signature, expected_signature,
                "Signature should be valid even with special characters in channel name"
            );

            ResponseTemplate::new(200)
        })
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send webhook with special characters in channel name
    sender
        .send_channel_occupied(&app, "test-channel-with-émojis-🎉")
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_signature_with_client_event_data() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create webhook
    let webhook = Webhook {
        url: Some(mock_server.uri()),
        lambda_function: None,
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: None,
    };

    let app = create_test_app_with_webhooks(vec![webhook]);
    let app_secret = app.secret.clone();

    // Set up mock with custom matcher to verify signature
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(move |req: &Request| {
            let signature = req
                .headers
                .get("X-Pusher-Signature")
                .unwrap()
                .to_str()
                .unwrap();

            let body = std::str::from_utf8(&req.body).unwrap();
            let expected_signature = compute_hmac_signature(body, &app_secret);

            assert_eq!(
                signature, expected_signature,
                "Signature should be valid for client events with complex data"
            );

            ResponseTemplate::new(200)
        })
        .expect(1)
        .mount(&mock_server)
        .await;

    let sender = WebhookSender::new();

    // Send client event with complex data
    let event_data = json!({
        "message": "Hello, world!",
        "user": {
            "id": 123,
            "name": "Test User"
        },
        "metadata": {
            "timestamp": 1234567890,
            "tags": ["tag1", "tag2"]
        }
    });

    sender
        .send_client_event(
            &app,
            "test-channel",
            "client-message",
            event_data,
            Some("socket-123"),
            Some("user-456"),
        )
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_webhook_signature_consistency() {
    // Test that the same payload always produces the same signature

    let app = create_test_app_with_webhooks(vec![]);
    let payload =
        r#"{"time_ms":1234567890,"events":[{"name":"channel_occupied","channel":"test"}]}"#;

    let sig1 = compute_hmac_signature(payload, &app.secret);
    let sig2 = compute_hmac_signature(payload, &app.secret);
    let sig3 = compute_hmac_signature(payload, &app.secret);

    assert_eq!(sig1, sig2, "Same payload should produce same signature");
    assert_eq!(sig2, sig3, "Same payload should produce same signature");
}

#[test]
fn test_hmac_signature_unit() {
    // Unit test for HMAC signature generation
    let secret = "my-secret-key";
    let data = "test data to sign";

    let signature = compute_hmac_signature(data, secret);

    // Verify signature is 64 hex characters (SHA256 produces 32 bytes = 64 hex chars)
    assert_eq!(signature.len(), 64);
    assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify signature is deterministic
    let signature2 = compute_hmac_signature(data, secret);
    assert_eq!(signature, signature2);

    // Verify different data produces different signature
    let signature3 = compute_hmac_signature("different data", secret);
    assert_ne!(signature, signature3);

    // Verify different secret produces different signature
    let signature4 = compute_hmac_signature(data, "different-secret");
    assert_ne!(signature, signature4);
}

#[test]
fn test_hmac_signature_empty_data() {
    // Test HMAC with empty data
    let secret = "my-secret-key";
    let data = "";

    let signature = compute_hmac_signature(data, secret);

    // Should still produce a valid signature
    assert_eq!(signature.len(), 64);
    assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hmac_signature_empty_secret() {
    // Test HMAC with empty secret (should still work)
    let secret = "";
    let data = "test data";

    let signature = compute_hmac_signature(data, secret);

    // Should still produce a valid signature
    assert_eq!(signature.len(), 64);
    assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hmac_signature_unicode() {
    // Test HMAC with unicode characters
    let secret = "my-secret-🔑";
    let data = "test data with émojis 🎉 and special chars: ñ, ü, ö";

    let signature = compute_hmac_signature(data, secret);

    // Should produce a valid signature
    assert_eq!(signature.len(), 64);
    assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));

    // Should be deterministic
    let signature2 = compute_hmac_signature(data, secret);
    assert_eq!(signature, signature2);
}

#[test]
fn test_hmac_signature_long_data() {
    // Test HMAC with very long data
    let secret = "my-secret-key";
    let data = "a".repeat(10000);

    let signature = compute_hmac_signature(&data, secret);

    // Should still produce a valid signature of the same length
    assert_eq!(signature.len(), 64);
    assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
}
