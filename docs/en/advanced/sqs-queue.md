# SQS Queue Manager

The `SqsQueueManager` implements asynchronous webhook job processing using AWS SQS (Simple Queue Service).

## Features

- **Asynchronous Processing**: Jobs are enqueued to SQS and processed by background worker tasks
- **Batch Processing**: Workers retrieve up to 10 messages at once for efficient processing
- **Long Polling**: Reduces API calls by waiting up to 20 seconds for messages
- **Automatic Retries**: Failed jobs are automatically retried using SQS visibility timeout
- **Configurable Concurrency**: Run multiple worker tasks in parallel
- **Graceful Shutdown**: Workers stop cleanly when the queue manager is disconnected

## Configuration

```rust
use soketi_rs::queues::{SqsQueueManager, SqsQueueConfig};
use std::sync::Arc;

let config = SqsQueueConfig {
    queue_url: "https://sqs.us-east-1.amazonaws.com/123456789012/my-queue".to_string(),
    concurrency: 4,              // Number of concurrent workers
    batch_size: 10,              // Max messages per batch (1-10)
    wait_time_seconds: 20,       // Long polling wait time (0-20)
    visibility_timeout: 30,      // How long messages are hidden (seconds)
    max_retries: 3,              // Max retry attempts
    region: Some("us-east-1".to_string()),
};

let webhook_sender = Arc::new(WebhookSender::new());
let queue_manager = SqsQueueManager::new(config, webhook_sender).await?;
```

## Usage

### Starting Workers

```rust
// Start background workers to process jobs
queue_manager.start_workers().await?;
```

### Enqueueing Jobs

```rust
use soketi_rs::queues::WebhookJob;
use std::time::{SystemTime, UNIX_EPOCH};

let job = WebhookJob {
    app_id: "my_app".to_string(),
    app_key: "my_key".to_string(),
    app_secret: "my_secret".to_string(),
    event_type: "channel_occupied".to_string(),
    channel: "my-channel".to_string(),
    event: None,
    data: None,
    socket_id: None,
    user_id: None,
    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
};

queue_manager.enqueue(job).await?;
```

### Monitoring Queue

```rust
// Get approximate number of messages in queue
let queue_length = queue_manager.get_queue_length().await?;

// Get approximate number of messages being processed
let processing_length = queue_manager.get_processing_length().await?;
```

### Graceful Shutdown

```rust
// Stop workers and clean up resources
queue_manager.disconnect().await?;
```

## How It Works

### Message Flow

1. **Enqueue**: Jobs are serialized to JSON and sent to the SQS queue
2. **Receive**: Workers poll the queue using long polling (reduces API calls)
3. **Process**: Workers process messages in batches (up to 10 at once)
4. **Delete**: Successfully processed messages are deleted from the queue
5. **Retry**: Failed messages become visible again after the visibility timeout

### Retry Logic

The SqsQueueManager uses SQS's built-in retry mechanism:

- When a message is received, it becomes invisible for the `visibility_timeout` period
- If the worker successfully processes the message, it's deleted from the queue
- If the worker fails or crashes, the message becomes visible again after the timeout
- SQS will continue retrying until the message is deleted or moved to a Dead Letter Queue (DLQ)

### Batch Processing

Workers retrieve up to `batch_size` messages in a single API call:

- Reduces API costs and improves throughput
- Messages are processed sequentially within each batch
- Multiple workers can process different batches in parallel

### Long Polling

Workers use long polling to reduce empty responses:

- The `wait_time_seconds` parameter controls how long to wait for messages
- If no messages are available, SQS waits up to this duration before responding
- Reduces API calls and costs compared to short polling

## AWS Setup

### Prerequisites

1. **AWS Credentials**: Configure AWS credentials using one of these methods:
   - Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`
   - AWS credentials file: `~/.aws/credentials`
   - IAM role (for EC2/ECS/Lambda)

2. **SQS Queue**: Create an SQS queue in your AWS account:
   ```bash
   aws sqs create-queue --queue-name pusher-webhooks
   ```

3. **IAM Permissions**: Ensure your credentials have these permissions:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": [
           "sqs:SendMessage",
           "sqs:ReceiveMessage",
           "sqs:DeleteMessage",
           "sqs:GetQueueAttributes"
         ],
         "Resource": "arn:aws:sqs:*:*:pusher-webhooks"
       }
     ]
   }
   ```

### Dead Letter Queue (Recommended)

Configure a Dead Letter Queue (DLQ) to handle messages that fail repeatedly:

```bash
# Create DLQ
aws sqs create-queue --queue-name pusher-webhooks-dlq

# Configure redrive policy on main queue
aws sqs set-queue-attributes \
  --queue-url https://sqs.us-east-1.amazonaws.com/123456789012/pusher-webhooks \
  --attributes '{
    "RedrivePolicy": "{\"deadLetterTargetArn\":\"arn:aws:sqs:us-east-1:123456789012:pusher-webhooks-dlq\",\"maxReceiveCount\":\"5\"}"
  }'
```

## Performance Considerations

### Concurrency

- Higher concurrency = more parallel processing
- Each worker maintains its own SQS connection
- Recommended: 1-10 workers depending on workload

### Batch Size

- Larger batches = fewer API calls
- Maximum: 10 messages per batch (SQS limit)
- Recommended: 10 for high throughput, 1-5 for low latency

### Visibility Timeout

- Should be longer than the maximum processing time
- Too short: messages may be processed multiple times
- Too long: failed messages take longer to retry
- Recommended: 30-60 seconds

### Long Polling

- Reduces empty responses and API costs
- Maximum: 20 seconds (SQS limit)
- Recommended: 20 seconds for cost optimization

## Comparison with Other Queue Managers

| Feature | SyncQueueManager | RedisQueueManager | SqsQueueManager |
|---------|------------------|-------------------|-----------------|
| Async Processing | ❌ | ✅ | ✅ |
| Distributed | ❌ | ✅ | ✅ |
| Batch Processing | ❌ | ❌ | ✅ |
| Retry Logic | ❌ | ✅ | ✅ (built-in) |
| External Dependency | None | Redis | AWS SQS |
| Cost | Free | Redis hosting | SQS API calls |
| Best For | Development | Self-hosted | AWS deployments |

## Testing

### Unit Tests

Run unit tests (no AWS required):
```bash
cargo test --test sqs_queue_manager_test
```

### Integration Tests

Run integration tests (requires AWS credentials and SQS queue):
```bash
export TEST_SQS_QUEUE_URL="https://sqs.us-east-1.amazonaws.com/123456789012/test-queue"
cargo test --test sqs_queue_manager_test -- --ignored
```

## Troubleshooting

### Connection Issues

**Problem**: Workers fail to connect to SQS

**Solutions**:
- Verify AWS credentials are configured correctly
- Check IAM permissions
- Verify the queue URL is correct
- Check network connectivity to AWS

### Messages Not Being Processed

**Problem**: Messages remain in the queue

**Solutions**:
- Verify workers are started: `queue_manager.start_workers().await?`
- Check worker logs for errors
- Verify the queue URL is correct
- Check visibility timeout isn't too long

### Duplicate Processing

**Problem**: Messages are processed multiple times

**Solutions**:
- Increase visibility timeout
- Ensure messages are deleted after successful processing
- Check for worker crashes or errors

### High API Costs

**Problem**: Too many SQS API calls

**Solutions**:
- Increase `wait_time_seconds` to 20 (long polling)
- Increase `batch_size` to 10
- Reduce number of workers if queue is often empty

## Requirements

This implementation satisfies the following requirements:

- **9.3**: Support an SQS queue driver for AWS deployments
- **9.4**: Enqueue webhook jobs with retry logic
- **9.6**: Support batch processing for SQS queues

## See Also

- [Redis Queue Manager](redis_queue_manager.md)
- [Sync Queue Manager](sync_queue_manager.md)
- [Webhook Sender](webhook_sender.md)
- [AWS SQS Documentation](https://docs.aws.amazon.com/sqs/)
