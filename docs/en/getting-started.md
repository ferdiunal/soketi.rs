# Getting Started with Soketi

> A quick guide to get you up and running with Soketi, the high-performance Pusher protocol-compatible WebSocket server written in Rust.

## Table of Contents

- [Introduction](#introduction)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Basic Configuration](#basic-configuration)
- [Your First Connection](#your-first-connection)
- [WebSocket Connection Examples](#websocket-connection-examples)
- [Next Steps](#next-steps)

## Introduction

Soketi is a fast, scalable WebSocket server that implements the Pusher protocol. It allows you to build real-time applications with ease, supporting public channels, private channels, and presence channels.

## Quick Start

The fastest way to get started with Soketi is using Docker:

```bash
docker run -p 6001:6001 -p 9601:9601 \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  quay.io/soketi/soketi:latest-16-alpine
```

Your Soketi server is now running on `ws://localhost:6001`!

## Installation

### Using Docker (Recommended)

Docker is the easiest way to run Soketi:

```bash
docker pull quay.io/soketi/soketi:latest-16-alpine
```

### Using Cargo

If you have Rust installed, you can build from source:

```bash
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
./target/release/soketi
```

### Using Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/soketi/soketi.rs/releases).

## Basic Configuration

Create a `config.json` file:

```json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "max_connections": 100,
          "enable_client_messages": true,
          "enabled": true,
          "max_backend_events_per_second": 100,
          "max_client_events_per_second": 100,
          "max_read_requests_per_second": 100
        }
      ]
    }
  },
  "host": "0.0.0.0",
  "port": 6001,
  "metrics": {
    "enabled": true,
    "port": 9601,
    "host": "0.0.0.0"
  }
}
```

Start Soketi with your configuration:

```bash
soketi --config-file config.json
```

> **Note:** Use `--config-file` (not `--config`) to specify the configuration file path.

## Your First Connection

### Using JavaScript/TypeScript

Install the Pusher JavaScript library:

```bash
npm install pusher-js
```

Connect to your Soketi server:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
});

const channel = pusher.subscribe('my-channel');

channel.bind('my-event', (data: any) => {
  console.log('Received:', data);
});

console.log('Connected to Soketi!');
```

### Testing Your Connection

You can trigger events using the HTTP API:

```bash
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-event",
    "channel": "my-channel",
    "data": "{\"message\": \"Hello from Soketi!\"}"
  }'
```

## WebSocket Connection Examples

### Basic Public Channel

Public channels are open to all clients and don't require authentication:

```typescript
import Pusher from 'pusher-js';

// Initialize Pusher client
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1', // Optional, for compatibility
});

// Subscribe to a public channel
const channel = pusher.subscribe('public-channel');

// Bind to events
channel.bind('message', (data: any) => {
  console.log('New message:', data);
});

// Connection state events
pusher.connection.bind('connected', () => {
  console.log('Connected to Soketi');
});

pusher.connection.bind('disconnected', () => {
  console.log('Disconnected from Soketi');
});

pusher.connection.bind('error', (err: any) => {
  console.error('Connection error:', err);
});
```

### Private Channel with Authentication

Private channels require authentication before clients can subscribe:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/pusher/auth', // Your auth endpoint
  auth: {
    headers: {
      'Authorization': 'Bearer your-token-here'
    }
  }
});

// Subscribe to a private channel (must start with 'private-')
const privateChannel = pusher.subscribe('private-user-123');

privateChannel.bind('pusher:subscription_succeeded', () => {
  console.log('Successfully subscribed to private channel');
});

privateChannel.bind('notification', (data: any) => {
  console.log('Private notification:', data);
});

privateChannel.bind('pusher:subscription_error', (status: any) => {
  console.error('Subscription failed:', status);
});
```

### Presence Channel with User Info

Presence channels track which users are subscribed and provide member information:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/pusher/auth',
  auth: {
    headers: {
      'Authorization': 'Bearer your-token-here'
    }
  }
});

// Subscribe to a presence channel (must start with 'presence-')
const presenceChannel = pusher.subscribe('presence-chat-room');

// When subscription succeeds, get all members
presenceChannel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Current members:', members.count);
  members.each((member: any) => {
    console.log('Member:', member.id, member.info);
  });
});

// When a new member joins
presenceChannel.bind('pusher:member_added', (member: any) => {
  console.log('Member joined:', member.id, member.info);
});

// When a member leaves
presenceChannel.bind('pusher:member_removed', (member: any) => {
  console.log('Member left:', member.id);
});

// Regular events on presence channel
presenceChannel.bind('chat-message', (data: any) => {
  console.log('Chat message:', data);
});
```

### Client Events (Peer-to-Peer)

Client events allow clients to send messages directly to other clients on the same channel:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/pusher/auth',
});

// Client events only work on private and presence channels
const channel = pusher.subscribe('private-chat');

channel.bind('pusher:subscription_succeeded', () => {
  // Trigger a client event (must start with 'client-')
  channel.trigger('client-typing', {
    user: 'John',
    isTyping: true
  });
});

// Listen for client events from other users
channel.bind('client-typing', (data: any) => {
  console.log(`${data.user} is typing:`, data.isTyping);
});
```

### Handling Reconnection

Soketi automatically handles reconnection, but you can customize the behavior:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  // Reconnection settings
  activityTimeout: 30000, // 30 seconds
  pongTimeout: 10000, // 10 seconds
});

const channel = pusher.subscribe('my-channel');

// Track connection state
pusher.connection.bind('state_change', (states: any) => {
  console.log(`Connection state changed from ${states.previous} to ${states.current}`);
});

// Handle reconnection
pusher.connection.bind('connected', () => {
  console.log('Connected! Socket ID:', pusher.connection.socket_id);
});

pusher.connection.bind('connecting', () => {
  console.log('Connecting to Soketi...');
});

pusher.connection.bind('unavailable', () => {
  console.log('Connection unavailable, will retry...');
});

pusher.connection.bind('failed', () => {
  console.error('Connection failed permanently');
});
```

### Complete Example: Real-time Chat

Here's a complete example combining multiple concepts:

```typescript
import Pusher from 'pusher-js';

// Initialize Pusher
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/api/pusher/auth',
  auth: {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    }
  }
});

// Subscribe to presence channel
const chatChannel = pusher.subscribe('presence-chat-room');

// Handle subscription success
chatChannel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log(`Connected! ${members.count} users online`);
  updateUserList(members);
});

// Handle new members
chatChannel.bind('pusher:member_added', (member: any) => {
  console.log(`${member.info.name} joined`);
  addUserToList(member);
});

// Handle members leaving
chatChannel.bind('pusher:member_removed', (member: any) => {
  console.log(`${member.info.name} left`);
  removeUserFromList(member);
});

// Handle chat messages
chatChannel.bind('chat-message', (data: any) => {
  displayMessage(data);
});

// Handle typing indicators
chatChannel.bind('client-typing', (data: any) => {
  showTypingIndicator(data.userId, data.isTyping);
});

// Send a message
function sendMessage(message: string) {
  fetch('/api/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    },
    body: JSON.stringify({
      channel: 'presence-chat-room',
      event: 'chat-message',
      data: {
        message,
        userId: getCurrentUserId(),
        userName: getCurrentUserName(),
        timestamp: new Date().toISOString()
      }
    })
  });
}

// Send typing indicator
function sendTypingIndicator(isTyping: boolean) {
  chatChannel.trigger('client-typing', {
    userId: getCurrentUserId(),
    isTyping
  });
}

// Connection monitoring
pusher.connection.bind('state_change', (states: any) => {
  updateConnectionStatus(states.current);
});

// Helper functions (implement based on your UI)
function updateUserList(members: any) { /* ... */ }
function addUserToList(member: any) { /* ... */ }
function removeUserFromList(member: any) { /* ... */ }
function displayMessage(data: any) { /* ... */ }
function showTypingIndicator(userId: string, isTyping: boolean) { /* ... */ }
function updateConnectionStatus(status: string) { /* ... */ }
function getCurrentUserId(): string { return 'user-123'; }
function getCurrentUserName(): string { return 'John Doe'; }
```

## Next Steps

Now that you have Soketi running, explore these topics:

- **[Installation Guide](installation.md)** - Detailed installation options
- **[Configuration Reference](configuration.md)** - Complete configuration options
- **[API Reference](api-reference.md)** - WebSocket and HTTP API documentation
- **[Deployment Guide](deployment/reverse-proxy.md)** - Production deployment with reverse proxies
- **[Examples](examples/basic-chat.md)** - Code examples and tutorials

## Related Resources

- [Official Pusher Documentation](https://pusher.com/docs)
- [Soketi GitHub Repository](https://github.com/soketi/soketi.rs)
- [Troubleshooting Guide](troubleshooting.md)
