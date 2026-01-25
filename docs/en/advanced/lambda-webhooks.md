# AWS Lambda Webhook Integration

This document describes how to configure and use AWS Lambda functions as webhook endpoints for Pusher events in the soketi-rs server.

## Overview

The soketi-rs server supports invoking AWS Lambda functions as webhook endpoints, providing a serverless alternative to traditional HTTP webhooks. Lambda webhooks support:

- **Async invocation** (Event): Fire-and-forget invocation that doesn't wait for a response
- **Sync invocation** (RequestResponse): Waits for the Lambda function to complete and returns the response
- **Client context**: Pass custom metadata to Lambda functions
- **Channel filtering**: Filter events by channel name prefix/suffix
- **Batching**: Batch multiple events together before invoking Lambda

## Configuration

### Basic Lambda Webhook

To configure a Lambda webhook, set the `lambda_function` field in the webhook configuration:

```rust
use soketi_rs::app::{App, Webhook, LambdaConfig};

let mut app = App::new(
    "app-id".to_string(),
    "app-key".to_string(),
    "app-secret".to_string(),
);

app.webhooks = vec![Webhook {
    url: None,  // No HTTP URL
    lambda_function: Some("my-lambda-function".to_string()),
    event_types: vec!["channel_occupied".to_string()],
    headers: None,
    filter: None,
    lambda: Some(LambdaConfig {
        client_context: None,
        async_invocation: true,
    }),
}];
```

### Invocation Types

#### Async Invocation (Event)

Async invocation is fire-and-forget. The Lambda function is invoked asynchronously and the server doesn't wait for a response:

```rust
lambda: Some(LambdaConfig {
    client_context: None,
    async_invocation: true,  // Fire-and-forget
})
```

**Use cases:**
- High-throughput event processing
- Non-critical notifications
- Background processing
- When you don't need to know if the Lambda succeeded

#### Sync Invocation (RequestResponse)

Sync invocation waits for the Lambda function to complete and returns the response:

```rust
lambda: Some(LambdaConfig {
    client_context: None,
    async_invocation: false,  // Wait for response
})
```

**Use cases:**
- Critical event processing
- When you need to know if the Lambda succeeded
- When you need the Lambda's response
- Lower-throughput scenarios

### Client Context

You can pass custom metadata to Lambda functions using client context:

```rust
use serde_json::json;

lambda: Some(LambdaConfig {
    client_context: Some(json!({
        "custom": {
            "app_id": "my-app",
            "environment": "production",
            "region": "us-east-1"
        }
    })),
    async_invocation: true,
})
```

The client context is available in the Lambda function's context object.

### Channel Filtering

Lambda webhooks support channel name filtering:

```rust
use soketi_rs::app::WebhookFilter;

Webhook {
    url: None,
    lambda_function: Some("my-function".to_string()),
    event_types: vec!["channel_occupied".to_string()],
    headers: None,
    filter: Some(WebhookFilter {
        channel_name_starts_with: Some("private-".to_string()),
        channel_name_ends_with: Some("-notifications".to_string()),
    }),
    lambda: Some(LambdaConfig {
        client_context: None,
        async_invocation: true,
    }),
}
```

This webhook will only be invoked for channels that start with `private-` and end with `-notifications`.

## Event Types

Lambda webhooks support all Pusher event types:

- `channel_occupied`: When a channel becomes occupied (0 → 1+ subscribers)
- `channel_vacated`: When a channel becomes vacated (1+ → 0 subscribers)
- `member_added`: When a member joins a presence channel
- `member_removed`: When a member leaves a presence channel
- `client_event`: When a client sends an event
- `cache_miss`: When a cache miss occurs on a cache-enabled channel

Example with multiple event types:

```rust
Webhook {
    url: None,
    lambda_function: Some("my-function".to_string()),
    event_types: vec![
        "channel_occupied".to_string(),
        "channel_vacated".to_string(),
        "member_added".to_string(),
        "member_removed".to_string(),
    ],
    headers: None,
    filter: None,
    lambda: Some(LambdaConfig {
        client_context: None,
        async_invocation: true,
    }),
}
```

## Webhook Payload

Lambda functions receive a JSON payload with the following structure:

```json
{
  "time_ms": 1234567890123,
  "events": [
    {
      "name": "channel_occupied",
      "channel": "my-channel",
      "event": null,
      "data": null,
      "socket_id": null,
      "user_id": null
    }
  ]
}
```

### Payload Fields

- `time_ms`: Unix timestamp in milliseconds when the webhook was sent
- `events`: Array of events (may contain multiple events if batching is enabled)
  - `name`: Event type (e.g., "channel_occupied", "member_added")
  - `channel`: Channel name
  - `event`: Event name (for client_event only)
  - `data`: Event data (for client_event only)
  - `socket_id`: Socket ID (for client_event only)
  - `user_id`: User ID (for member events and client_event on presence channels)

## Batching

Lambda webhooks support batching multiple events together:

```rust
use soketi_rs::webhook_sender::{WebhookSender, BatchingConfig};

let batching_config = BatchingConfig {
    enabled: true,
    duration_ms: 100,  // Batch events for 100ms
};

let sender = WebhookSender::with_config(batching_config, true).await;
```

When batching is enabled:
- Events are collected for the specified duration
- Multiple events are sent in a single Lambda invocation
- The `events` array in the payload will contain multiple events
- Reduces Lambda invocation costs for high-throughput scenarios

## Initialization

### With Default AWS Configuration

```rust
use soketi_rs::webhook_sender::{WebhookSender, BatchingConfig};

let batching_config = BatchingConfig {
    enabled: false,
    duration_ms: 50,
};

// Initialize with Lambda support (uses default AWS config from environment)
let sender = WebhookSender::with_config(batching_config, true).await;
```

### With Custom Lambda Client

```rust
use soketi_rs::webhook_sender::WebhookSender;

// Create custom AWS config and Lambda client
let aws_config = aws_config::load_from_env().await;
let lambda_client = aws_sdk_lambda::Client::new(&aws_config);

// Create WebhookSender with custom Lambda client
let sender = WebhookSender::new()
    .with_lambda_client(lambda_client);
```

## AWS Credentials

The Lambda client requires AWS credentials to be configured. You can provide credentials using:

1. **Environment variables:**
   ```bash
   export AWS_ACCESS_KEY_ID=your_access_key
   export AWS_SECRET_ACCESS_KEY=your_secret_key
   export AWS_REGION=us-east-1
   ```

2. **AWS credentials file** (`~/.aws/credentials`):
   ```ini
   [default]
   aws_access_key_id = your_access_key
   aws_secret_access_key = your_secret_key
   ```

3. **AWS config file** (`~/.aws/config`):
   ```ini
   [default]
   region = us-east-1
   ```

4. **IAM role** (when running on EC2, ECS, or Lambda)

## IAM Permissions

The AWS credentials must have permission to invoke Lambda functions. Required IAM policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "lambda:InvokeFunction",
        "lambda:InvokeAsync"
      ],
      "Resource": "arn:aws:lambda:*:*:function:*"
    }
  ]
}
```

For production, restrict the `Resource` to specific Lambda function ARNs.

## Mixed HTTP and Lambda Webhooks

You can configure both HTTP and Lambda webhooks for the same app:

```rust
app.webhooks = vec![
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
```

Both webhooks will receive the same events.

## Example Lambda Function

Here's an example Lambda function (Node.js) that processes webhook events:

```javascript
exports.handler = async (event) => {
    console.log('Received webhook:', JSON.stringify(event, null, 2));
    
    const { time_ms, events } = event;
    
    for (const evt of events) {
        console.log(`Event: ${evt.name}, Channel: ${evt.channel}`);
        
        switch (evt.name) {
            case 'channel_occupied':
                // Handle channel occupied
                console.log(`Channel ${evt.channel} is now occupied`);
                break;
            case 'channel_vacated':
                // Handle channel vacated
                console.log(`Channel ${evt.channel} is now vacated`);
                break;
            case 'member_added':
                // Handle member added
                console.log(`User ${evt.user_id} joined ${evt.channel}`);
                break;
            case 'member_removed':
                // Handle member removed
                console.log(`User ${evt.user_id} left ${evt.channel}`);
                break;
            case 'client_event':
                // Handle client event
                console.log(`Client event ${evt.event} on ${evt.channel}`);
                break;
        }
    }
    
    return {
        statusCode: 200,
        body: JSON.stringify({ message: 'Webhook processed successfully' })
    };
};
```

## Error Handling

Lambda invocation errors are logged but don't block the webhook sender:

```rust
// If Lambda invocation fails, an error is logged
Log::error(&format!("Lambda webhook failed: {}", e));
```

For async invocations, errors are not returned to the caller. For sync invocations, errors are logged but the webhook sender continues processing.

## Best Practices

1. **Use async invocation for high-throughput scenarios** to avoid blocking
2. **Use sync invocation for critical events** where you need to know if processing succeeded
3. **Enable batching** to reduce Lambda invocation costs
4. **Use channel filtering** to reduce unnecessary Lambda invocations
5. **Set appropriate IAM permissions** to restrict Lambda access
6. **Monitor Lambda metrics** (invocations, errors, duration) in CloudWatch
7. **Use client context** to pass app-specific metadata to Lambda functions
8. **Test Lambda functions** with sample payloads before deploying

## Troubleshooting

### Lambda function not being invoked

1. Check AWS credentials are configured correctly
2. Verify IAM permissions allow Lambda invocation
3. Check Lambda function name is correct
4. Verify event types match the events being triggered
5. Check channel filters if configured

### Lambda invocation errors

1. Check CloudWatch logs for Lambda function errors
2. Verify Lambda function has correct permissions
3. Check Lambda function timeout settings
4. Verify payload format is correct

### Performance issues

1. Use async invocation for non-critical events
2. Enable batching to reduce invocation count
3. Optimize Lambda function cold start time
4. Use provisioned concurrency for consistent performance

## See Also

- [Webhook Sender Documentation](./WEBHOOK_SENDER.md)
- [AWS Lambda Documentation](https://docs.aws.amazon.com/lambda/)
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
