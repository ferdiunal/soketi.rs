# Soketi.rs - High-Performance WebSocket Server

[![GitHub](https://img.shields.io/badge/GitHub-soketi.rs-blue?logo=github)](https://github.com/ferdiunal/soketi.rs)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](https://github.com/ferdiunal/soketi.rs/blob/main/LICENSE)

A high-performance, Pusher-compatible WebSocket server written in Rust. Soketi.rs provides real-time messaging capabilities with support for public, private, and presence channels.

## Quick Start

```bash
# Pull and run the latest image
docker pull funal/soketi-rs:latest

# Run with default configuration
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest

# Run with custom configuration
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  funal/soketi-rs:latest
```

## Features

- 🚀 **High Performance** - Built with Rust for maximum speed and efficiency
- 📡 **Pusher Compatible** - 100% compatible with Pusher client libraries
- 🔐 **Authentication** - Built-in support for private and presence channels
- 👥 **Presence Channels** - Real-time user tracking and member lists
- 💬 **Client Events** - Direct client-to-client messaging
- 📊 **Metrics** - Prometheus metrics for monitoring
- 🔄 **Horizontal Scaling** - Redis adapter for multi-server deployments
- 🗄️ **Multiple Backends** - Support for MySQL, PostgreSQL, DynamoDB
- 🎯 **Rate Limiting** - Configurable rate limits per app
- 🪝 **Webhooks** - Event notifications via HTTP callbacks

## Environment Variables

```bash
# Server configuration
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_DEBUG=false

# App configuration
APP_ID=your-app-id
APP_KEY=your-app-key
APP_SECRET=your-app-secret

# Redis (for clustering)
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_DB=0

# Metrics
METRICS_ENABLED=true
METRICS_PORT=9601
```

## Docker Compose Example

```yaml
version: '3.8'

services:
  soketi:
    image: funal/soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    environment:
      - SOKETI_HOST=0.0.0.0
      - SOKETI_PORT=6001
      - APP_ID=app-id
      - APP_KEY=app-key
      - APP_SECRET=app-secret
    volumes:
      - ./config.json:/app/config.json
    restart: unless-stopped

  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    restart: unless-stopped
```

## Basic Configuration

Create a `config.json` file:

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "debug": false,
  "app_manager": {
    "driver": "Array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "enabled": true,
          "enable_client_messages": true,
          "max_connections": 10000
        }
      ]
    }
  }
}
```

## Client Usage

```javascript
// Initialize Pusher client
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false
});

// Subscribe to a channel
const channel = pusher.subscribe('chat-room');

// Listen for messages
channel.bind('new-message', (data) => {
  console.log('New message:', data.message);
});
```

## Production Deployment

### With Redis Clustering

```yaml
version: '3.8'

services:
  soketi:
    image: funal/soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
    volumes:
      - ./config.production.json:/app/config.json
    depends_on:
      - redis
    deploy:
      replicas: 3
    restart: unless-stopped

  redis:
    image: redis:alpine
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - soketi
    restart: unless-stopped
```

### Scaling

```bash
# Scale soketi instances
docker-compose up -d --scale soketi=3

# View service status
docker-compose ps
```

## Health Check

```bash
# Check if server is running
curl http://localhost:6001/

# Check metrics
curl http://localhost:9601/metrics
```

## Ports

- **6001** - WebSocket server (default)
- **9601** - Prometheus metrics endpoint

## Supported Architectures

- `linux/amd64`
- `linux/arm64`

## Performance

- **Connections**: 100,000+ concurrent connections per instance
- **Messages**: 1M+ messages per second
- **Latency**: <1ms average message latency
- **Memory**: ~50MB base + ~1KB per connection

## Documentation

- **GitHub Repository**: [https://github.com/ferdiunal/soketi.rs](https://github.com/ferdiunal/soketi.rs)
- **Getting Started**: [English](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md) | [Türkçe](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/baslangic.md)
- **Configuration Guide**: [Documentation](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md)
- **API Reference**: [Documentation](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/api-reference.md)
- **Deployment Guides**: [Docker](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/docker-deployment.md)

## Client Libraries

Compatible with all Pusher client libraries:

- JavaScript: [pusher-js](https://github.com/pusher/pusher-js)
- Laravel: [Laravel Echo](https://github.com/laravel/echo)
- iOS: [pusher-websocket-swift](https://github.com/pusher/pusher-websocket-swift)
- Android: [pusher-websocket-java](https://github.com/pusher/pusher-websocket-java)
- Python: [pusher-http-python](https://github.com/pusher/pusher-http-python)
- PHP: [pusher-http-php](https://github.com/pusher/pusher-http-php)
- Ruby: [pusher-http-ruby](https://github.com/pusher/pusher-http-ruby)
- Go: [pusher-http-go](https://github.com/pusher/pusher-http-go)

## Support

- **Issues**: [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ferdiunal/soketi.rs/discussions)

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](https://github.com/ferdiunal/soketi.rs/blob/main/LICENSE) file for details.

---

Made with ❤️ by [Ferdi ÜNAL](https://github.com/ferdiunal)
