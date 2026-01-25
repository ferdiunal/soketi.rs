# Docker Quick Start Guide

## Quick Commands

### Start the Server
```bash
docker-compose up -d
```

### View Logs
```bash
docker-compose logs -f soketi
```

### Stop the Server
```bash
docker-compose down
```

### Rebuild
```bash
docker-compose build --no-cache
docker-compose up -d
```

## Available Services

| Service | Port | Description |
|---------|------|-------------|
| Soketi Server | 6001 | WebSocket server |
| Metrics | 9601 | Prometheus metrics |
| Redis | 6379 | Cache & clustering |

## Using Docker Hub Image

### Pull and Run
```bash
# Pull the latest image
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

### Using Docker Compose

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
```

## Configuration

Create a `config.json` file:

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

## Environment Variables

Create a `.env` file:

```env
# Soketi
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_METRICS_PORT=9601

# App Configuration
APP_ID=your-app-id
APP_KEY=your-app-key
APP_SECRET=your-app-secret

# Redis (optional)
REDIS_HOST=redis
REDIS_PORT=6379
```

## Health Checks

```bash
# Check server status
curl http://localhost:6001/health

# Check metrics
curl http://localhost:9601/metrics
```

## Troubleshooting

### Port Already in Use
```bash
# Find process using the port
lsof -i :6001

# Stop the process
kill -9 <PID>
```

### Container Won't Start
```bash
# View logs
docker-compose logs soketi

# Restart container
docker-compose restart soketi
```

### Reset Everything
```bash
docker-compose down -v
docker-compose up -d
```

## Supported Architectures

- `linux/amd64`
- `linux/arm64`

## Available Tags

- `latest` - Latest stable release
- `v1.x.x` - Specific version
- `main` - Latest development build

## Docker Hub

**Repository**: [ferdiunal/soketi-rs](https://hub.docker.com/r/funal/soketi-rs)

---

For detailed documentation, see [Getting Started](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md) and [Configuration](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md).
