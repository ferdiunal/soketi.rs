# Soketi.rs - High-Performance WebSocket Server

![Soketi.rs Cover](art/cover_en.png)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/ferdiunal/soketi-rs/docker-publish.yml?branch=main)](https://github.com/ferdiunal/soketi-rs/actions)
[![Version](https://img.shields.io/github/v/release/ferdiunal/soketi-rs)](https://github.com/ferdiunal/soketi-rs/releases)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/docker/pulls/ferdiunal/soketi-rs)](https://hub.docker.com/r/funal/soketi-rs)
[![Docker Image Size](https://img.shields.io/docker/image-size/ferdiunal/soketi-rs/latest)](https://hub.docker.com/r/funal/soketi-rs)

A high-performance, Pusher-compatible WebSocket server written in Rust. Soketi.rs provides real-time messaging capabilities with support for public, private, and presence channels.

[🇹🇷 Türkçe Dokümantasyon](README.md)

## ✨ Features

- **🚀 High Performance**: Built with Rust for maximum speed and efficiency
- **📡 Pusher Protocol**: 100% compatible with Pusher client libraries
- **🔐 Authentication**: Built-in support for private and presence channels
- **👥 Presence Channels**: Real-time user tracking and member lists
- **💬 Client Events**: Direct client-to-client messaging
- **📊 Metrics**: Prometheus metrics for monitoring
- **🔄 Horizontal Scaling**: Redis adapter for multi-server deployments
- **🗄️ Multiple Backends**: Support for MySQL, PostgreSQL, DynamoDB
- **🎯 Rate Limiting**: Configurable rate limits per app
- **🪝 Webhooks**: Event notifications via HTTP callbacks
- **🐳 Docker Ready**: Production-ready Docker images

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage Examples](#usage-examples)
- [Docker Deployment](#docker-deployment)
- [Documentation](#documentation)
- [API Documentation](#api-documentation)
- [Client Libraries](#client-libraries)
- [Architecture](#architecture)
- [Performance](#performance)
- [Contributing](#contributing)
- [License](#license)

## 🚀 Quick Start

### Using Docker (Fastest)

```bash
# Pull and run the latest image from Docker Hub
docker pull funal/soketi-rs:latest

docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest
```

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f soketi

# Access the demo
open http://localhost:3000
```

### Using Cargo

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo build --release
./target/release/soketi-rs --config-file config.json
```

> **Note:** Use `--config-file` parameter to specify the configuration file path.

## 📦 Installation

### Prerequisites

- **Rust 1.75+** (for building from source)
- **Docker & Docker Compose** (for containerized deployment)
- **Redis** (optional, for clustering)
- **PostgreSQL/MySQL/DynamoDB** (optional, for app management)

### From Source

```bash
# Clone repository
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs

# Build release binary
cargo build --release

# Run tests
cargo test

# Install binary
cargo install --path .
```

### Using Docker

```bash
# Pull the image
docker pull funal/soketi-rs:latest

# Run container
docker run -d \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  funal/soketi-rs:latest
```

## ⚙️ Configuration

### Basic Configuration

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

### Environment Variables

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

### Advanced Configuration

See the [Configuration Guide](docs/en/configuration.md) for detailed configuration options including:
- App management (Array, MySQL, PostgreSQL, DynamoDB)
- Adapters (Local, Redis, NATS, Cluster)
- Rate limiting and webhooks
- Metrics and monitoring
- SSL/TLS configuration

## � Usage Examples

### Basic Chat Application

```javascript
// Initialize Pusher client
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false
});

// Subscribe to a public channel
const channel = pusher.subscribe('chat-room');

// Listen for messages
channel.bind('new-message', (data) => {
  console.log('New message:', data.message);
  displayMessage(data);
});

// Send a message (requires client events enabled)
channel.trigger('client-new-message', {
  user: 'John',
  message: 'Hello everyone!'
});
```

### Private Channels with Authentication

```javascript
// Subscribe to a private channel
const privateChannel = pusher.subscribe('private-user-123');

// Server-side authentication endpoint
app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channel = req.body.channel_name;
  
  // Verify user has access to this channel
  if (userCanAccessChannel(req.user, channel)) {
    const auth = pusher.authenticate(socketId, channel);
    res.send(auth);
  } else {
    res.status(403).send('Forbidden');
  }
});
```

### Presence Channels

```javascript
// Subscribe to a presence channel
const presenceChannel = pusher.subscribe('presence-team');

// Member added
presenceChannel.bind('pusher:member_added', (member) => {
  console.log('User joined:', member.info.name);
  updateUserList();
});

// Member removed
presenceChannel.bind('pusher:member_removed', (member) => {
  console.log('User left:', member.info.name);
  updateUserList();
});

// Get current members
presenceChannel.bind('pusher:subscription_succeeded', (members) => {
  members.each((member) => {
    console.log('Online:', member.info.name);
  });
});
```

### Triggering Events from Server

```javascript
// Using Node.js Pusher library
const Pusher = require('pusher');

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false
});

// Trigger an event
pusher.trigger('my-channel', 'my-event', {
  message: 'Hello from server!'
});

// Trigger to multiple channels
pusher.trigger(['channel-1', 'channel-2'], 'my-event', {
  message: 'Broadcast message'
});
```

For more examples, see:
- [Basic Chat Example](docs/en/examples/basic-chat.md)
- [Authentication Examples](docs/en/examples/authentication.md)
- [Private Channels](docs/en/examples/private-channels.md)
- [Presence Channels](docs/en/examples/presence.md)

## 📚 Documentation

Comprehensive documentation is available in multiple languages:

### English Documentation
- [Getting Started](docs/en/getting-started.md) - Quick start guide and basic concepts
- [Installation](docs/en/installation.md) - Detailed installation instructions
- [Configuration](docs/en/configuration.md) - Complete configuration reference
- [API Reference](docs/en/api-reference.md) - HTTP and WebSocket API documentation
- [Troubleshooting](docs/en/troubleshooting.md) - Common issues and solutions

#### Deployment Guides
- [Vercel Deployment](docs/en/deployment/vercel.md) - Deploy to Vercel
- [Netlify Deployment](docs/en/deployment/netlify.md) - Deploy to Netlify
- [Reverse Proxy Setup](docs/en/deployment/reverse-proxy.md) - Caddy and Nginx configuration with HTTP/2 and HTTP/3

#### Code Examples
- [Basic Chat](docs/en/examples/basic-chat.md) - Simple chat application
- [Authentication](docs/en/examples/authentication.md) - User authentication patterns
- [Private Channels](docs/en/examples/private-channels.md) - Secure private messaging
- [Presence Channels](docs/en/examples/presence.md) - Real-time user presence

### Turkish Documentation (Türkçe Dokümantasyon)
- [Başlangıç](docs/tr/baslangic.md) - Hızlı başlangıç kılavuzu
- [Kurulum](docs/tr/kurulum.md) - Detaylı kurulum talimatları
- [Yapılandırma](docs/tr/yapilandirma.md) - Yapılandırma referansı
- [API Referansı](docs/tr/api-referans.md) - HTTP ve WebSocket API dokümantasyonu
- [Sorun Giderme](docs/tr/sorun-giderme.md) - Yaygın sorunlar ve çözümler

#### Deployment Kılavuzları
- [Vercel Deployment](docs/tr/deployment/vercel.md) - Vercel'e deployment
- [Netlify Deployment](docs/tr/deployment/netlify.md) - Netlify'a deployment
- [Reverse Proxy Kurulumu](docs/tr/deployment/reverse-proxy.md) - Caddy ve Nginx yapılandırması

#### Kod Örnekleri
- [Temel Chat](docs/tr/ornekler/temel-chat.md) - Basit chat uygulaması
- [Kimlik Doğrulama](docs/tr/ornekler/kimlik-dogrulama.md) - Kullanıcı kimlik doğrulama
- [Özel Kanallar](docs/tr/ornekler/ozel-kanallar.md) - Güvenli özel mesajlaşma
- [Presence](docs/tr/ornekler/presence.md) - Gerçek zamanlı kullanıcı takibi

### Advanced Topics
- [MySQL Setup](docs/MYSQL_SETUP.md) - MySQL app manager configuration
- [PostgreSQL Setup](docs/POSTGRES_SETUP.md) - PostgreSQL app manager configuration
- [DynamoDB Setup](docs/DYNAMODB_SETUP.md) - DynamoDB app manager configuration
- [Redis Adapter](docs/REDIS_ADAPTER_IMPLEMENTATION.md) - Redis clustering setup
- [NATS Adapter](docs/NATS_ADAPTER_IMPLEMENTATION.md) - NATS messaging integration
- [Cluster Adapter](docs/CLUSTER_ADAPTER_IMPLEMENTATION.md) - Native clustering
- [Lambda Webhooks](docs/LAMBDA_WEBHOOKS.md) - AWS Lambda webhook integration
- [SQS Queue Manager](docs/sqs_queue_manager.md) - AWS SQS queue configuration

## 🐳 Docker Deployment

All deployment files are organized in the `deployment/` directory:

```
deployment/
├── docker/    # Standard Docker deployment
├── nginx/     # Nginx reverse proxy setup
└── caddy/     # Caddy reverse proxy setup (automatic HTTPS)
```

### Standard Docker Deployment

```bash
cd deployment/docker

# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Production Deployment with Nginx

```bash
cd deployment/nginx

# Configure SSL certificates
cp .env.nginx.example .env
# Edit .env file

# Start services
docker-compose -f docker-compose.nginx.yml up -d
```

### Production Deployment with Caddy (Automatic HTTPS)

```bash
cd deployment/caddy

# Configure domain
cp .env.caddy.example .env
# Edit .env file

# Start services (automatic SSL!)
docker-compose -f docker-compose.caddy.yml up -d
```

### Scaling

```bash
# Scale soketi instances
docker-compose up -d --scale soketi=3

# View service status
docker-compose ps
```

### With Monitoring Stack

```bash
# Start with Prometheus and Grafana
docker-compose --profile monitoring up -d

# Access Grafana
open http://localhost:3001
# Default credentials: admin/admin
```

### Production Best Practices

1. **Use Redis for clustering**:
   ```json
   {
     "adapter": {
       "driver": "Redis",
       "redis": {
         "host": "redis",
         "port": 6379
       }
     }
   }
   ```

2. **Enable metrics**:
   ```json
   {
     "metrics": {
       "enabled": true,
       "driver": "Prometheus",
       "port": 9601
     }
   }
   ```

3. **Configure resource limits** in `docker-compose.yml`

4. **Use health checks** for automatic recovery

5. **Set up log aggregation** (ELK, Loki, etc.)

## 📚 API Documentation

### WebSocket Connection

```javascript
// Using Pusher.js
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false
});

// Subscribe to channel
const channel = pusher.subscribe('my-channel');

// Listen for events
channel.bind('my-event', (data) => {
  console.log('Received:', data);
});
```

### HTTP API

#### Trigger Event

```bash
POST /apps/{app_id}/events
Content-Type: application/json

{
  "name": "my-event",
  "channel": "my-channel",
  "data": "{\"message\":\"Hello World\"}"
}
```

#### Get Channels

```bash
GET /apps/{app_id}/channels
```

#### Get Channel Info

```bash
GET /apps/{app_id}/channels/{channel_name}
```

See the [API Reference](docs/en/api-reference.md) for complete API documentation.

## 📱 Client Libraries

Soketi.rs is compatible with all Pusher client libraries:

- **JavaScript**: [pusher-js](https://github.com/pusher/pusher-js)
- **Laravel**: [Laravel Echo](https://github.com/laravel/echo)
- **iOS**: [pusher-websocket-swift](https://github.com/pusher/pusher-websocket-swift)
- **Android**: [pusher-websocket-java](https://github.com/pusher/pusher-websocket-java)
- **Python**: [pusher-http-python](https://github.com/pusher/pusher-http-python)
- **PHP**: [pusher-http-php](https://github.com/pusher/pusher-http-php)
- **Ruby**: [pusher-http-ruby](https://github.com/pusher/pusher-http-ruby)
- **Go**: [pusher-http-go](https://github.com/pusher/pusher-http-go)

## 🏗️ Architecture

```
┌─────────────────┐
│   Client Apps   │
│  (Web/Mobile)   │
└────────┬────────┘
         │ WebSocket
         ▼
┌─────────────────┐
│  Soketi Server  │
│     (Rust)      │
└────────┬────────┘
         │
    ┌────┴────┬────────┬─────────┐
    ▼         ▼        ▼         ▼
┌───────┐ ┌───────┐ ┌──────┐ ┌────────┐
│ Redis │ │  DB   │ │Metrics│ │Webhooks│
└───────┘ └───────┘ └──────┘ └────────┘
```

### Components

- **WebSocket Server**: Handles client connections
- **HTTP API**: REST API for triggering events
- **Adapter**: Message distribution (Local/Redis/NATS/Cluster)
- **App Manager**: Application configuration (Array/MySQL/PostgreSQL/DynamoDB)
- **Cache Manager**: Caching layer (Memory/Redis)
- **Rate Limiter**: Request rate limiting (Local/Redis)
- **Queue Manager**: Webhook queue (Sync/Redis/SQS)
- **Metrics**: Prometheus metrics exporter

## 📊 Performance

### Benchmarks

- **Connections**: 100,000+ concurrent connections per instance
- **Messages**: 1M+ messages per second
- **Latency**: <1ms average message latency
- **Memory**: ~50MB base + ~1KB per connection
- **CPU**: Efficient multi-core utilization

### Optimization Tips

1. Use Redis adapter for horizontal scaling
2. Enable connection pooling
3. Configure appropriate rate limits
4. Use SSD storage for persistence
5. Enable Prometheus metrics for monitoring

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test

# Run benchmarks
cargo bench
```

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run demo
cargo run -- --config-file test-config.json
```

## 📄 License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [Soketi](https://github.com/soketi/soketi)
- Built with [Tokio](https://tokio.rs/)
- Compatible with [Pusher Protocol](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol)

## 📞 Support

- **Documentation**: [English](docs/en/getting-started.md) | [Türkçe](docs/tr/baslangic.md)
- **Issues**: [GitHub Issues](https://github.com/ferdiunal/soketi-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ferdiunal/soketi-rs/discussions)
- **Discord**: [Join our Discord](https://discord.gg/soketi)

---

Made with ❤️ by [Ferdi ÜNAL](https://github.com/ferdiunal)
