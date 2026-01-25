# RedisAdapter Implementation

## Overview

The RedisAdapter provides horizontal scaling support for the Pusher server using Redis pub/sub for message distribution across multiple server instances. It extends the LocalAdapter with Redis-based message broadcasting and synchronization capabilities.

## Requirements Satisfied

This implementation satisfies the following requirements from the specification:

- **Requirement 4.3**: Support a Redis adapter for horizontal scaling using Redis pub/sub
- **Requirement 4.5**: Broadcast messages across all instances when using horizontal adapters
- **Requirement 4.6**: Synchronize channel membership across instances when using horizontal adapters

## Architecture

### Components

1. **LocalAdapter**: Manages local socket connections and channel subscriptions
2. **Redis Client**: Handles Redis pub/sub connections for message distribution
3. **Message Types**: Serializable message types for different operations
4. **Node ID**: Unique identifier for each server instance to prevent message loops

### Message Flow

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  Server 1   │         │    Redis    │         │  Server 2   │
│             │         │   Pub/Sub   │         │             │
│ RedisAdapter│────────▶│             │────────▶│ RedisAdapter│
│             │ Publish │             │Subscribe│             │
│ LocalAdapter│         │             │         │ LocalAdapter│
└─────────────┘         └─────────────┘         └─────────────┘
      │                                                 │
      ▼                                                 ▼
  Local Sockets                                   Local Sockets
```

## Implementation Details

### RedisAdapter Structure

```rust
pub struct RedisAdapter {
    local: LocalAdapter,           // Manages local connections
    redis_client: Client,           // Redis client for pub/sub
    node_id: String,                // Unique node identifier
    channel_prefix: String,         // Redis channel prefix
    config: RedisAdapterConfig,     // Configuration
}
```

### Message Types

The adapter uses four message types for distributed operations:

1. **Broadcast**: Send a message to all subscribers of a channel
   - Includes: node_id, app_id, channel, message, except_socket_id
   - Used for: Event broadcasting across instances

2. **AddMember**: Add a member to a presence channel
   - Includes: node_id, app_id, channel, socket_id, member
   - Used for: Presence channel synchronization

3. **RemoveMember**: Remove a member from a presence channel
   - Includes: node_id, app_id, channel, socket_id
   - Used for: Presence channel cleanup

4. **TerminateUser**: Terminate all connections for a user
   - Includes: node_id, app_id, user_id
   - Used for: User disconnection across instances

### Key Features

#### 1. Message Broadcasting

When a message is sent to a channel:
1. The message is delivered to local sockets immediately
2. The message is published to Redis for distribution to other nodes
3. Other nodes receive the message and deliver it to their local sockets
4. Messages from self are ignored to prevent duplicates

#### 2. Presence Channel Synchronization

When a member joins or leaves a presence channel:
1. The member is added/removed from the local adapter
2. The operation is published to Redis
3. Other nodes receive the operation and update their local state
4. This ensures consistent presence data across all instances

#### 3. User Connection Termination

When terminating user connections:
1. Local connections are terminated immediately
2. A termination message is published to Redis
3. Other nodes receive the message and terminate their local connections
4. This ensures the user is disconnected from all instances

#### 4. Connection Management

The adapter maintains a persistent Redis pub/sub connection:
- Automatic reconnection on connection failure
- Exponential backoff for reconnection attempts
- Graceful handling of Redis unavailability

## Configuration

### RedisAdapterConfig

```rust
pub struct RedisAdapterConfig {
    pub host: String,              // Redis host (default: "127.0.0.1")
    pub port: u16,                 // Redis port (default: 6379)
    pub db: u8,                    // Redis database (default: 0)
    pub username: Option<String>,  // Redis username (optional)
    pub password: Option<String>,  // Redis password (optional)
    pub key_prefix: String,        // Key prefix (default: "pusher-rs")
}
```

### Environment Variables

The adapter can be configured using environment variables:

```bash
ADAPTER_DRIVER=redis
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_DB=0
REDIS_PASSWORD=your_password
REDIS_KEY_PREFIX=pusher-rs
```

## Usage

### Initialization

```rust
use soketi_rs::adapters::redis::RedisAdapter;
use soketi_rs::config::RedisAdapterConfig;

// Create with default config
let config = RedisAdapterConfig::default();
let adapter = RedisAdapter::new(config).await?;

// Initialize the adapter (starts pub/sub listener)
adapter.init().await?;
```

### Sending Messages

```rust
// Send a message to a channel
adapter.send("app-1", "my-channel", r#"{"event":"test","data":"hello"}"#, None).await?;

// Send a message excluding a specific socket
adapter.send("app-1", "my-channel", message, Some("socket-123")).await?;
```

### Presence Channels

```rust
use soketi_rs::app::PresenceMember;

// Add a member to a presence channel
let member = PresenceMember {
    user_id: "user-1".to_string(),
    user_info: serde_json::json!({"name": "John Doe"}),
};
adapter.add_member("app-1", "presence-channel", "socket-1", member).await?;

// Remove a member
adapter.remove_member("app-1", "presence-channel", "socket-1").await?;

// Get all members
let members = adapter.get_channel_members("app-1", "presence-channel").await?;
```

### User Management

```rust
// Terminate all connections for a user
adapter.terminate_user_connections("app-1", "user-1").await?;
```

## Testing

### Unit Tests

Basic unit tests verify:
- Adapter creation with various configurations
- Connection URL construction
- Authentication handling

### Integration Tests

Integration tests verify:
- Message broadcasting across multiple instances
- Presence channel synchronization
- User connection termination
- Redis pub/sub functionality

To run tests:

```bash
# Run all Redis adapter tests
cargo test redis_adapter

# Run with Redis available
cargo test --test redis_adapter_test -- --nocapture

# Run basic tests (don't require Redis)
cargo test --test redis_adapter_basic_test
```

## Performance Considerations

### Scalability

- The adapter scales horizontally by adding more server instances
- Each instance maintains its own local state
- Redis pub/sub provides efficient message distribution
- No single point of failure (except Redis itself)

### Latency

- Local message delivery is immediate (no network overhead)
- Cross-instance delivery depends on Redis latency
- Typical latency: 1-5ms for local Redis, 10-50ms for remote Redis

### Resource Usage

- Each instance maintains one Redis pub/sub connection
- Memory usage scales with local socket count
- Redis memory usage is minimal (only for pub/sub channels)

## Limitations

### Current Limitations

1. **Socket Count Queries**: Returns only local socket counts, not cluster-wide
2. **Channel Member Queries**: Returns only local members, not cluster-wide
3. **No Request-Response**: No support for querying other nodes directly

### Future Enhancements

1. **Cluster-wide Queries**: Implement request-response pattern for aggregated queries
2. **Health Monitoring**: Add health checks for Redis connection
3. **Metrics**: Add metrics for message distribution and latency
4. **Compression**: Add message compression for large payloads

## Comparison with ClusterAdapter

| Feature | RedisAdapter | ClusterAdapter |
|---------|--------------|----------------|
| Discovery | Not needed | UDP multicast |
| Message Distribution | Redis pub/sub | UDP broadcast |
| Scalability | High (Redis) | Limited (UDP) |
| Network Requirements | Redis server | Multicast support |
| Latency | Low (1-5ms) | Very low (<1ms) |
| Reliability | High (Redis) | Medium (UDP) |
| Setup Complexity | Medium | Low |

## Troubleshooting

### Common Issues

1. **Connection Failed**
   - Verify Redis is running: `redis-cli ping`
   - Check host and port configuration
   - Verify network connectivity

2. **Authentication Failed**
   - Verify username and password
   - Check Redis ACL configuration
   - Ensure user has pub/sub permissions

3. **Messages Not Received**
   - Check Redis pub/sub channels: `redis-cli PUBSUB CHANNELS`
   - Verify key_prefix matches across instances
   - Check for network issues

4. **High Latency**
   - Check Redis server load
   - Verify network latency to Redis
   - Consider using local Redis instance

## References

- [Redis Pub/Sub Documentation](https://redis.io/docs/manual/pubsub/)
- [Pusher Protocol Specification](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol)
- [Soketi Reference Implementation](https://github.com/soketi/soketi)
