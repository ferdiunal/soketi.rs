# Troubleshooting Guide

> Comprehensive guide to diagnosing and resolving common issues with soketi.rs WebSocket server

## Table of Contents

- [Connection Issues](#connection-issues)
- [Authentication Problems](#authentication-problems)
- [Channel Subscription Errors](#channel-subscription-errors)
- [Performance Issues](#performance-issues)
- [Deployment Problems](#deployment-problems)
- [Configuration Errors](#configuration-errors)
- [Debug Techniques](#debug-techniques)
- [Monitoring and Logging](#monitoring-and-logging)
- [Common Error Messages](#common-error-messages)
- [Getting Help](#getting-help)

## Connection Issues

### WebSocket Connection Failed

**Symptom:**
```
Error: WebSocket connection to 'ws://localhost:6001/app/...' failed
```

**Possible Causes:**
1. soketi.rs server is not running
2. Incorrect host or port configuration
3. Firewall blocking WebSocket connections
4. SSL/TLS configuration mismatch

**Solutions:**

**1. Verify Server is Running:**
```bash
# Check if soketi.rs is running
curl http://localhost:6001/health

# Check Docker container status
docker ps | grep soketi

# Check process
ps aux | grep soketi
```

**2. Verify Configuration:**
```typescript
// Check your Pusher client configuration
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',  // Should match your server host
  wsPort: 6001,         // Should match your server port
  forceTLS: false,      // Set to true if using HTTPS
  encrypted: false,     // Set to true if using HTTPS
  enabledTransports: ['ws', 'wss'],
});
```

**3. Check Firewall Rules:**
```bash
# Linux - Check if port is open
sudo netstat -tulpn | grep 6001

# Allow port through firewall (Ubuntu/Debian)
sudo ufw allow 6001/tcp

# Check if port is accessible
telnet localhost 6001
```

**4. SSL/TLS Mismatch:**
```typescript
// If your server uses HTTPS, ensure client matches
const pusher = new Pusher('app-key', {
  wsHost: 'your-domain.com',
  wsPort: 443,
  wssPort: 443,
  forceTLS: true,      // Must be true for HTTPS
  encrypted: true,     // Must be true for HTTPS
  enabledTransports: ['wss'],  // Use 'wss' for secure connections
});
```

### Connection Drops Frequently

**Symptom:**
Client disconnects and reconnects repeatedly.

**Possible Causes:**
1. Network instability
2. Timeout settings too aggressive
3. Load balancer timeout issues
4. Server resource constraints

**Solutions:**

**1. Adjust Timeout Settings:**
```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  activityTimeout: 120000,  // Increase to 120 seconds
  pongTimeout: 30000,       // Increase to 30 seconds
});
```

**2. Configure Load Balancer Timeouts:**
```nginx
# Nginx configuration
location /app/ {
    proxy_pass http://soketi_backend;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    
    # Increase timeouts
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

**3. Monitor Server Resources:**
```bash
# Check CPU and memory usage
top

# Check soketi.rs resource usage
docker stats soketi

# Check system logs
journalctl -u soketi -f
```

### Cannot Connect from Browser

**Symptom:**
Connection works from server/backend but fails from browser.

**Possible Causes:**
1. CORS configuration issues
2. Mixed content (HTTP page trying to connect to WS)
3. Browser security policies

**Solutions:**

**1. Configure CORS:**
```bash
# Environment variable
SOKETI_CORS_ALLOWED_ORIGINS=https://your-domain.com,https://app.your-domain.com

# Or in config.json
{
  "cors": {
    "credentials": true,
    "origin": ["https://your-domain.com"],
    "methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
  }
}
```

**2. Fix Mixed Content:**
```typescript
// If your page is HTTPS, use WSS
const pusher = new Pusher('app-key', {
  wsHost: 'your-domain.com',
  forceTLS: true,  // Force secure connection
  encrypted: true,
});
```

**3. Check Browser Console:**
```javascript
// Enable Pusher debug logging
Pusher.logToConsole = true;

const pusher = new Pusher('app-key', {
  // ... config
});
```

## Authentication Problems

### Private Channel Authorization Failed

**Symptom:**
```
Error: Unable to retrieve auth string from auth endpoint
```

**Possible Causes:**
1. Auth endpoint not configured or incorrect
2. Auth endpoint returning wrong format
3. Session/token not valid
4. CORS issues with auth endpoint

**Solutions:**

**1. Verify Auth Endpoint Configuration:**
```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  authEndpoint: '/api/pusher/auth',  // Must be correct path
  auth: {
    headers: {
      'Authorization': `Bearer ${token}`  // Include auth token
    }
  }
});
```

**2. Check Auth Endpoint Response:**
```typescript
// Correct auth endpoint response format
export async function POST(req: Request) {
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  // For private channels
  const authSignature = pusher.authorizeChannel(socketId, channelName);
  
  // Must return this exact format
  return Response.json({
    auth: authSignature.auth
  });
}
```

**3. Test Auth Endpoint Manually:**
```bash
# Test your auth endpoint
curl -X POST http://localhost:3000/api/pusher/auth \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -H "Authorization: Bearer your-token" \
  -d "socket_id=123.456&channel_name=private-test"

# Expected response:
# {"auth":"app-key:signature"}
```

**4. Enable CORS for Auth Endpoint:**
```typescript
// app/api/pusher/auth/route.ts
export async function OPTIONS(req: Request) {
  return new Response(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  });
}
```

### Presence Channel User Info Not Showing

**Symptom:**
User successfully subscribes to presence channel but user info is missing.

**Possible Causes:**
1. Auth endpoint not returning user_info
2. Incorrect presence data format

**Solutions:**

**1. Return Correct Presence Data:**
```typescript
// app/api/pusher/auth/route.ts
export async function POST(req: Request) {
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  if (channelName?.startsWith('presence-')) {
    // Must include presenceData for presence channels
    const presenceData = {
      user_id: session.user.id,
      user_info: {
        name: session.user.name,
        email: session.user.email,
        avatar: session.user.avatar,
      }
    };
    
    const authResponse = pusher.authorizeChannel(
      socketId,
      channelName,
      presenceData  // This is required!
    );
    
    return Response.json(authResponse);
  }
}
```

**2. Verify Client-Side Handling:**
```typescript
const channel = pusher.subscribe('presence-chat');

channel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Members:', members.count);
  members.each((member: any) => {
    console.log('Member ID:', member.id);
    console.log('Member Info:', member.info);  // Should contain user_info
  });
});
```

### Invalid Signature Error

**Symptom:**
```
Error: Invalid signature
```

**Possible Causes:**
1. App secret mismatch between client and server
2. Incorrect signature generation
3. Clock skew between servers

**Solutions:**

**1. Verify App Credentials:**
```bash
# Server-side (soketi.rs)
SOKETI_DEFAULT_APP_ID=app-id
SOKETI_DEFAULT_APP_KEY=app-key
SOKETI_DEFAULT_APP_SECRET=app-secret

# Client-side
NEXT_PUBLIC_PUSHER_KEY=app-key  # Must match server

# Server SDK (for auth endpoint)
PUSHER_APP_ID=app-id
PUSHER_SECRET=app-secret  # Must match server
```

**2. Check Pusher Server Configuration:**
```typescript
// Ensure server SDK uses correct credentials
const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID!,
  key: process.env.NEXT_PUBLIC_PUSHER_KEY!,
  secret: process.env.PUSHER_SECRET!,  // Must match soketi.rs secret
  host: process.env.PUSHER_HOST,
  port: process.env.PUSHER_PORT,
  useTLS: true,
});
```

**3. Synchronize Server Clocks:**
```bash
# Install NTP
sudo apt-get install ntp

# Sync time
sudo ntpdate -s time.nist.gov

# Enable NTP service
sudo systemctl enable ntp
sudo systemctl start ntp
```

## Channel Subscription Errors

### Subscription Timeout

**Symptom:**
```
Error: Subscription timeout
```

**Possible Causes:**
1. Server not responding to subscription request
2. Network latency too high
3. Server overloaded

**Solutions:**

**1. Increase Subscription Timeout:**
```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  activityTimeout: 120000,  // 120 seconds
});
```

**2. Check Server Logs:**
```bash
# Docker logs
docker logs soketi -f

# Check for subscription errors
docker logs soketi 2>&1 | grep -i "subscription"
```

**3. Monitor Server Load:**
```bash
# Check if server is overloaded
docker stats soketi

# Scale up if needed
docker-compose up --scale soketi=3
```

### Client Events Not Working

**Symptom:**
Client events (client-*) are not being received by other clients.

**Possible Causes:**
1. Client events not enabled in app configuration
2. Using client events on public channels (not allowed)
3. Event name doesn't start with "client-"

**Solutions:**

**1. Enable Client Events:**
```json
// config.json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "enable_client_messages": true  // Must be true
        }
      ]
    }
  }
}
```

**2. Use Private or Presence Channels:**
```typescript
// Client events only work on private/presence channels
const channel = pusher.subscribe('private-chat');  // ✅ Works
// const channel = pusher.subscribe('public-chat');  // ❌ Won't work

channel.bind('pusher:subscription_succeeded', () => {
  // Trigger client event (must start with 'client-')
  channel.trigger('client-typing', { user: 'John', isTyping: true });
});

// Listen for client events
channel.bind('client-typing', (data) => {
  console.log('User typing:', data);
});
```

**3. Check Event Name Format:**
```typescript
// ✅ Correct - starts with 'client-'
channel.trigger('client-message', data);
channel.trigger('client-typing', data);

// ❌ Wrong - doesn't start with 'client-'
channel.trigger('message', data);  // Won't work as client event
```

### Max Connections Exceeded

**Symptom:**
```
Error: Connection limit exceeded
```

**Possible Causes:**
1. Too many concurrent connections
2. Connection limit set too low
3. Connections not being properly closed

**Solutions:**

**1. Increase Connection Limit:**
```json
// config.json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "max_connections": 1000  // Increase limit
        }
      ]
    }
  }
}
```

**2. Properly Disconnect Clients:**
```typescript
// Disconnect when component unmounts
useEffect(() => {
  const pusher = new Pusher('app-key', { /* config */ });
  const channel = pusher.subscribe('my-channel');

  return () => {
    channel.unbind_all();
    pusher.unsubscribe('my-channel');
    pusher.disconnect();  // Important!
  };
}, []);
```

**3. Monitor Active Connections:**
```bash
# Check metrics endpoint
curl http://localhost:9601/metrics

# Look for connection count
# soketi_connections_total
```

## Performance Issues

### High Latency

**Symptom:**
Messages take several seconds to arrive.

**Possible Causes:**
1. Network latency
2. Server overloaded
3. Inefficient event handling
4. Geographic distance

**Solutions:**

**1. Deploy Closer to Users:**
```bash
# Use CDN or deploy in multiple regions
# Choose region closest to your users
```

**2. Optimize Event Handling:**
```typescript
// ❌ Bad - Processing in event handler
channel.bind('message', (data) => {
  // Heavy processing here blocks other events
  processHeavyData(data);
});

// ✅ Good - Offload to worker
channel.bind('message', (data) => {
  // Queue for background processing
  queueForProcessing(data);
});
```

**3. Use Connection Pooling:**
```typescript
// Reuse single Pusher instance
const pusher = new Pusher('app-key', { /* config */ });

// Don't create new instance for each component
// ❌ Bad
function Component1() {
  const pusher = new Pusher('app-key', { /* config */ });
}

// ✅ Good - Import shared instance
import { pusher } from '@/lib/pusher';
```

**4. Enable Compression:**
```nginx
# Nginx configuration
gzip on;
gzip_vary on;
gzip_proxied any;
gzip_comp_level 6;
gzip_types text/plain application/json;
```

### High Memory Usage

**Symptom:**
soketi.rs consuming excessive memory.

**Possible Causes:**
1. Too many concurrent connections
2. Large message payloads
3. Memory leak
4. Insufficient garbage collection

**Solutions:**

**1. Limit Message Size:**
```json
// config.json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "max_event_payload_in_kb": 100  // Limit to 100KB
        }
      ]
    }
  }
}
```

**2. Monitor Memory Usage:**
```bash
# Check memory usage
docker stats soketi

# Set memory limits
docker run -m 512m soketi
```

**3. Restart Periodically:**
```bash
# Use Docker restart policy
docker run --restart=unless-stopped soketi

# Or use systemd timer for periodic restarts
```

**4. Scale Horizontally:**
```yaml
# docker-compose.yml
version: '3.8'
services:
  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    deploy:
      replicas: 3  # Run multiple instances
      resources:
        limits:
          memory: 512M
```

### Slow Event Broadcasting

**Symptom:**
Events take long time to broadcast to all subscribers.

**Possible Causes:**
1. Single-threaded bottleneck
2. No horizontal scaling
3. Inefficient adapter

**Solutions:**

**1. Use Redis Adapter:**
```bash
# Enable Redis for horizontal scaling
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=redis
SOKETI_REDIS_PORT=6379
SOKETI_REDIS_DB=0
```

**2. Deploy Multiple Instances:**
```yaml
# docker-compose.yml
version: '3.8'
services:
  redis:
    image: redis:alpine
    
  soketi-1:
    image: quay.io/soketi/soketi:latest-16-alpine
    environment:
      - SOKETI_ADAPTER_DRIVER=redis
      - SOKETI_REDIS_HOST=redis
      
  soketi-2:
    image: quay.io/soketi/soketi:latest-16-alpine
    environment:
      - SOKETI_ADAPTER_DRIVER=redis
      - SOKETI_REDIS_HOST=redis
      
  nginx:
    image: nginx:alpine
    ports:
      - "6001:6001"
    depends_on:
      - soketi-1
      - soketi-2
```

**3. Optimize Event Payload:**
```typescript
// ❌ Bad - Large payload
channel.trigger('update', {
  fullDocument: largeObject,
  metadata: moreData,
  history: evenMoreData
});

// ✅ Good - Minimal payload
channel.trigger('update', {
  id: documentId,
  type: 'modified'
});
// Client fetches full data if needed
```

## Deployment Problems

### Docker Container Won't Start

**Symptom:**
```
Error: Container exits immediately after starting
```

**Possible Causes:**
1. Configuration error
2. Port already in use
3. Missing environment variables
4. Invalid credentials

**Solutions:**

**1. Check Container Logs:**
```bash
# View logs
docker logs soketi

# Follow logs in real-time
docker logs -f soketi
```

**2. Check Port Availability:**
```bash
# Check if port is already in use
sudo lsof -i :6001

# Kill process using the port
sudo kill -9 <PID>

# Or use different port
docker run -p 6002:6001 soketi
```

**3. Verify Environment Variables:**
```bash
# Check required variables are set
docker run --rm soketi env | grep SOKETI

# Run with explicit variables
docker run \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  -p 6001:6001 \
  quay.io/soketi/soketi:latest-16-alpine
```

**4. Test Configuration:**
```bash
# Test with minimal config
docker run -p 6001:6001 \
  -e SOKETI_DEFAULT_APP_ID=test \
  -e SOKETI_DEFAULT_APP_KEY=test \
  -e SOKETI_DEFAULT_APP_SECRET=test \
  quay.io/soketi/soketi:latest-16-alpine
```

### Reverse Proxy Not Working

**Symptom:**
Cannot connect through Nginx/Caddy reverse proxy.

**Possible Causes:**
1. WebSocket upgrade headers missing
2. Incorrect proxy configuration
3. SSL/TLS issues
4. Timeout too short

**Solutions:**

**1. Verify Nginx Configuration:**
```nginx
location /app/ {
    proxy_pass http://soketi:6001;
    proxy_http_version 1.1;
    
    # Required for WebSocket
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    
    # Standard headers
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    
    # Timeouts
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

**2. Test Nginx Configuration:**
```bash
# Test configuration syntax
nginx -t

# Reload configuration
nginx -s reload

# Check error logs
tail -f /var/log/nginx/error.log
```

**3. Verify Caddy Configuration:**
```caddyfile
your-domain.com {
    reverse_proxy /app/* {
        to soketi:6001
        
        # WebSocket headers
        header_up Host {host}
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
    }
}
```

**4. Test Reverse Proxy:**
```bash
# Test WebSocket connection through proxy
wscat -c wss://your-domain.com/app/app-key

# Check proxy logs
docker logs nginx -f
docker logs caddy -f
```

### SSL Certificate Issues

**Symptom:**
```
Error: SSL certificate problem: unable to get local issuer certificate
```

**Possible Causes:**
1. Self-signed certificate
2. Certificate chain incomplete
3. Certificate expired
4. Wrong certificate path

**Solutions:**

**1. Use Let's Encrypt:**
```bash
# With Caddy (automatic)
# Caddy handles SSL automatically

# With Certbot (Nginx)
sudo certbot --nginx -d your-domain.com
```

**2. Verify Certificate:**
```bash
# Check certificate expiration
openssl x509 -in /path/to/cert.pem -noout -dates

# Check certificate chain
openssl s_client -connect your-domain.com:443 -showcerts

# Test SSL configuration
curl -vI https://your-domain.com
```

**3. Configure Certificate Path:**
```bash
# soketi.rs with SSL
SOKETI_SSL_CERT=/etc/ssl/certs/cert.pem
SOKETI_SSL_KEY=/etc/ssl/private/key.pem
SOKETI_SSL_PASSPHRASE=your-passphrase  # If encrypted
```

**4. Allow Self-Signed Certificates (Development Only):**
```typescript
// ⚠️ Only for development!
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  forceTLS: true,
  // Node.js only
  // process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
});
```

## Configuration Errors

### Invalid Configuration File

**Symptom:**
```
Error: Failed to parse configuration file
```

**Possible Causes:**
1. Invalid JSON syntax
2. Missing required fields
3. Wrong data types

**Solutions:**

**1. Validate JSON:**
```bash
# Validate JSON syntax
cat config.json | jq .

# Or use online validator
# https://jsonlint.com/
```

**2. Check Required Fields:**
```json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",           // Required
          "key": "app-key",         // Required
          "secret": "app-secret",   // Required
          "enabled": true           // Required
        }
      ]
    }
  }
}
```

**3. Use Environment Variables Instead:**
```bash
# Simpler alternative to config file
docker run \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  -p 6001:6001 \
  quay.io/soketi/soketi:latest-16-alpine
```

### App Credentials Not Working

**Symptom:**
```
Error: Invalid app credentials
```

**Possible Causes:**
1. Mismatched credentials between client and server
2. Typo in credentials
3. Using wrong app ID

**Solutions:**

**1. Verify All Credentials Match:**
```bash
# Server (soketi.rs)
SOKETI_DEFAULT_APP_ID=my-app
SOKETI_DEFAULT_APP_KEY=my-key
SOKETI_DEFAULT_APP_SECRET=my-secret

# Client
NEXT_PUBLIC_PUSHER_KEY=my-key  # Must match SOKETI_DEFAULT_APP_KEY

# Server SDK (auth endpoint)
PUSHER_APP_ID=my-app           # Must match SOKETI_DEFAULT_APP_ID
PUSHER_SECRET=my-secret        # Must match SOKETI_DEFAULT_APP_SECRET
```

**2. Test Credentials:**
```bash
# Test connection with credentials
curl -X POST http://localhost:6001/apps/my-app/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-event",
    "channel": "test-channel",
    "data": "{\"message\": \"test\"}"
  }'
```

**3. Check for Whitespace:**
```bash
# Trim whitespace from credentials
export SOKETI_DEFAULT_APP_KEY=$(echo "my-key" | tr -d '[:space:]')
```

## Debug Techniques

### Enable Debug Logging

**Client-Side:**
```typescript
// Enable Pusher debug logging
Pusher.logToConsole = true;

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  // ... other config
});

// You'll see detailed logs in browser console:
// Pusher : State changed : connecting -> connected
// Pusher : Event sent : {"event":"pusher:subscribe","data":{"channel":"my-channel"}}
```

**Server-Side:**
```bash
# Enable debug mode
SOKETI_DEBUG=true
SOKETI_LOG_LEVEL=debug

# Or in config.json
{
  "debug": true,
  "log_level": "debug"
}
```

### Monitor WebSocket Traffic

**Using Browser DevTools:**
```
1. Open Chrome DevTools (F12)
2. Go to Network tab
3. Filter by "WS" (WebSocket)
4. Click on connection to see frames
5. View sent/received messages
```

**Using wscat:**
```bash
# Install wscat
npm install -g wscat

# Connect to soketi.rs
wscat -c ws://localhost:6001/app/app-key

# Send test message
> {"event":"pusher:subscribe","data":{"channel":"test"}}

# View responses
< {"event":"pusher:connection_established","data":"..."}
```

**Using tcpdump:**
```bash
# Capture WebSocket traffic
sudo tcpdump -i any -A 'tcp port 6001'

# Save to file for analysis
sudo tcpdump -i any -w websocket.pcap 'tcp port 6001'
```

### Test Event Flow

**1. Test Server to Client:**
```bash
# Trigger event via HTTP API
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-event",
    "channel": "test-channel",
    "data": "{\"message\": \"Hello\"}"
  }'

# Client should receive the event
```

**2. Test Client to Server:**
```typescript
// Subscribe to channel
const channel = pusher.subscribe('test-channel');

// Bind to event
channel.bind('test-event', (data) => {
  console.log('Received:', data);
});

// Verify subscription succeeded
channel.bind('pusher:subscription_succeeded', () => {
  console.log('✅ Subscribed successfully');
});

channel.bind('pusher:subscription_error', (error) => {
  console.error('❌ Subscription failed:', error);
});
```

**3. Test Client Events:**
```typescript
// On private/presence channel
const channel = pusher.subscribe('private-test');

channel.bind('pusher:subscription_succeeded', () => {
  // Trigger client event
  channel.trigger('client-test', { message: 'Hello' });
});

// Listen for client event
channel.bind('client-test', (data) => {
  console.log('Client event received:', data);
});
```

### Inspect Connection State

```typescript
// Monitor all connection state changes
pusher.connection.bind('state_change', (states) => {
  console.log(`State: ${states.previous} → ${states.current}`);
});

// Possible states:
// - initialized
// - connecting
// - connected
// - unavailable
// - failed
// - disconnected

// Check current state
console.log('Current state:', pusher.connection.state);

// Get socket ID (available when connected)
console.log('Socket ID:', pusher.connection.socket_id);
```

### Trace Event Handlers

```typescript
// Log all events on a channel
const channel = pusher.subscribe('my-channel');

// Bind to all events
channel.bind_global((eventName, data) => {
  console.log(`Event: ${eventName}`, data);
});

// List all bound events
console.log('Bound events:', channel.callbacks._callbacks);
```

## Monitoring and Logging

### Metrics Endpoint

soketi.rs provides a Prometheus-compatible metrics endpoint:

```bash
# Access metrics
curl http://localhost:9601/metrics

# Key metrics to monitor:
# - soketi_connections_total: Total active connections
# - soketi_messages_sent_total: Total messages sent
# - soketi_messages_received_total: Total messages received
# - soketi_channels_total: Total active channels
```

### Health Check Endpoint

```bash
# Check server health
curl http://localhost:6001/health

# Expected response:
# {"status":"ok"}

# Use in monitoring systems
# Kubernetes liveness probe:
livenessProbe:
  httpGet:
    path: /health
    port: 6001
  initialDelaySeconds: 10
  periodSeconds: 30
```

### Structured Logging

**Configure Log Format:**
```bash
# JSON logging for easier parsing
SOKETI_LOG_FORMAT=json

# Log to file
SOKETI_LOG_FILE=/var/log/soketi/soketi.log
```

**Parse Logs:**
```bash
# Filter error logs
cat soketi.log | jq 'select(.level == "error")'

# Count events by type
cat soketi.log | jq -r '.event' | sort | uniq -c

# Monitor in real-time
tail -f soketi.log | jq .
```

### External Monitoring

**Integrate with Monitoring Services:**

**1. Datadog:**
```yaml
# docker-compose.yml
services:
  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    labels:
      com.datadoghq.ad.logs: '[{"source": "soketi", "service": "websocket"}]'
```

**2. Prometheus:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['soketi:9601']
```

**3. Grafana Dashboard:**
```bash
# Import soketi.rs dashboard
# Use metrics from Prometheus
# Visualize connections, messages, latency
```

**4. Sentry (Error Tracking):**
```typescript
// In your Next.js app
import * as Sentry from '@sentry/nextjs';

pusher.connection.bind('error', (err) => {
  Sentry.captureException(err);
});
```

## Common Error Messages

### "Connection refused"

**Meaning:** Cannot connect to soketi.rs server.

**Solutions:**
- Verify server is running: `curl http://localhost:6001/health`
- Check host and port configuration
- Verify firewall rules

### "Invalid app key"

**Meaning:** Client using wrong app key.

**Solutions:**
- Verify `NEXT_PUBLIC_PUSHER_KEY` matches `SOKETI_DEFAULT_APP_KEY`
- Check for typos or whitespace
- Ensure environment variables are loaded

### "Subscription error: 403"

**Meaning:** Authorization failed for private/presence channel.

**Solutions:**
- Check auth endpoint is configured correctly
- Verify auth endpoint returns correct format
- Ensure user is authenticated
- Check CORS configuration

### "Max connections exceeded"

**Meaning:** Too many concurrent connections.

**Solutions:**
- Increase `max_connections` in config
- Scale horizontally with multiple instances
- Implement connection pooling

### "Event payload too large"

**Meaning:** Message exceeds size limit.

**Solutions:**
- Reduce message size
- Increase `max_event_payload_in_kb` in config
- Split large messages into smaller chunks

### "Channel not found"

**Meaning:** Trying to trigger event on non-existent channel.

**Solutions:**
- Ensure channel is subscribed before triggering
- Check channel name spelling
- Verify channel exists in metrics

### "Rate limit exceeded"

**Meaning:** Too many requests in short time.

**Solutions:**
- Implement client-side rate limiting
- Increase rate limits in config:
  ```json
  {
    "max_backend_events_per_second": 1000,
    "max_client_events_per_second": 100
  }
  ```
- Use batching for multiple events

### "WebSocket upgrade failed"

**Meaning:** Server cannot upgrade HTTP to WebSocket.

**Solutions:**
- Check reverse proxy WebSocket configuration
- Verify `Upgrade` and `Connection` headers
- Ensure HTTP/1.1 is used

## Getting Help

### Before Asking for Help

1. **Check this troubleshooting guide** for your specific issue
2. **Enable debug logging** and review logs
3. **Test with minimal configuration** to isolate the problem
4. **Verify credentials** are correct and consistent
5. **Check server status** and resource usage

### Information to Include

When asking for help, provide:

1. **soketi.rs version:**
   ```bash
   docker run quay.io/soketi/soketi:latest-16-alpine --version
   ```

2. **Configuration:**
   ```bash
   # Sanitize secrets before sharing!
   cat config.json | jq 'del(.app_manager.array.apps[].secret)'
   ```

3. **Error messages:**
   ```bash
   # Full error logs
   docker logs soketi 2>&1 | tail -50
   ```

4. **Client configuration:**
   ```typescript
   // Share your Pusher client setup (remove secrets!)
   const pusher = new Pusher('app-key', {
     wsHost: 'localhost',
     wsPort: 6001,
     // ...
   });
   ```

5. **Environment:**
   - Operating system
   - Docker version (if using Docker)
   - Node.js version (for client)
   - Browser version (if browser issue)

### Community Resources

**GitHub Issues:**
- Search existing issues: [soketi.rs Issues](https://github.com/soketi/soketi.rs/issues)
- Create new issue with template
- Include all relevant information

**Discord/Slack:**
- Join community chat
- Ask questions in appropriate channels
- Help others with similar issues

**Stack Overflow:**
- Tag questions with `soketi` and `websocket`
- Search existing questions first
- Provide minimal reproducible example

### Professional Support

For production deployments and enterprise support:
- Contact soketi.rs team
- Consider managed hosting solutions
- Hire WebSocket consultants

## Quick Reference

### Essential Commands

```bash
# Check server health
curl http://localhost:6001/health

# View metrics
curl http://localhost:9601/metrics

# Test WebSocket connection
wscat -c ws://localhost:6001/app/app-key

# Trigger test event
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{"name":"test","channel":"test","data":"{}"}'

# View Docker logs
docker logs soketi -f

# Check container stats
docker stats soketi

# Restart container
docker restart soketi
```

### Debug Checklist

- [ ] Server is running (`curl http://localhost:6001/health`)
- [ ] Credentials match between client and server
- [ ] Port is accessible (check firewall)
- [ ] WebSocket upgrade headers configured (if using proxy)
- [ ] SSL/TLS configuration matches (forceTLS, encrypted)
- [ ] Auth endpoint configured correctly (for private/presence)
- [ ] CORS configured for browser clients
- [ ] Debug logging enabled
- [ ] Metrics show expected values
- [ ] No errors in server logs

### Performance Checklist

- [ ] Using Redis adapter for horizontal scaling
- [ ] Multiple soketi.rs instances deployed
- [ ] Load balancer configured with proper timeouts
- [ ] Connection pooling implemented
- [ ] Event payloads optimized (< 100KB)
- [ ] Rate limits configured appropriately
- [ ] Compression enabled
- [ ] Monitoring and alerting set up
- [ ] Resource limits configured
- [ ] Geographic distribution considered

## Related Resources

- [Getting Started Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md)
- [Configuration Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md)
- [API Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/api-reference.md)
- [Deployment Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/deployment/reverse-proxy.md)
- [Vercel Deployment](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/deployment/vercel.md)
- [Netlify Deployment](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/deployment/netlify.md)
- [Examples](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/basic-chat.md)

---

**Need more help?** Check the [soketi.rs documentation](https://docs.soketi.app) or [open an issue](https://github.com/soketi/soketi.rs/issues) on GitHub.


### Config File Not Loading

**Symptom:**
Server starts but no apps are loaded, or "App not found" errors.

**Possible Causes:**
1. Config file not specified with `--config-file` parameter
2. Wrong parameter name used (`--config` instead of `--config-file`)
3. Config file path incorrect

**Solutions:**

**1. Use Correct Parameter:**
```bash
# ✅ Correct
cargo run -- --config-file config.json
./soketi --config-file config.json

# ❌ Wrong
cargo run -- --config config.json  # Will fail
./soketi --config config.json      # Will fail
```

**2. Verify Config File Path:**
```bash
# Check file exists
ls -la config.json

# Use absolute path if needed
./soketi --config-file /path/to/config.json

# Check file permissions
chmod 644 config.json
```

**3. Enable Debug Mode to See Config Loading:**
```bash
# Run with debug to see if apps are loaded
./soketi --config-file config.json --debug true

# Look for log line:
# DEBUG soketi_rs::server: Loading ArrayAppManager with X apps
```

**4. Verify Config File Format:**
```json
{
  "app_manager": {
    "driver": "Array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "enabled": true
        }
      ]
    }
  }
}
```

### Messages Not Reaching Clients

**Symptom:**
HTTP API accepts messages but clients don't receive them. Logs show "No sockets found in channel".

**Possible Causes:**
1. Sockets not being added to adapter
2. Channel subscription not completing
3. Socket ID mismatch

**Solutions:**

**1. Verify Socket Registration:**
```bash
# Enable debug mode to see socket operations
./soketi --config-file config.json --debug true

# Look for these log lines:
# DEBUG soketi_rs::ws: Socket added to adapter: <socket_id>
# DEBUG soketi_rs::ws: Successfully subscribed to channel: <channel_name>
```

**2. Check Subscription Flow:**
```typescript
// Client-side - verify subscription succeeds
const channel = pusher.subscribe('my-channel');

channel.bind('pusher:subscription_succeeded', () => {
  console.log('✅ Subscription successful');
});

channel.bind('pusher:subscription_error', (error) => {
  console.error('❌ Subscription failed:', error);
});
```

**3. Test End-to-End:**
```bash
# 1. Start server with debug
./soketi --config-file config.json --debug true

# 2. Connect client and subscribe to channel

# 3. Send test message
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test",
    "channel": "my-channel",
    "data": "{\"message\": \"test\"}"
  }'

# 4. Check debug logs for:
# DEBUG soketi_rs::adapters::local: Found X sockets in channel: my-channel
# DEBUG soketi_rs::adapters::local: Sending to socket: <socket_id>
```

### Private Channel Auth Fails But Presence Works

**Symptom:**
Presence channels authenticate successfully but private channels fail with "Invalid signature".

**Possible Causes:**
1. Backend sending `channel_data` for private channels
2. Signature calculation includes wrong data
3. Auth endpoint not differentiating channel types

**Solutions:**

**1. Fix Auth Endpoint to Handle Channel Types:**
```typescript
// Backend auth endpoint
export async function POST(req: Request) {
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  // ✅ Correct - Different handling for different channel types
  if (channelName?.startsWith('presence-')) {
    // Presence channel - include user data
    const authResponse = pusher.authenticate(socketId, channelName, {
      user_id: userId,
      user_info: { name: userName }
    });
    return Response.json(authResponse);
  } else if (channelName?.startsWith('private-')) {
    // Private channel - NO user data
    const authResponse = pusher.authenticate(socketId, channelName);
    return Response.json(authResponse);
  }
  
  // ❌ Wrong - Same handling for all channels
  // const authResponse = pusher.authenticate(socketId, channelName, {
  //   user_id: userId,  // Don't include for private channels!
  //   user_info: { name: userName }
  // });
}
```

**2. Verify Auth Response Format:**
```bash
# Test private channel auth
curl -X POST http://localhost:4000/pusher/auth \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "socket_id=123.456&channel_name=private-test"

# Expected response for private channel:
# {"auth":"app-key:signature"}
# Should NOT include channel_data

# Test presence channel auth
curl -X POST http://localhost:4000/pusher/auth \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "socket_id=123.456&channel_name=presence-test"

# Expected response for presence channel:
# {"auth":"app-key:signature","channel_data":"{\"user_id\":\"...\",\"user_info\":{...}}"}
```

**3. Check Pusher.js Library Version:**
```bash
# Ensure using compatible version
npm list pusher-js

# Update if needed
npm install pusher-js@latest
```

### Debug Mode Not Showing Logs

**Symptom:**
Debug mode enabled but no debug logs appear.

**Possible Causes:**
1. Wrong debug flag format
2. Log level not set correctly
3. Logs being filtered

**Solutions:**

**1. Use Correct Debug Flag:**
```bash
# ✅ Correct
./soketi --config-file config.json --debug true

# ❌ Wrong
./soketi --config-file config.json --debug  # Missing value
./soketi --config-file config.json -d       # Wrong flag
```

**2. Check Log Output:**
```bash
# Ensure not redirecting stderr
./soketi --config-file config.json --debug true 2>&1

# Or save to file
./soketi --config-file config.json --debug true 2>&1 | tee soketi.log
```

**3. Verify Debug Logs Format:**
```
# Debug logs should look like:
2026-01-25T16:25:19.341777Z DEBUG soketi_rs::server: src/server.rs:191: Loading ArrayAppManager with 1 apps
2026-01-25T16:25:23.740581Z DEBUG soketi_rs::ws: src/ws.rs:34: Found app by key 'app-key': app_id = 'app-id'
```

## Recent Fixes and Known Issues

### Fixed in Latest Version

**1. Socket Registration Issue (v0.1.0)**
- **Problem:** Sockets were added to channels but not to the adapter's socket map
- **Symptom:** Messages broadcast but not received by clients
- **Fix:** Added `adapter.add_socket()` call after socket establishment
- **Commit:** Added in ws.rs socket handler

**2. Config File Loading (v0.1.0)**
- **Problem:** Config file not loaded without explicit parameter
- **Symptom:** Server starts with no apps configured
- **Fix:** Must use `--config-file` parameter
- **Documentation:** Updated getting-started.md

**3. Private Channel Auth (v0.1.0)**
- **Problem:** Backend sending channel_data for all channel types
- **Symptom:** Private channels fail auth, presence channels work
- **Fix:** Differentiate channel types in auth endpoint
- **Example:** See demo-chat/backend/server.js

### Known Limitations

**1. Client Events**
- Only work on private and presence channels
- Must start with "client-" prefix
- Require `enable_client_messages: true` in app config

**2. Presence Channels**
- Member list not automatically synced on reconnect
- Large member lists may impact performance
- Consider pagination for channels with many members

**3. Rate Limiting**
- Per-app rate limits only
- No per-user rate limiting yet
- Consider implementing in your application layer

### Reporting New Issues

When reporting issues:

1. **Check if already fixed** in latest version
2. **Search existing issues** on GitHub
3. **Provide minimal reproduction** case
4. **Include debug logs** with `--debug true`
5. **Specify versions** of all components

**Issue Template:**
```markdown
## Description
Brief description of the issue

## Steps to Reproduce
1. Start server with: `./soketi --config-file config.json`
2. Connect client with: `pusher.subscribe('channel')`
3. Trigger event: `curl -X POST ...`

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- soketi.rs version: 0.1.0
- OS: macOS 14.0
- Client: pusher-js 8.4.0
- Browser: Chrome 120

## Debug Logs
```
Paste relevant debug logs here
```

## Configuration
```json
{
  "app_manager": {
    // Sanitized config
  }
}
```
```

