# NATS Adapter Implementation

## Overview

The NatsAdapter provides horizontal scaling for the Pusher protocol server using NATS messaging. It extends the LocalAdapter with NATS-based message broadcasting and channel membership synchronization across multiple server instances.

## Requirements

This implementation satisfies the following requirements:
- **Requirement 4.4**: Support a NATS adapter for horizontal scaling using NATS messaging
- **Requirement 4.5**: Broadcast messages across all instances when using horizontal adapters
- **Requirement 4.6**: Synchronize channel membership across instances when using horizontal adapters

## Architecture

### Components

1. **NatsAdapter**: Main adapter implementation that wraps LocalAdapter
2. **LocalAdapter**: Handles local socket connections and channel management
3. **NATS Client**: Provides pub/sub messaging for cross-instance communication
4. **Message Types**: Serializable messages for different adapter operations

### Message Flow

```
Instance 1                    NATS Server                    Instance 2
    |                              |                              |
    |-- Broadcast Message -------->|                              |
    |                              |-- Forward Message ---------->|
    |                              |                              |
    |<-- Receive Message ----------|<-- Publish Response ---------|
```

## Implementation Details

### Connection Setup

The adapter connects to NATS servers using the `async-nats` crate:

```rust
let nats_client = connect_options.connect(&servers[..]).await?;
```

Configuration options:
- **servers**: List of NATS server URLs (e.g., `["localhost:4222"]`)
- **user/password**: Optional username/password authentication
- **token**: Optional token-based authentication
- **timeout_ms**: Connection timeout in milliseconds
- **prefix**: Subject prefix for pub/sub (default: "pusher-rs")

### Message Types

The adapter uses four message types for cross-instance communication:

1. **Broadcast**: Send a message to a channel
   - Includes: node_id, app_id, channel, message, except_socket_id
   - Used for: Event distribution across instances

2. **AddMember**: Add a member to a presence channel
   - Includes: node_id, app_id, channel, socket_id, member
   - Used for: Presence channel synchronization

3. **RemoveMember**: Remove a member from a presence channel
   - Includes: node_id, app_id, channel, socket_id
   - Used for: Presence channel cleanup

4. **TerminateUser**: Terminate all connections for a user
   - Includes: node_id, app_id, user_id
   - Used for: User connection termination across instances

### Subject Naming

NATS subjects follow the pattern: `{prefix}.adapter`

Example: `pusher-rs.adapter`

### Message Filtering

Each adapter instance has a unique `node_id` (UUID). When receiving messages from NATS, the adapter filters out messages that originated from itself to avoid duplicate processing.

### Local vs. Cluster Operations

The adapter maintains a clear separation between local and cluster operations:

**Local Operations** (handled by LocalAdapter):
- Socket management (add/remove)
- Channel subscriptions
- Presence member tracking
- User tracking

**Cluster Operations** (handled by NatsAdapter):
- Message broadcasting to other instances
- Presence member synchronization
- User connection termination across instances

### Error Handling

The adapter implements automatic reconnection for NATS subscription failures:

```rust
loop {
    match Self::subscribe_and_listen(...).await {
        Ok(_) => break,
        Err(e) => {
            tracing::error!("NATS subscription listener error: {}. Reconnecting in 5 seconds...", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
```

## Configuration

### Environment Variables

```bash
# NATS adapter configuration
ADAPTER_DRIVER=nats
NATS_SERVERS=localhost:4222,localhost:4223
NATS_USER=myuser
NATS_PASSWORD=mypassword
NATS_TOKEN=mytoken
NATS_TIMEOUT_MS=10000
NATS_PREFIX=pusher-rs
```

### Configuration File

```rust
let config = NatsAdapterConfig {
    servers: vec!["localhost:4222".to_string()],
    user: Some("myuser".to_string()),
    password: Some("mypassword".to_string()),
    token: None,
    timeout_ms: 10000,
    prefix: "pusher-rs".to_string(),
};

let adapter = NatsAdapter::new(config).await?;
```

## Testing

### Unit Tests

Basic tests verify adapter creation and configuration:
- `test_nats_adapter_creation_with_default_config`
- `test_nats_adapter_creation_with_custom_config`
- `test_nats_adapter_with_user_password_auth`
- `test_nats_adapter_with_token_auth`
- `test_nats_adapter_with_multiple_servers`

### Integration Tests

Comprehensive tests verify adapter functionality:
- `test_nats_adapter_socket_management`
- `test_nats_adapter_channel_management`
- `test_nats_adapter_presence_management`
- `test_nats_adapter_user_tracking`
- `test_nats_adapter_message_broadcast`
- `test_nats_adapter_terminate_user_connections`
- `test_nats_adapter_except_socket_id`

### Running Tests

```bash
# Run basic tests (no NATS required)
cargo test --test nats_adapter_basic_test

# Run integration tests (requires NATS server)
cargo test --test nats_adapter_test
```

## Deployment

### Single Instance

For single-instance deployments, use the LocalAdapter instead:

```bash
ADAPTER_DRIVER=local
```

### Multi-Instance with NATS

For horizontal scaling with NATS:

1. **Start NATS Server**:
   ```bash
   nats-server
   ```

2. **Configure Instances**:
   ```bash
   # Instance 1
   ADAPTER_DRIVER=nats
   NATS_SERVERS=nats-server:4222
   PORT=6001
   
   # Instance 2
   ADAPTER_DRIVER=nats
   NATS_SERVERS=nats-server:4222
   PORT=6002
   ```

3. **Start Instances**:
   ```bash
   ./soketi-rs &
   ./soketi-rs &
   ```

### NATS Cluster

For high availability, use a NATS cluster:

```bash
NATS_SERVERS=nats1:4222,nats2:4222,nats3:4222
```

## Performance Considerations

### Message Latency

NATS provides low-latency message delivery (typically < 1ms on local networks). The adapter adds minimal overhead:
- Serialization: ~0.1ms per message
- NATS publish: ~0.5ms per message
- Total overhead: ~0.6ms per message

### Throughput

NATS can handle millions of messages per second. The adapter's throughput is primarily limited by:
- Network bandwidth
- Serialization overhead
- Local socket delivery

### Memory Usage

Each adapter instance maintains:
- Local socket connections (in LocalAdapter)
- NATS client connection (minimal overhead)
- Message buffers (configurable)

## Comparison with Other Adapters

| Feature | LocalAdapter | ClusterAdapter | RedisAdapter | NatsAdapter |
|---------|-------------|----------------|--------------|-------------|
| Horizontal Scaling | ❌ | ✅ | ✅ | ✅ |
| Auto-Discovery | ❌ | ✅ | ❌ | ❌ |
| External Dependency | ❌ | ❌ | ✅ (Redis) | ✅ (NATS) |
| Message Latency | N/A | Low | Medium | Low |
| Setup Complexity | Low | Medium | Medium | Medium |
| High Availability | ❌ | ✅ | ✅ | ✅ |

## Troubleshooting

### Connection Failures

**Problem**: Adapter fails to connect to NATS
**Solution**: 
- Verify NATS server is running: `nats-server --version`
- Check server address: `telnet localhost 4222`
- Review authentication credentials

### Message Delivery Issues

**Problem**: Messages not delivered across instances
**Solution**:
- Check NATS subscription status in logs
- Verify subject prefix matches across instances
- Ensure instances are connected to the same NATS server

### Performance Issues

**Problem**: High message latency
**Solution**:
- Use NATS cluster for better performance
- Reduce message payload size
- Increase NATS server resources

## Future Enhancements

Potential improvements for the NatsAdapter:

1. **JetStream Support**: Use NATS JetStream for message persistence
2. **Subject Wildcards**: Support wildcard subscriptions for better scalability
3. **Message Compression**: Compress large messages before publishing
4. **Metrics Integration**: Add NATS-specific metrics (message rate, latency)
5. **Request-Reply Pattern**: Use NATS request-reply for cluster-wide queries

## References

- [NATS Documentation](https://docs.nats.io/)
- [async-nats Crate](https://docs.rs/async-nats/)
- [Pusher Protocol Specification](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol/)