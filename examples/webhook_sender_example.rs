use serde_json::json;
use soketi_rs::app::{App, LambdaConfig, Webhook};
use soketi_rs::webhook_sender::{BatchingConfig, WebhookSender};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("=== WebhookSender Initialization Examples ===\n");

    // Example 1: Basic initialization
    println!("1. Basic initialization:");
    let sender1 = WebhookSender::new();
    println!("   ✓ Created WebhookSender with default configuration\n");

    // Example 2: With batching enabled
    println!("2. With batching enabled:");
    let sender2 = WebhookSender::new().with_batching(true, 100);
    println!("   ✓ Created WebhookSender with batching (100ms duration)\n");

    // Example 3: With custom HTTP client
    println!("3. With custom HTTP client:");
    let custom_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("MyApp/1.0")
        .build()
        .unwrap();
    let sender3 = WebhookSender::with_http_client(custom_client);
    println!("   ✓ Created WebhookSender with custom HTTP client\n");

    // Example 4: With full configuration (no Lambda)
    println!("4. With full configuration (no Lambda):");
    let batching_config = BatchingConfig {
        enabled: true,
        duration_ms: 200,
    };
    let sender4 = WebhookSender::with_config(batching_config, false).await;
    println!("   ✓ Created WebhookSender with custom batching config\n");

    // Example 5: With Lambda support
    println!("5. With Lambda support:");
    let batching_config = BatchingConfig {
        enabled: false,
        duration_ms: 50,
    };
    let sender5 = WebhookSender::with_config(batching_config, true).await;
    println!("   ✓ Created WebhookSender with Lambda client initialized\n");

    // Example 6: Builder pattern with multiple configurations
    println!("6. Builder pattern with multiple configurations:");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let sender6 = WebhookSender::with_http_client(client).with_batching(true, 150);
    println!("   ✓ Created WebhookSender with custom HTTP client and batching\n");

    // Example 7: App configurations with webhooks
    println!("7. App configurations with webhooks:");

    // App with HTTP webhook
    let mut app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );
    app1.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string(), "channel_occupied".to_string()],
        headers: Some({
            let mut headers = HashMap::new();
            headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
            headers
        }),
        filter: None,
        lambda: None,
    }];
    println!("   ✓ App with HTTP webhook configured");
    println!(
        "     - Has client_event webhooks: {}",
        app1.has_client_event_webhooks()
    );
    println!(
        "     - Has channel_occupied webhooks: {}",
        app1.has_channel_occupied_webhooks()
    );

    // App with Lambda webhook
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );
    app2.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-webhook-handler".to_string()),
        event_types: vec!["member_added".to_string(), "member_removed".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];
    println!("   ✓ App with Lambda webhook configured");
    println!(
        "     - Has member_added webhooks: {}",
        app2.has_member_added_webhooks()
    );
    println!(
        "     - Has member_removed webhooks: {}",
        app2.has_member_removed_webhooks()
    );

    // App with filtered webhook
    let mut app3 = App::new(
        "app3".to_string(),
        "key3".to_string(),
        "secret3".to_string(),
    );
    app3.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: Some(soketi_rs::app::WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: Some("-notifications".to_string()),
        }),
        lambda: None,
    }];
    println!("   ✓ App with filtered webhook configured");
    println!("     - Filter: channels starting with 'private-' and ending with '-notifications'\n");

    println!("\n=== Webhook Sending Methods Examples ===\n");

    // Example 8: Demonstrating all webhook sending methods
    println!("8. Demonstrating all webhook sending methods:");

    // Create an app with all webhook types configured
    let mut app_all = App::new(
        "app_all".to_string(),
        "key_all".to_string(),
        "secret_all".to_string(),
    );
    app_all.webhooks = vec![Webhook {
        url: Some("https://example.com/webhook".to_string()),
        lambda_function: None,
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
            "member_added".to_string(),
            "member_removed".to_string(),
            "client_event".to_string(),
            "cache_miss".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: None,
    }];

    println!("   a) send_channel_occupied:");
    sender1
        .send_channel_occupied(&app_all, "test-channel")
        .await;
    println!("      ✓ Webhook sent for channel becoming occupied\n");

    println!("   b) send_channel_vacated:");
    sender1.send_channel_vacated(&app_all, "test-channel").await;
    println!("      ✓ Webhook sent for channel becoming vacated\n");

    println!("   c) send_member_added:");
    sender1
        .send_member_added(&app_all, "presence-channel", "user123")
        .await;
    println!("      ✓ Webhook sent for member joining presence channel\n");

    println!("   d) send_member_removed:");
    sender1
        .send_member_removed(&app_all, "presence-channel", "user123")
        .await;
    println!("      ✓ Webhook sent for member leaving presence channel\n");

    println!("   e) send_client_event:");
    let event_data = json!({
        "message": "Hello, World!",
        "timestamp": 1234567890
    });
    sender1
        .send_client_event(
            &app_all,
            "test-channel",
            "client-message",
            event_data,
            Some("socket123"),
            Some("user456"),
        )
        .await;
    println!("      ✓ Webhook sent for client event\n");

    println!("   f) send_cache_missed:");
    sender1.send_cache_missed(&app_all, "cache-channel").await;
    println!("      ✓ Webhook sent for cache miss\n");

    // Example 9: Demonstrating webhook filtering
    println!("9. Demonstrating webhook filtering:");
    println!("   When an app has no webhooks configured, methods return immediately:");
    let app_no_webhooks = App::new(
        "app_no_webhooks".to_string(),
        "key_no_webhooks".to_string(),
        "secret_no_webhooks".to_string(),
    );
    sender1
        .send_channel_occupied(&app_no_webhooks, "test-channel")
        .await;
    println!("   ✓ No webhook sent (app has no webhooks configured)\n");

    println!("=== All examples completed successfully! ===");
}
