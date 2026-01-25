# Standard Docker Deployment

This directory contains the standard Docker deployment configuration for Soketi.rs.

## Quick Start

```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

## Using Pre-built Image

The easiest way to deploy is using the pre-built image from Docker Hub:

```bash
docker pull funal/soketi-rs:latest
docker run -d -p 6001:6001 -p 9601:9601 funal/soketi-rs:latest
```

## Building Locally

```bash
# From repository root
docker build -f deployment/docker/Dockerfile -t soketi-rs:latest .

# Or use docker-compose
cd deployment/docker
docker-compose build

# Run the container
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json:ro \
  soketi-rs:latest
```

> **Note:** The Dockerfile must be run from the repository root with `-f deployment/docker/Dockerfile` flag.

## Configuration

### Quick Start (Development)

Use the example configuration:

```bash
cp config.example.json config.json
```

Edit `config.json` and update your app credentials:
- `app-id` → Your application ID
- `app-key` → Your application key
- `app-secret` → Your application secret

### Production Configuration

For production deployments, use the production template:

```bash
cp config.production.json config.json
```

This includes:
- PostgreSQL for app management
- Redis for clustering, caching, and rate limiting
- Optimized settings for production

**Important**: Update these values in `config.json`:
- Database credentials
- Redis connection details
- CORS origins
- SSL certificates (if using HTTPS)

## Ports

- **6001** - WebSocket server
- **9601** - Metrics endpoint

## Environment Variables

```bash
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_METRICS_PORT=9601
```

## Documentation

- [Docker Guide (EN)](../../docs/en/docker-guide.md)
- [Docker Deployment (EN)](../../docs/en/docker-deployment.md)
- [Configuration (EN)](../../docs/en/configuration.md)

## Docker Hub

**Repository**: [ferdiunal/soketi-rs](https://hub.docker.com/r/funal/soketi-rs)

Available tags:
- `latest` - Latest stable release
- `v1.x.x` - Specific version
- `main` - Latest development build

## Support

- [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)
- [GitHub Discussions](https://github.com/ferdiunal/soketi.rs/discussions)
