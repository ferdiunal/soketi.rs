# Docker Deployment Guide

## Overview

Soketi.rs is available as a Docker image on Docker Hub, making deployment simple and straightforward.

**Docker Hub Repository**: [ferdiunal/soketi-rs](https://hub.docker.com/r/funal/soketi-rs)

## Quick Start

### Pull and Run

```bash
# Pull the latest image
docker pull funal/soketi-rs:latest

# Run the server
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest
```

### Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  soketi:
    image: funal/soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    volumes:
      - ./config.json:/app/config.json
    restart: unless-stopped
    
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    restart: unless-stopped
```

Start the services:

```bash
docker-compose up -d
```

## Configuration

### Using config.json

Create a `config.json` file in your project directory:

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "metrics_port": 9601,
  "apps": [
    {
      "id": "app-id",
      "key": "app-key",
      "secret": "app-secret",
      "max_connections": 100,
      "enable_client_messages": true,
      "enabled": true
    }
  ]
}
```

Mount it to the container:

```bash
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  funal/soketi-rs:latest
```

### Using Environment Variables

```bash
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -e SOKETI_HOST=0.0.0.0 \
  -e SOKETI_PORT=6001 \
  -e SOKETI_METRICS_PORT=9601 \
  funal/soketi-rs:latest
```

## Available Tags

- `latest` - Latest stable release
- `v1.x.x` - Specific semantic version
- `main` - Latest development build

## Supported Platforms

- `linux/amd64` - x86_64 architecture
- `linux/arm64` - ARM64 architecture (Apple Silicon, AWS Graviton, etc.)

## Ports

- **6001** - WebSocket server (default)
- **9601** - Metrics endpoint (Prometheus compatible)

## Features

- ✅ Pusher protocol compatible
- ✅ WebSocket support
- ✅ Public, private, and presence channels
- ✅ Client events
- ✅ Webhooks
- ✅ Prometheus metrics
- ✅ Multiple adapters (Local, Redis, NATS, Cluster)
- ✅ High performance
- ✅ Low memory footprint

## Production Deployment

### With Redis Adapter

```yaml
version: '3.8'

services:
  soketi:
    image: funal/soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    volumes:
      - ./config.json:/app/config.json
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
    depends_on:
      - redis
    restart: unless-stopped
    
  redis:
    image: redis:alpine
    restart: unless-stopped
```

### Scaling

Scale horizontally with multiple instances:

```bash
docker-compose up -d --scale soketi=3
```

### Health Checks

```bash
# Check server status
curl http://localhost:6001/health

# Check metrics
curl http://localhost:9601/metrics
```

## Monitoring

### Prometheus Integration

Add Prometheus to your `docker-compose.yml`:

```yaml
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    restart: unless-stopped
```

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['soketi:9601']
```

## Troubleshooting

### View Logs

```bash
# Docker
docker logs soketi

# Docker Compose
docker-compose logs -f soketi
```

### Restart Container

```bash
# Docker
docker restart soketi

# Docker Compose
docker-compose restart soketi
```

### Reset Everything

```bash
docker-compose down -v
docker-compose up -d
```

## CI/CD and Automated Builds

### GitHub Actions Workflow

The project uses GitHub Actions for automated Docker image builds and deployments. Images are automatically built and pushed to Docker Hub when:

- **Push to main branch** - Creates `main` tag
- **Version tags** - Creates versioned tags (e.g., `v1.0.0`, `1.0`, `1`)
- **Pull requests** - Builds only (no push)

### Multi-Platform Builds

Images are built for multiple architectures:
- `linux/amd64` - Standard x86_64 servers
- `linux/arm64` - ARM-based servers (AWS Graviton, Apple Silicon, etc.)

### Build Optimization

- **Multi-stage builds** - Smaller final images (~100-200MB)
- **Build cache** - Faster subsequent builds
- **Layer optimization** - Efficient Docker layer caching

### Automated README Sync

The Docker Hub README is automatically updated from the repository on each release.

## For Contributors

### Building Locally

```bash
# Build for your platform
docker build -t soketi-rs .

# Build for multiple platforms
docker buildx build --platform linux/amd64,linux/arm64 -t soketi-rs .
```

### Publishing to Docker Hub

The CI/CD pipeline handles this automatically, but for manual publishing:

```bash
# Login to Docker Hub
docker login

# Build and push
docker buildx build --platform linux/amd64,linux/arm64 \
  -t funal/soketi-rs:latest \
  --push .
```

### Creating a Release

```bash
# Tag a new version
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will automatically:
# 1. Build the Docker image
# 2. Push to Docker Hub with version tags
# 3. Update Docker Hub README
```

## Support

- **GitHub Issues**: [Report a bug](https://github.com/ferdiunal/soketi.rs/issues)
- **GitHub Discussions**: [Ask questions](https://github.com/ferdiunal/soketi.rs/discussions)
- **Documentation**: [Full docs](https://github.com/ferdiunal/soketi.rs/tree/main/docs)

## License

MIT License - see [LICENSE](https://github.com/ferdiunal/soketi.rs/blob/main/LICENSE) for details.
