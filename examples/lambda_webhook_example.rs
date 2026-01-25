use serde_json::json;
use soketi_rs::app::{App, LambdaConfig, Webhook};
/// Example demonstrating AWS Lambda webhook invocation
///
/// This example shows how to configure and use Lambda functions as webhook endpoints
/// for Pusher events. Lambda webhooks support both async (Event) and sync (RequestResponse)
/// invocation types, as well as custom client context.
///
/// To run this example:
/// ```
/// cargo run --example lambda_webhook_example
/// ```
///
/// Note: This example requires AWS credentials to be configured in your environment.
/// You can set them using environment variables or AWS config files.
use soketi_rs::webhook_sender::{BatchingConfig, WebhookSender};

#[tokio::main]
async fn main() {
    println!("=== AWS Lambda Webhook Example ===\n");

    // Example 1: Basic Lambda webhook with async invocation
    println!("1. Basic Lambda webhook with async invocation:");
    let mut app1 = App::new(
        "app1".to_string(),
        "key1".to_string(),
        "secret1".to_string(),
    );

    app1.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-channel-events-function".to_string()),
        event_types: vec![
            "channel_occupied".to_string(),
            "channel_vacated".to_string(),
        ],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true, // Fire-and-forget invocation
        }),
    }];

    println!("   Lambda Function: my-channel-events-function");
    println!("   Invocation Type: Async (Event)");
    println!("   Event Types: channel_occupied, channel_vacated");
    println!();

    // Example 2: Lambda webhook with sync invocation
    println!("2. Lambda webhook with sync invocation:");
    let mut app2 = App::new(
        "app2".to_string(),
        "key2".to_string(),
        "secret2".to_string(),
    );

    app2.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-member-events-function".to_string()),
        event_types: vec!["member_added".to_string(), "member_removed".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: false, // Wait for response
        }),
    }];

    println!("   Lambda Function: my-member-events-function");
    println!("   Invocation Type: Sync (RequestResponse)");
    println!("   Event Types: member_added, member_removed");
    println!();

    // Example 3: Lambda webhook with client context
    println!("3. Lambda webhook with custom client context:");
    let mut app3 = App::new(
        "app3".to_string(),
        "key3".to_string(),
        "secret3".to_string(),
    );

    let client_context = json!({
        "custom": {
            "app_id": "app3",
            "environment": "production",
            "region": "us-east-1",
            "metadata": {
                "version": "1.0.0",
                "deployment": "blue-green"
            }
        }
    });

    app3.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-client-events-function".to_string()),
        event_types: vec!["client_event".to_string()],
        headers: None,
        filter: None,
        lambda: Some(LambdaConfig {
            client_context: Some(client_context.clone()),
            async_invocation: true,
        }),
    }];

    println!("   Lambda Function: my-client-events-function");
    println!("   Invocation Type: Async (Event)");
    println!("   Event Types: client_event");
    println!(
        "   Client Context: {}",
        serde_json::to_string_pretty(&client_context).unwrap()
    );
    println!();

    // Example 4: Lambda webhook with channel filtering
    println!("4. Lambda webhook with channel name filtering:");
    let mut app4 = App::new(
        "app4".to_string(),
        "key4".to_string(),
        "secret4".to_string(),
    );

    app4.webhooks = vec![Webhook {
        url: None,
        lambda_function: Some("my-private-channels-function".to_string()),
        event_types: vec!["channel_occupied".to_string()],
        headers: None,
        filter: Some(soketi_rs::app::WebhookFilter {
            channel_name_starts_with: Some("private-".to_string()),
            channel_name_ends_with: Some("-notifications".to_string()),
        }),
        lambda: Some(LambdaConfig {
            client_context: None,
            async_invocation: true,
        }),
    }];

    println!("   Lambda Function: my-private-channels-function");
    println!("   Invocation Type: Async (Event)");
    println!("   Event Types: channel_occupied");
    println!("   Channel Filter: starts_with='private-', ends_with='-notifications'");
    println!();

    // Example 5: Multiple Lambda webhooks for different event types
    println!("5. Multiple Lambda webhooks for different event types:");
    let mut app5 = App::new(
        "app5".to_string(),
        "key5".to_string(),
        "secret5".to_string(),
    );

    app5.webhooks = vec![
        Webhook {
            url: None,
            lambda_function: Some("channel-lifecycle-function".to_string()),
            event_types: vec![
                "channel_occupied".to_string(),
                "channel_vacated".to_string(),
            ],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: Some(json!({"type": "channel_lifecycle"})),
                async_invocation: true,
            }),
        },
        Webhook {
            url: None,
            lambda_function: Some("presence-events-function".to_string()),
            event_types: vec!["member_added".to_string(), "member_removed".to_string()],
            headers: None,
            filter: Some(soketi_rs::app::WebhookFilter {
                channel_name_starts_with: Some("presence-".to_string()),
                channel_name_ends_with: None,
            }),
            lambda: Some(LambdaConfig {
                client_context: Some(json!({"type": "presence_events"})),
                async_invocation: false,
            }),
        },
        Webhook {
            url: None,
            lambda_function: Some("client-messages-function".to_string()),
            event_types: vec!["client_event".to_string()],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: Some(json!({"type": "client_messages"})),
                async_invocation: true,
            }),
        },
    ];

    println!("   Webhook 1:");
    println!("     Lambda Function: channel-lifecycle-function");
    println!("     Event Types: channel_occupied, channel_vacated");
    println!("     Invocation: Async");
    println!();
    println!("   Webhook 2:");
    println!("     Lambda Function: presence-events-function");
    println!("     Event Types: member_added, member_removed");
    println!("     Channel Filter: starts_with='presence-'");
    println!("     Invocation: Sync");
    println!();
    println!("   Webhook 3:");
    println!("     Lambda Function: client-messages-function");
    println!("     Event Types: client_event");
    println!("     Invocation: Async");
    println!();

    // Example 6: Mixed HTTP and Lambda webhooks
    println!("6. Mixed HTTP and Lambda webhooks:");
    let mut app6 = App::new(
        "app6".to_string(),
        "key6".to_string(),
        "secret6".to_string(),
    );

    app6.webhooks = vec![
        Webhook {
            url: Some("https://example.com/webhook".to_string()),
            lambda_function: None,
            event_types: vec!["channel_occupied".to_string()],
            headers: None,
            filter: None,
            lambda: None,
        },
        Webhook {
            url: None,
            lambda_function: Some("my-lambda-function".to_string()),
            event_types: vec!["channel_occupied".to_string()],
            headers: None,
            filter: None,
            lambda: Some(LambdaConfig {
                client_context: None,
                async_invocation: true,
            }),
        },
    ];

    println!("   Webhook 1: HTTP endpoint (https://example.com/webhook)");
    println!("   Webhook 2: Lambda function (my-lambda-function)");
    println!("   Both receive channel_occupied events");
    println!();

    // Example 7: Initialize WebhookSender with Lambda support
    println!("7. Initialize WebhookSender with Lambda support:");

    // Option A: With batching disabled
    let batching_config = BatchingConfig {
        enabled: false,
        duration_ms: 50,
    };
    let sender1 = WebhookSender::with_config(batching_config, true).await;
    println!("   Created WebhookSender with Lambda enabled (no batching)");

    // Option B: With batching enabled
    let batching_config = BatchingConfig {
        enabled: true,
        duration_ms: 100,
    };
    let sender2 = WebhookSender::with_config(batching_config, true).await;
    println!("   Created WebhookSender with Lambda enabled (batching: 100ms)");

    // Option C: With custom Lambda client
    let aws_config = aws_config::load_from_env().await;
    let lambda_client = aws_sdk_lambda::Client::new(&aws_config);
    let sender3 = WebhookSender::new().with_lambda_client(lambda_client);
    println!("   Created WebhookSender with custom Lambda client");
    println!();

    // Example 8: Sending webhooks (demonstration only - requires actual Lambda functions)
    println!("8. Sending webhooks to Lambda functions:");
    println!("   Note: The following would invoke actual Lambda functions if they exist:");
    println!();

    // This would send a channel_occupied webhook to the Lambda function
    // sender1.send_channel_occupied(&app1, "test-channel").await;
    println!("   sender.send_channel_occupied(&app, \"test-channel\").await;");
    println!("   -> Invokes Lambda function asynchronously");
    println!();

    // This would send a member_added webhook to the Lambda function
    // sender2.send_member_added(&app2, "presence-room", "user-123").await;
    println!("   sender.send_member_added(&app, \"presence-room\", \"user-123\").await;");
    println!("   -> Invokes Lambda function synchronously and waits for response");
    println!();

    // This would send a client_event webhook to the Lambda function
    // sender3.send_client_event(&app3, "private-chat", "client-message",
    //     json!({"text": "Hello"}), Some("socket-123"), Some("user-456")).await;
    println!("   sender.send_client_event(&app, \"private-chat\", \"client-message\",");
    println!(
        "       json!({{\"text\": \"Hello\"}}), Some(\"socket-123\"), Some(\"user-456\")).await;"
    );
    println!("   -> Invokes Lambda function with client context");
    println!();

    println!("=== Example Complete ===");
    println!();
    println!("Key Points:");
    println!("- Lambda webhooks support both async (Event) and sync (RequestResponse) invocation");
    println!("- Client context can be passed to Lambda functions for additional metadata");
    println!("- Channel name filtering works with Lambda webhooks");
    println!("- Multiple Lambda webhooks can be configured for different event types");
    println!("- HTTP and Lambda webhooks can be mixed in the same app");
    println!("- Batching is supported for Lambda webhooks");
}
