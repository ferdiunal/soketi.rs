/// Example demonstrating the message validation functions
///
/// This example shows how to use the validation module to validate:
/// - Channel names
/// - Event names
/// - Payload sizes
/// - Batch sizes
/// - Channel counts
/// - JSON structure
/// - Required fields
/// - Complete events
/// - Batch events
/// - Subscriptions
/// - Presence member data
use soketi_rs::app::App;
use soketi_rs::config::ServerConfig;
use soketi_rs::validation::*;

fn main() {
    println!("=== Message Validation Examples ===\n");

    // Create a default server configuration
    let config = ServerConfig::default();

    // Create a test app with custom limits
    let mut app = App::new(
        "test_app".to_string(),
        "test_key".to_string(),
        "test_secret".to_string(),
    );
    app.max_channel_name_length = Some(50);
    app.max_event_name_length = Some(100);
    app.max_event_payload_in_kb = Some(10.0);
    app.max_event_batch_size = Some(5);
    app.max_event_channels_at_once = Some(10);

    // Example 1: Validate channel name length
    println!("1. Channel Name Validation:");
    let valid_channel = "my-channel";
    match validate_channel_name_length(valid_channel, Some(&app), &config) {
        Ok(_) => println!("   ✓ Channel name '{}' is valid", valid_channel),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    let long_channel = "a".repeat(60);
    match validate_channel_name_length(&long_channel, Some(&app), &config) {
        Ok(_) => println!("   ✓ Long channel name is valid"),
        Err(e) => println!("   ✗ Long channel name rejected: {}", e),
    }
    println!();

    // Example 2: Validate event name length
    println!("2. Event Name Validation:");
    let valid_event = "user-login";
    match validate_event_name_length(valid_event, Some(&app), &config) {
        Ok(_) => println!("   ✓ Event name '{}' is valid", valid_event),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Example 3: Validate payload size
    println!("3. Payload Size Validation:");
    let small_payload = r#"{"message": "Hello, World!"}"#;
    match validate_payload_size(small_payload, Some(&app), &config) {
        Ok(_) => println!(
            "   ✓ Small payload is valid ({} bytes)",
            small_payload.len()
        ),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    let large_payload = "x".repeat(15 * 1024); // 15 KB
    match validate_payload_size(&large_payload, Some(&app), &config) {
        Ok(_) => println!("   ✓ Large payload is valid"),
        Err(e) => println!("   ✗ Large payload rejected: {}", e),
    }
    println!();

    // Example 4: Validate batch size
    println!("4. Batch Size Validation:");
    match validate_batch_size(3, Some(&app), &config) {
        Ok(_) => println!("   ✓ Batch of 3 events is valid"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    match validate_batch_size(10, Some(&app), &config) {
        Ok(_) => println!("   ✓ Batch of 10 events is valid"),
        Err(e) => println!("   ✗ Batch of 10 events rejected: {}", e),
    }
    println!();

    // Example 5: Validate channel count
    println!("5. Channel Count Validation:");
    match validate_channel_count(5, Some(&app), &config) {
        Ok(_) => println!("   ✓ 5 channels is valid"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    match validate_channel_count(15, Some(&app), &config) {
        Ok(_) => println!("   ✓ 15 channels is valid"),
        Err(e) => println!("   ✗ 15 channels rejected: {}", e),
    }
    println!();

    // Example 6: Validate JSON structure
    println!("6. JSON Structure Validation:");
    let valid_json = r#"{"name": "test", "value": 42}"#;
    match validate_json_structure(valid_json) {
        Ok(value) => println!("   ✓ Valid JSON parsed: {:?}", value),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    let invalid_json = r#"{"name": "test", invalid}"#;
    match validate_json_structure(invalid_json) {
        Ok(_) => println!("   ✓ JSON is valid"),
        Err(e) => println!("   ✗ Invalid JSON rejected: {}", e),
    }
    println!();

    // Example 7: Validate required fields
    println!("7. Required Fields Validation:");
    let json_with_fields = serde_json::json!({
        "event": "test-event",
        "channel": "test-channel",
        "data": "test-data"
    });
    match validate_required_fields(&json_with_fields, &["event", "channel"]) {
        Ok(_) => println!("   ✓ All required fields present"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    let json_missing_field = serde_json::json!({
        "event": "test-event"
    });
    match validate_required_fields(&json_missing_field, &["event", "channel"]) {
        Ok(_) => println!("   ✓ All required fields present"),
        Err(e) => println!("   ✗ Missing field rejected: {}", e),
    }
    println!();

    // Example 8: Validate complete event
    println!("8. Complete Event Validation:");
    let event_name = "user-registered";
    let event_payload = r#"{"user_id": "123", "email": "user@example.com"}"#;
    match validate_event(event_name, Some(event_payload), Some(&app), &config) {
        Ok(_) => println!("   ✓ Event '{}' is valid", event_name),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Example 9: Validate batch events
    println!("9. Batch Events Validation:");
    let channels1 = vec!["channel-1".to_string(), "channel-2".to_string()];
    let channels2 = vec!["channel-3".to_string()];
    let payload = r#"{"message": "test"}"#;

    let events = vec![
        ("event-1", channels1.as_slice(), Some(payload)),
        ("event-2", channels2.as_slice(), Some(payload)),
    ];

    match validate_batch_events(&events, Some(&app), &config) {
        Ok(_) => println!("   ✓ Batch of {} events is valid", events.len()),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Example 10: Validate subscription
    println!("10. Subscription Validation:");

    // Public channel
    match validate_subscription("public-channel", None, Some(&app), &config) {
        Ok(_) => println!("   ✓ Public channel subscription is valid"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Private channel with auth
    let private_auth = serde_json::json!({
        "auth": "test_key:signature"
    });
    match validate_subscription("private-channel", Some(&private_auth), Some(&app), &config) {
        Ok(_) => println!("   ✓ Private channel subscription is valid"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Presence channel with auth and channel_data
    let presence_auth = serde_json::json!({
        "auth": "test_key:signature",
        "channel_data": r#"{"user_id": "123", "user_info": {"name": "Test User"}}"#
    });
    match validate_subscription(
        "presence-channel",
        Some(&presence_auth),
        Some(&app),
        &config,
    ) {
        Ok(_) => println!("   ✓ Presence channel subscription is valid"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Presence channel missing channel_data
    let incomplete_presence_auth = serde_json::json!({
        "auth": "test_key:signature"
    });
    match validate_subscription(
        "presence-channel",
        Some(&incomplete_presence_auth),
        Some(&app),
        &config,
    ) {
        Ok(_) => println!("   ✓ Incomplete presence subscription is valid"),
        Err(e) => println!("   ✗ Incomplete presence subscription rejected: {}", e),
    }
    println!();

    // Example 11: Validate presence member size
    println!("11. Presence Member Size Validation:");
    let member_data = r#"{"user_id": "123", "user_info": {"name": "Test User", "avatar": "url"}}"#;
    match validate_presence_member_size(member_data, Some(&app), &config) {
        Ok(_) => println!(
            "   ✓ Presence member data is valid ({} bytes)",
            member_data.len()
        ),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    println!("=== All validation examples completed ===");
}
