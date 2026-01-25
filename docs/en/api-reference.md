# API Reference

> Complete API documentation for Soketi WebSocket and HTTP APIs.

## Table of Contents

- [WebSocket API](#websocket-api)
- [HTTP API](#http-api)
- [Authentication](#authentication)
- [Channel Types](#channel-types)
- [Events](#events)
- [Error Codes](#error-codes)

## WebSocket API

Soketi implements the Pusher WebSocket protocol. Clients connect via WebSocket and communicate using JSON messages.

### Connection

Connect to Soketi using the Pusher client library:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
});
```

### Connection Events

| Event | Description |
|-------|-------------|
| `pusher:connection_established` | Fired when connection is established |
| `pusher:error` | Fired when an error occurs |

Example:

```typescript
pusher.connection.bind('connected', () => {
  console.log('Connected to Soketi');
});

pusher.connection.bind('error', (err: any) => {
  console.error('Connection error:', err);
});
```

### Subscribing to Channels

#### Public Channels

```typescript
const channel = pusher.subscribe('my-channel');

channel.bind('my-event', (data: any) => {
  console.log('Received:', data);
});
```

#### Private Channels

Private channels require authentication:

```typescript
const privateChannel = pusher.subscribe('private-my-channel');

privateChannel.bind('my-event', (data: any) => {
  console.log('Received:', data);
});
```

#### Presence Channels

Presence channels track online users:

```typescript
const presenceChannel = pusher.subscribe('presence-my-channel');

presenceChannel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Online users:', members.count);
  members.each((member: any) => {
    console.log('User:', member.id, member.info);
  });
});

presenceChannel.bind('pusher:member_added', (member: any) => {
  console.log('User joined:', member.id);
});

presenceChannel.bind('pusher:member_removed', (member: any) => {
  console.log('User left:', member.id);
});
```

### Client Events

Send events from client to other clients (must be enabled in app configuration):

```typescript
channel.trigger('client-my-event', {
  message: 'Hello from client!'
});
```

**Requirements**:
- Channel must be private or presence
- `enable_client_messages` must be true in app configuration
- Event name must start with `client-`

### Unsubscribing

```typescript
pusher.unsubscribe('my-channel');
```

### Disconnecting

```typescript
pusher.disconnect();
```

## HTTP API

The HTTP API allows you to trigger events, query channel information, and manage connections from your backend.

### Base URL

```
http://localhost:6001/apps/{app_id}
```

### Authentication

All HTTP API requests must be authenticated using HMAC SHA256 signature.

#### Request Signing

1. Create auth string: `POST\n/apps/{app_id}/events\nauth_key={key}&auth_timestamp={timestamp}&auth_version=1.0&body_md5={md5}`
2. Sign with HMAC SHA256 using app secret
3. Add signature to query parameters

Example using Node.js:

```typescript
import crypto from 'crypto';

function signRequest(
  method: string,
  path: string,
  params: Record<string, string>,
  body: string,
  secret: string
): string {
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `${method}\n${path}\n${sortedParams}&body_md5=${bodyMd5}`;
  
  return crypto
    .createHmac('sha256', secret)
    .update(authString)
    .digest('hex');
}
```

### Trigger Events

Send events to channels from your backend.

**Endpoint**: `POST /apps/{app_id}/events`

**Request Parameters**:

| Parameter | Type | Description | Required |
|-----------|------|-------------|----------|
| `name` | string | Event name | Yes |
| `data` | string or object | Event data (JSON string or object) | Yes |
| `channel` | string | Single channel name | No* |
| `channels` | array | Array of channel names | No* |
| `socket_id` | string | Socket ID to exclude from receiving the event | No |

*Either `channel` or `channels` must be provided

**Request Body Examples**:

**Single Channel**:

```json
{
  "name": "my-event",
  "channel": "my-channel",
  "data": "{\"message\": \"Hello World!\"}"
}
```

**Multiple Channels**:

```json
{
  "name": "my-event",
  "channels": ["channel-1", "channel-2", "channel-3"],
  "data": "{\"message\": \"Hello World!\"}"
}
```

**With Socket ID Exclusion**:

```json
{
  "name": "message-sent",
  "channel": "chat-room",
  "data": "{\"user\": \"John\", \"message\": \"Hi everyone!\"}",
  "socket_id": "123.456"
}
```

**Response**:

```json
{}
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | Event triggered successfully |
| `400` | Invalid request (validation error) |
| `401` | Authentication failed |
| `403` | Forbidden (rate limit exceeded) |
| `413` | Payload too large |

**Example using Pusher Server SDK (Recommended)**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Trigger to single channel
await pusher.trigger('my-channel', 'my-event', {
  message: 'Hello World!'
});

// Trigger to multiple channels
await pusher.trigger(['channel-1', 'channel-2'], 'my-event', {
  message: 'Hello World!'
});

// Trigger with socket ID exclusion
await pusher.trigger('chat-room', 'message-sent', {
  user: 'John',
  message: 'Hi everyone!'
}, {
  socket_id: '123.456'
});
```

**Example using cURL**:

```bash
# Generate authentication parameters
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
TIMESTAMP=$(date +%s)
BODY='{"name":"my-event","channel":"my-channel","data":"{\"message\":\"Hello!\"}"}'
BODY_MD5=$(echo -n "$BODY" | md5sum | cut -d' ' -f1)

# Create auth string and signature
AUTH_STRING="POST
/apps/$APP_ID/events
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&body_md5=$BODY_MD5"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# Make request
curl -X POST "http://localhost:6001/apps/$APP_ID/events?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE&body_md5=$BODY_MD5" \
  -H "Content-Type: application/json" \
  -d "$BODY"
```

**Example using Axios (Node.js)**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function triggerEvent(
  appId: string,
  appKey: string,
  appSecret: string,
  channel: string,
  event: string,
  data: any
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const body = JSON.stringify({
    name: event,
    channel: channel,
    data: JSON.stringify(data),
  });
  
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
    body_md5: bodyMd5,
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `POST\n/apps/${appId}/events\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001/apps/${appId}/events?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.post(url, body, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  return response.data;
}

// Usage
await triggerEvent(
  'app-id',
  'app-key',
  'app-secret',
  'my-channel',
  'my-event',
  { message: 'Hello World!' }
);
```

**Validation Limits**:

| Limit | Default | Description |
|-------|---------|-------------|
| Event name length | 200 characters | Maximum length of event name |
| Channel name length | 200 characters | Maximum length of channel name |
| Payload size | 10 KB | Maximum size of event data |
| Channels per request | 100 | Maximum number of channels in one request |

These limits can be configured per application in the app configuration.

### Batch Events

Trigger multiple events in a single request for better performance.

**Endpoint**: `POST /apps/{app_id}/batch_events`

**Request Body**:

```json
{
  "batch": [
    {
      "name": "event-1",
      "channel": "channel-1",
      "data": "{\"message\": \"Event 1\"}"
    },
    {
      "name": "event-2",
      "channels": ["channel-2", "channel-3"],
      "data": "{\"message\": \"Event 2\"}"
    },
    {
      "name": "event-3",
      "channel": "channel-4",
      "data": "{\"user\": \"John\", \"action\": \"joined\"}",
      "socket_id": "123.456"
    }
  ]
}
```

**Response**:

```json
{}
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | All events triggered successfully |
| `400` | Invalid request (validation error) |
| `401` | Authentication failed |
| `403` | Forbidden (rate limit exceeded) |

**Example using Pusher Server SDK**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Trigger batch events
await pusher.triggerBatch([
  {
    channel: 'channel-1',
    name: 'event-1',
    data: { message: 'Event 1' },
  },
  {
    channel: 'channel-2',
    name: 'event-2',
    data: { message: 'Event 2' },
  },
  {
    channel: 'channel-3',
    name: 'event-3',
    data: { message: 'Event 3' },
  },
]);
```

**Example using Axios**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function triggerBatchEvents(
  appId: string,
  appKey: string,
  appSecret: string,
  events: Array<{
    channel: string;
    name: string;
    data: any;
    socket_id?: string;
  }>
) {
  const timestamp = Math.floor(Date.now() / 1000);
  
  const body = JSON.stringify({
    batch: events.map(event => ({
      name: event.name,
      channel: event.channel,
      data: JSON.stringify(event.data),
      socket_id: event.socket_id,
    })),
  });
  
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
    body_md5: bodyMd5,
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `POST\n/apps/${appId}/batch_events\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001/apps/${appId}/batch_events?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.post(url, body, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  return response.data;
}

// Usage
await triggerBatchEvents(
  'app-id',
  'app-key',
  'app-secret',
  [
    { channel: 'channel-1', name: 'event-1', data: { message: 'Event 1' } },
    { channel: 'channel-2', name: 'event-2', data: { message: 'Event 2' } },
    { channel: 'channel-3', name: 'event-3', data: { message: 'Event 3' } },
  ]
);
```

**Batch Size Limits**:

| Limit | Default | Description |
|-------|---------|-------------|
| Batch size | 10 events | Maximum number of events per batch request |

**Benefits of Batch Events**:

- **Reduced Network Overhead**: Single HTTP request instead of multiple
- **Better Performance**: Lower latency for multiple events
- **Atomic Operations**: All events are processed together
- **Rate Limit Efficiency**: Counts as one request for rate limiting

### Get Channels

List all active channels with their subscription information.

**Endpoint**: `GET /apps/{app_id}/channels`

**Query Parameters**:

| Parameter | Type | Description | Required |
|-----------|------|-------------|----------|
| `filter_by_prefix` | string | Filter channels by prefix | No |
| `info` | string | Comma-separated list of attributes to include | No |

**Available Info Attributes**:

- `user_count` - Number of unique users (presence channels only)
- `subscription_count` - Number of active subscriptions

**Response**:

```json
{
  "channels": {
    "channel-1": {
      "subscription_count": 3,
      "occupied": true
    },
    "channel-2": {
      "subscription_count": 1,
      "occupied": true
    },
    "presence-room": {
      "subscription_count": 5,
      "occupied": true,
      "user_count": 5
    }
  }
}
```

**Example - List All Channels**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Get all channels
const result = await pusher.get({ path: '/channels' });
console.log(result.channels);
```

**Example - Filter by Prefix**:

```typescript
// Get only presence channels
const result = await pusher.get({
  path: '/channels',
  params: { filter_by_prefix: 'presence-' }
});
console.log(result.channels);
```

**Example - Include User Count**:

```typescript
// Get channels with user count info
const result = await pusher.get({
  path: '/channels',
  params: { info: 'user_count' }
});
console.log(result.channels);
```

**Example using Axios**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function getChannels(
  appId: string,
  appKey: string,
  appSecret: string,
  filterPrefix?: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/channels`;
  
  const params: Record<string, string> = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  if (filterPrefix) {
    params.filter_by_prefix = filterPrefix;
  }
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `GET\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.get(url);
  return response.data;
}

// Usage
const channels = await getChannels('app-id', 'app-key', 'app-secret');
console.log(channels);

// With filter
const presenceChannels = await getChannels(
  'app-id',
  'app-key',
  'app-secret',
  'presence-'
);
console.log(presenceChannels);
```

**Example using cURL**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
TIMESTAMP=$(date +%s)

# Create auth string
AUTH_STRING="GET
/apps/$APP_ID/channels
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# Make request
curl "http://localhost:6001/apps/$APP_ID/channels?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE"

# With filter
curl "http://localhost:6001/apps/$APP_ID/channels?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE&filter_by_prefix=presence-"
```

**Notes**:

- Only channels with at least one active subscription are returned
- Empty channels are automatically excluded from the response
- The `occupied` field is always `true` for returned channels

### Get Channel Info

Get detailed information about a specific channel.

**Endpoint**: `GET /apps/{app_id}/channels/{channel_name}`

**Query Parameters**:

| Parameter | Type | Description | Required |
|-----------|------|-------------|----------|
| `info` | string | Comma-separated list of attributes to include | No |

**Available Info Attributes**:

- `user_count` - Number of unique users (presence channels only)
- `subscription_count` - Number of active subscriptions

**Response for Regular Channel**:

```json
{
  "occupied": true,
  "subscription_count": 5
}
```

**Response for Presence Channel**:

```json
{
  "occupied": true,
  "subscription_count": 5,
  "user_count": 5
}
```

**Example using Pusher SDK**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Get channel info
const result = await pusher.get({
  path: '/channels/my-channel',
  params: { info: 'subscription_count' }
});
console.log(result);

// Get presence channel info with user count
const presenceResult = await pusher.get({
  path: '/channels/presence-room',
  params: { info: 'user_count,subscription_count' }
});
console.log(presenceResult);
```

**Example using Axios**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function getChannelInfo(
  appId: string,
  appKey: string,
  appSecret: string,
  channelName: string,
  info?: string[]
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/channels/${encodeURIComponent(channelName)}`;
  
  const params: Record<string, string> = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  if (info && info.length > 0) {
    params.info = info.join(',');
  }
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `GET\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.get(url);
  return response.data;
}

// Usage
const channelInfo = await getChannelInfo(
  'app-id',
  'app-key',
  'app-secret',
  'my-channel',
  ['subscription_count']
);
console.log(channelInfo);

// Presence channel
const presenceInfo = await getChannelInfo(
  'app-id',
  'app-key',
  'app-secret',
  'presence-room',
  ['user_count', 'subscription_count']
);
console.log(presenceInfo);
```

**Example using cURL**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
CHANNEL_NAME="my-channel"
TIMESTAMP=$(date +%s)

# URL encode channel name
ENCODED_CHANNEL=$(echo -n "$CHANNEL_NAME" | jq -sRr @uri)

# Create auth string
AUTH_STRING="GET
/apps/$APP_ID/channels/$ENCODED_CHANNEL
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# Make request
curl "http://localhost:6001/apps/$APP_ID/channels/$ENCODED_CHANNEL?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE"
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | Channel info retrieved successfully |
| `401` | Authentication failed |
| `404` | Channel not found or has no subscribers |

### Get Channel Users

Get list of users in a presence channel with their user information.

**Endpoint**: `GET /apps/{app_id}/channels/{channel_name}/users`

**Requirements**:

- Channel must be a presence channel (name starts with `presence-`)
- Channel must have at least one active subscriber

**Response**:

```json
{
  "users": [
    {
      "id": "user-1"
    },
    {
      "id": "user-2",
      "user_info": {
        "name": "John Doe",
        "email": "john@example.com"
      }
    },
    {
      "id": "user-3",
      "user_info": {
        "name": "Jane Smith",
        "email": "jane@example.com",
        "avatar": "https://example.com/avatar.jpg"
      }
    }
  ]
}
```

**Example using Pusher SDK**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Get users in presence channel
const result = await pusher.get({
  path: '/channels/presence-room/users'
});

console.log('Users:', result.users);
result.users.forEach((user: any) => {
  console.log(`User ${user.id}:`, user.user_info);
});
```

**Example using Axios**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function getChannelUsers(
  appId: string,
  appKey: string,
  appSecret: string,
  channelName: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/channels/${encodeURIComponent(channelName)}/users`;
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `GET\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.get(url);
  return response.data;
}

// Usage
const users = await getChannelUsers(
  'app-id',
  'app-key',
  'app-secret',
  'presence-room'
);

console.log('Online users:', users.users.length);
users.users.forEach((user: any) => {
  console.log(`- ${user.id}: ${user.user_info?.name || 'Anonymous'}`);
});
```

**Example using cURL**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
CHANNEL_NAME="presence-room"
TIMESTAMP=$(date +%s)

# URL encode channel name
ENCODED_CHANNEL=$(echo -n "$CHANNEL_NAME" | jq -sRr @uri)

# Create auth string
AUTH_STRING="GET
/apps/$APP_ID/channels/$ENCODED_CHANNEL/users
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# Make request
curl "http://localhost:6001/apps/$APP_ID/channels/$ENCODED_CHANNEL/users?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE"
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | Users retrieved successfully |
| `400` | Channel is not a presence channel |
| `401` | Authentication failed |
| `404` | Channel not found or has no subscribers |

**Notes**:

- Only works for presence channels (channels starting with `presence-`)
- Returns all unique users currently subscribed to the channel
- The `user_info` field contains the data provided during authentication
- If no `user_info` was provided during auth, only the `id` field is returned

### Terminate User Connections

Terminate all WebSocket connections for a specific user across all channels.

**Endpoint**: `POST /apps/{app_id}/users/{user_id}/terminate_connections`

**Use Cases**:

- Force logout a user from all devices
- Revoke access when user permissions change
- Security: Terminate compromised user sessions
- Administrative actions

**Response**:

```json
{}
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | Connections terminated successfully |
| `401` | Authentication failed |
| `500` | Internal server error |

**Behavior**:

1. Server identifies all connections associated with the user ID
2. Sends `pusher:error` event with code `4009` to each connection
3. Closes all WebSocket connections for that user
4. User is removed from all presence channels

**Example using Pusher SDK**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Terminate all connections for a user
await pusher.post({
  path: '/users/user-123/terminate_connections'
});

console.log('User connections terminated');
```

**Example using Axios**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function terminateUserConnections(
  appId: string,
  appKey: string,
  appSecret: string,
  userId: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/users/${encodeURIComponent(userId)}/terminate_connections`;
  const body = '';
  
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
    body_md5: bodyMd5,
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `POST\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.post(url, body, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  return response.data;
}

// Usage
await terminateUserConnections('app-id', 'app-key', 'app-secret', 'user-123');
console.log('User connections terminated');
```

**Example using cURL**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
USER_ID="user-123"
TIMESTAMP=$(date +%s)
BODY=""
BODY_MD5=$(echo -n "$BODY" | md5sum | cut -d' ' -f1)

# URL encode user ID
ENCODED_USER=$(echo -n "$USER_ID" | jq -sRr @uri)

# Create auth string
AUTH_STRING="POST
/apps/$APP_ID/users/$ENCODED_USER/terminate_connections
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&body_md5=$BODY_MD5"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# Make request
curl -X POST "http://localhost:6001/apps/$APP_ID/users/$ENCODED_USER/terminate_connections?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE&body_md5=$BODY_MD5" \
  -H "Content-Type: application/json" \
  -d "$BODY"
```

**Client-Side Handling**:

When a user's connections are terminated, they will receive an error event:

```typescript
pusher.connection.bind('error', (err: any) => {
  if (err.error && err.error.code === 4009) {
    console.log('Connection terminated by server');
    // Handle forced logout
    // Redirect to login page or show message
    window.location.href = '/login?reason=session_terminated';
  }
});
```

**Complete Example - Force Logout System**:

```typescript
// Backend: Terminate user connections
import Pusher from 'pusher';
import express from 'express';

const app = express();
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Admin endpoint to force logout a user
app.post('/admin/force-logout/:userId', async (req, res) => {
  const userId = req.params.userId;
  
  try {
    // Terminate all Pusher connections for the user
    await pusher.post({
      path: `/users/${userId}/terminate_connections`
    });
    
    // Also invalidate user sessions in your database
    await invalidateUserSessions(userId);
    
    res.json({ success: true, message: 'User logged out from all devices' });
  } catch (error) {
    console.error('Failed to terminate connections:', error);
    res.status(500).json({ error: 'Failed to logout user' });
  }
});

// Frontend: Handle forced logout
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
});

pusher.connection.bind('error', (err: any) => {
  if (err.error && err.error.code === 4009) {
    // User was forcefully logged out
    localStorage.clear();
    sessionStorage.clear();
    window.location.href = '/login?reason=forced_logout';
  }
});
```

## Authentication

Soketi uses HMAC SHA256 signatures for authentication. There are three types of authentication:

1. **HTTP API Authentication** - For backend API requests
2. **Private Channel Authentication** - For private channel subscriptions
3. **Presence Channel Authentication** - For presence channel subscriptions with user info

### HTTP API Authentication

All HTTP API requests must be authenticated using HMAC SHA256 signature.

#### Authentication Parameters

Add these query parameters to your API requests:

| Parameter | Description | Required |
|-----------|-------------|----------|
| `auth_key` | Your application key | Yes |
| `auth_timestamp` | Unix timestamp in seconds | Yes |
| `auth_version` | Authentication version (use "1.0") | Yes |
| `auth_signature` | HMAC SHA256 signature | Yes |
| `body_md5` | MD5 hash of request body | Yes (for POST) |

#### Signature Generation

The signature is generated using HMAC SHA256:

1. Create the auth string: `METHOD\nPATH\nQUERY_STRING`
2. Sign with HMAC SHA256 using your app secret
3. Add the signature to query parameters

**Example - Manual Signature Generation (Node.js)**:

```typescript
import crypto from 'crypto';

function generateAuthSignature(
  method: string,
  path: string,
  appKey: string,
  appSecret: string,
  body: string = ''
): string {
  // Generate timestamp
  const timestamp = Math.floor(Date.now() / 1000);
  
  // Generate body MD5 (for POST requests)
  const bodyMd5 = body ? crypto.createHash('md5').update(body).digest('hex') : '';
  
  // Build query parameters (sorted alphabetically)
  const params: Record<string, string> = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  if (bodyMd5) {
    params.body_md5 = bodyMd5;
  }
  
  // Sort parameters and create query string
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  // Create auth string
  const authString = `${method}\n${path}\n${sortedParams}`;
  
  // Generate signature
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  return signature;
}

// Usage example
const method = 'POST';
const path = '/apps/app-id/events';
const appKey = 'your-app-key';
const appSecret = 'your-app-secret';
const body = JSON.stringify({
  name: 'my-event',
  channel: 'my-channel',
  data: JSON.stringify({ message: 'Hello!' })
});

const signature = generateAuthSignature(method, path, appKey, appSecret, body);
console.log('Signature:', signature);
```

**Example - Using Pusher SDK (Recommended)**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// The SDK handles authentication automatically
await pusher.trigger('my-channel', 'my-event', {
  message: 'Hello World!'
});
```

#### Timestamp Validation

**Important**: Authentication timestamps must be within **600 seconds (10 minutes)** of the server time. This prevents replay attacks.

If your timestamp is outside this window, you'll receive a `401 Unauthorized` error.

**Example - Timestamp Validation**:

```typescript
// Current timestamp (valid)
const validTimestamp = Math.floor(Date.now() / 1000);

// Timestamp from 5 minutes ago (valid)
const stillValid = validTimestamp - 300;

// Timestamp from 15 minutes ago (invalid)
const expired = validTimestamp - 900; // Will be rejected
```

### Private Channel Authentication

When a client subscribes to a private channel, they must authenticate via your backend.

#### Authentication Flow

1. Client attempts to subscribe to a private channel
2. Pusher client makes a POST request to your auth endpoint
3. Your backend validates the user and generates an auth signature
4. Client receives the signature and completes subscription

**Client Request**:

The Pusher client will make a POST request to your auth endpoint (default: `/pusher/auth`):

```
POST /pusher/auth
Content-Type: application/x-www-form-urlencoded

socket_id=123.456&channel_name=private-my-channel
```

**Server Response**:

Your backend must return a signed authentication token:

```typescript
import Pusher from 'pusher';
import express from 'express';

const app = express();
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channel = req.body.channel_name;
  
  // Validate user authentication (your logic here)
  if (!req.session || !req.session.user) {
    return res.status(403).json({ error: 'Unauthorized' });
  }
  
  // Validate channel access (your logic here)
  if (!canAccessChannel(req.session.user, channel)) {
    return res.status(403).json({ error: 'Forbidden' });
  }
  
  // Generate auth signature
  const authResponse = pusher.authorizeChannel(socketId, channel);
  res.json(authResponse);
});
```

**Response Format**:

```json
{
  "auth": "app-key:signature"
}
```

**Manual Signature Generation**:

If you're not using the Pusher SDK, you can generate the signature manually:

```typescript
import crypto from 'crypto';

function generateChannelAuth(
  appKey: string,
  appSecret: string,
  socketId: string,
  channelName: string
): string {
  // Create string to sign: socket_id:channel_name
  const toSign = `${socketId}:${channelName}`;
  
  // Generate HMAC SHA256 signature
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(toSign)
    .digest('hex');
  
  // Return in format: app_key:signature
  return `${appKey}:${signature}`;
}

// Usage
const auth = generateChannelAuth(
  'app-key',
  'app-secret',
  '123.456',
  'private-my-channel'
);

// Response
res.json({ auth });
```

### Presence Channel Authentication

Presence channels require additional user information to track online users.

**Server Response**:

```typescript
import Pusher from 'pusher';

app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channel = req.body.channel_name;
  
  // Validate user authentication
  if (!req.session || !req.session.user) {
    return res.status(403).json({ error: 'Unauthorized' });
  }
  
  // For presence channels, provide user data
  if (channel.startsWith('presence-')) {
    const presenceData = {
      user_id: req.session.user.id,
      user_info: {
        name: req.session.user.name,
        email: req.session.user.email,
        avatar: req.session.user.avatar,
      },
    };
    
    const authResponse = pusher.authorizeChannel(socketId, channel, presenceData);
    return res.json(authResponse);
  }
  
  // For private channels, no user data needed
  const authResponse = pusher.authorizeChannel(socketId, channel);
  res.json(authResponse);
});
```

**Response Format**:

```json
{
  "auth": "app-key:signature",
  "channel_data": "{\"user_id\":\"123\",\"user_info\":{\"name\":\"John\",\"email\":\"john@example.com\"}}"
}
```

**Manual Signature Generation for Presence Channels**:

```typescript
import crypto from 'crypto';

function generatePresenceAuth(
  appKey: string,
  appSecret: string,
  socketId: string,
  channelName: string,
  userId: string,
  userInfo: Record<string, any>
): { auth: string; channel_data: string } {
  // Create channel data
  const channelData = JSON.stringify({
    user_id: userId,
    user_info: userInfo,
  });
  
  // Create string to sign: socket_id:channel_name:channel_data
  const toSign = `${socketId}:${channelName}:${channelData}`;
  
  // Generate HMAC SHA256 signature
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(toSign)
    .digest('hex');
  
  return {
    auth: `${appKey}:${signature}`,
    channel_data: channelData,
  };
}

// Usage
const authResponse = generatePresenceAuth(
  'app-key',
  'app-secret',
  '123.456',
  'presence-room',
  'user-123',
  { name: 'John Doe', email: 'john@example.com' }
);

res.json(authResponse);
```

### User Authentication (pusher:signin)

For user-specific features, you can authenticate users with the `pusher:signin` event.

**Client Side**:

```typescript
// After connection is established
pusher.signin();
```

**Server Side**:

```typescript
app.post('/pusher/user-auth', (req, res) => {
  const socketId = req.body.socket_id;
  
  // Validate user session
  if (!req.session || !req.session.user) {
    return res.status(403).json({ error: 'Unauthorized' });
  }
  
  const userData = JSON.stringify({
    id: req.session.user.id,
    name: req.session.user.name,
    email: req.session.user.email,
  });
  
  // Generate user auth signature
  const toSign = `${socketId}::user::${userData}`;
  const signature = crypto
    .createHmac('sha256', 'app-secret')
    .update(toSign)
    .digest('hex');
  
  res.json({
    auth: `app-key:${signature}`,
    user_data: userData,
  });
});
```

### Authentication Error Handling

**Common Authentication Errors**:

| Error | Description | Solution |
|-------|-------------|----------|
| `401 Unauthorized` | Invalid signature or expired timestamp | Verify your app secret and timestamp |
| `403 Forbidden` | User not authorized for channel | Check your authorization logic |
| `4008` (WebSocket) | Connection is unauthorized | Verify app key and credentials |

**Example - Error Handling**:

```typescript
// Client side
const channel = pusher.subscribe('private-my-channel');

channel.bind('pusher:subscription_error', (status: any) => {
  if (status === 403) {
    console.error('Access denied to channel');
  } else if (status === 401) {
    console.error('Authentication failed');
  }
});

// Server side
app.post('/pusher/auth', (req, res) => {
  try {
    const socketId = req.body.socket_id;
    const channel = req.body.channel_name;
    
    // Your validation logic
    if (!isValidUser(req)) {
      return res.status(403).json({ 
        error: 'Forbidden',
        message: 'User not authorized for this channel'
      });
    }
    
    const authResponse = pusher.authorizeChannel(socketId, channel);
    res.json(authResponse);
  } catch (error) {
    console.error('Auth error:', error);
    res.status(500).json({ 
      error: 'Internal Server Error',
      message: 'Failed to generate auth signature'
    });
  }
});
```

## Channel Types

### Public Channels

- No authentication required
- Anyone can subscribe
- Channel name doesn't start with `private-` or `presence-`

### Private Channels

- Require authentication
- Channel name starts with `private-`
- Used for user-specific or sensitive data

### Presence Channels

- Require authentication with user info
- Channel name starts with `presence-`
- Track online users
- Provide member added/removed events

## Events

### System Events

| Event | Description |
|-------|-------------|
| `pusher:connection_established` | Connection successful |
| `pusher:error` | Error occurred |
| `pusher:subscription_succeeded` | Channel subscription successful |
| `pusher:subscription_error` | Channel subscription failed |
| `pusher:member_added` | User joined presence channel |
| `pusher:member_removed` | User left presence channel |

### Custom Events

You can trigger custom events with any name (except those starting with `pusher:` or `pusher_internal:`).

## Error Codes

| Code | Description |
|------|-------------|
| 4000 | Application does not exist |
| 4001 | Application disabled |
| 4003 | Application over connection quota |
| 4004 | Path not found |
| 4005 | Invalid version string format |
| 4006 | Unsupported protocol version |
| 4007 | No protocol version supplied |
| 4008 | Connection is unauthorized |
| 4009 | Connection limit exceeded |
| 4100 | Over capacity |
| 4200 | Generic error |
| 4201 | Pong reply not received in time |
| 4202 | Closed after inactivity |

## Rate Limits

Rate limits are configured per application:

- `max_connections`: Maximum concurrent connections
- `max_backend_events_per_second`: Backend event rate limit
- `max_client_events_per_second`: Client event rate limit
- `max_read_requests_per_second`: Read request rate limit

When rate limits are exceeded, requests will receive a 429 status code.

## Health Check and Monitoring Endpoints

Soketi provides several endpoints for health checks, monitoring, and observability.

### Health Check

Basic health check endpoint to verify the server is running.

**Endpoint**: `GET /`

**Authentication**: None required

**Response**:

```json
{
  "ok": true
}
```

**Example**:

```bash
curl http://localhost:6001/
```

**Use Case**: Basic uptime monitoring

### Readiness Check

Check if the server is ready to accept connections.

**Endpoint**: `GET /ready`

**Authentication**: None required

**Response (Ready)**:

```json
{
  "ready": true
}
```

**Response (Not Ready)**:

```json
{
  "ready": false,
  "reason": "server is shutting down"
}
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | Server is ready |
| `503` | Server is not ready (shutting down) |

**Example**:

```bash
curl http://localhost:6001/ready
```

**Use Case**: Kubernetes readiness probes, load balancer health checks

### Accept Traffic Check

Check if the server can accept traffic based on memory usage.

**Endpoint**: `GET /accept-traffic`

**Authentication**: None required

**Response (Can Accept)**:

```json
{
  "accept_traffic": true,
  "used_memory_mb": 512,
  "threshold_mb": 1024
}
```

**Response (Cannot Accept)**:

```json
{
  "accept_traffic": false,
  "reason": "memory threshold exceeded",
  "used_memory_mb": 1100,
  "threshold_mb": 1024
}
```

**Status Codes**:

| Code | Description |
|------|-------------|
| `200` | Server can accept traffic |
| `503` | Server cannot accept traffic (memory threshold exceeded or shutting down) |

**Example**:

```bash
curl http://localhost:6001/accept-traffic
```

**Configuration**:

The memory threshold can be configured in your server configuration:

```json
{
  "http_api": {
    "accept_traffic_memory_threshold_mb": 1024
  }
}
```

**Use Case**: Load balancer health checks with memory awareness

### Memory Usage

Get detailed memory usage information.

**Endpoint**: `GET /usage`

**Authentication**: None required

**Response**:

```json
{
  "memory": {
    "total_mb": 2048,
    "used_mb": 512,
    "free_mb": 1536,
    "available_mb": 1600,
    "usage_percent": 25.0
  }
}
```

**Example**:

```bash
curl http://localhost:6001/usage
```

**Use Case**: Monitoring dashboards, capacity planning

### Metrics (Prometheus)

Get Prometheus-compatible metrics for monitoring.

**Endpoint**: `GET /metrics`

**Authentication**: None required

**Query Parameters**:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `format` | Response format: `text` or `json` | `text` |

**Response (Plaintext - Prometheus format)**:

```
# HELP soketi_connections_total Total number of WebSocket connections
# TYPE soketi_connections_total gauge
soketi_connections_total 42

# HELP soketi_messages_sent_total Total number of messages sent
# TYPE soketi_messages_sent_total counter
soketi_messages_sent_total 1234

# HELP soketi_messages_received_total Total number of messages received
# TYPE soketi_messages_received_total counter
soketi_messages_received_total 5678
```

**Response (JSON format)**:

```json
{
  "connections_total": 42,
  "messages_sent_total": 1234,
  "messages_received_total": 5678,
  "channels_total": 15,
  "http_requests_total": 890
}
```

**Example - Plaintext (Prometheus)**:

```bash
curl http://localhost:6001/metrics
```

**Example - JSON**:

```bash
curl "http://localhost:6001/metrics?format=json"
```

**Prometheus Configuration**:

Add this to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['localhost:6001']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

**Use Case**: Prometheus monitoring, Grafana dashboards

**Note**: Metrics endpoint is only available if metrics are enabled in the server configuration.

### Health Check Examples

**Docker Health Check**:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:6001/ready || exit 1
```

**Kubernetes Probes**:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: soketi
spec:
  containers:
  - name: soketi
    image: soketi:latest
    ports:
    - containerPort: 6001
    livenessProbe:
      httpGet:
        path: /
        port: 6001
      initialDelaySeconds: 5
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: 6001
      initialDelaySeconds: 5
      periodSeconds: 5
```

**Load Balancer Health Check (Nginx)**:

```nginx
upstream soketi_backend {
    server soketi:6001 max_fails=3 fail_timeout=30s;
    
    # Health check
    check interval=5000 rise=2 fall=3 timeout=3000 type=http;
    check_http_send "GET /accept-traffic HTTP/1.0\r\n\r\n";
    check_http_expect_alive http_2xx;
}
```

## Next Steps

- **[Getting Started](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md)** - Quick start guide
- **[Examples](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/basic-chat.md)** - Code examples
- **[Configuration](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md)** - Configuration reference

## Related Resources

- [Pusher Protocol Documentation](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol)
- [Pusher JavaScript Client](https://github.com/pusher/pusher-js)
- [Pusher Server SDKs](https://pusher.com/docs/channels/channels_libraries/libraries)
