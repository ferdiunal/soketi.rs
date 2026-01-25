# Configuration Guide

This guide explains the configuration files in the deployment directories.

## Configuration Files

Each deployment directory contains two configuration templates:

### config.example.json

Basic configuration for development and testing:
- In-memory app management (Array driver)
- Local adapter (single instance)
- Memory cache
- No external dependencies

**Use for:**
- Local development
- Testing
- Quick demos

### config.production.json

Production-ready configuration:
- PostgreSQL for app management
- Redis for clustering and caching
- Optimized for multiple instances
- Rate limiting enabled

**Use for:**
- Production deployments
- Staging environments
- Multi-instance setups

## Quick Start

```bash
# Navigate to deployment directory
cd deployment/docker  # or nginx, or caddy

# Copy example config
cp config.example.json config.json

# Edit with your settings
nano config.json
```

## Configuration Sections

### App Manager

Manages application credentials and settings.

**Array** (Development):
```json
{
  "app_manager": {
    "driver": "Array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret"
        }
      ]
    }
  }
}
```

**PostgreSQL** (Production):
```json
{
  "app_manager": {
    "driver": "Postgres",
    "postgres": {
      "host": "postgres",
      "port": 5432,
      "user": "soketi",
      "password": "secure-password",
      "database": "soketi"
    }
  }
}
```

See [Database Setup](../docs/en/database-setup.md) for MySQL and DynamoDB options.

### Adapter

Handles message distribution across instances.

**Local** (Single instance):
```json
{
  "adapter": {
    "driver": "Local"
  }
}
```

**Redis** (Multiple instances):
```json
{
  "adapter": {
    "driver": "Redis",
    "redis": {
      "host": "redis",
      "port": 6379,
      "key_prefix": "soketi:"
    }
  }
}
```

See [Advanced Adapters](../docs/en/advanced/) for NATS and Cluster options.

### Cache

Caching layer for app configurations.

**Memory** (Development):
```json
{
  "cache": {
    "driver": "Memory"
  }
}
```

**Redis** (Production):
```json
{
  "cache": {
    "driver": "Redis",
    "redis": {
      "host": "redis",
      "port": 6379,
      "db": 1
    }
  }
}
```

### Rate Limiter

Controls request rate limits per app.

**Local** (Single instance):
```json
{
  "rate_limiter": {
    "driver": "Local"
  }
}
```

**Redis** (Multiple instances):
```json
{
  "rate_limiter": {
    "driver": "Redis",
    "redis": {
      "host": "redis",
      "port": 6379,
      "db": 2
    }
  }
}
```

### Queue

Webhook job queue.

**Sync** (Immediate processing):
```json
{
  "queue": {
    "driver": "Sync"
  }
}
```

**Redis** (Async processing):
```json
{
  "queue": {
    "driver": "Redis",
    "redis": {
      "host": "redis",
      "port": 6379,
      "concurrency": 4
    }
  }
}
```

**SQS** (AWS):
```json
{
  "queue": {
    "driver": "Sqs",
    "sqs": {
      "region": "us-east-1",
      "queue_url": "https://sqs.us-east-1.amazonaws.com/...",
      "concurrency": 4
    }
  }
}
```

### Metrics

Prometheus metrics endpoint.

```json
{
  "metrics": {
    "enabled": true,
    "driver": "Prometheus",
    "port": 9601,
    "prefix": "soketi"
  }
}
```

Access metrics at: `http://localhost:9601/metrics`

### SSL/TLS

Enable HTTPS for WebSocket connections.

```json
{
  "ssl": {
    "enabled": true,
    "cert_path": "/path/to/cert.pem",
    "key_path": "/path/to/key.pem"
  }
}
```

**Note**: When using Nginx or Caddy reverse proxy, SSL is handled by the proxy, not Soketi.

### CORS

Cross-Origin Resource Sharing settings.

**Development** (Allow all):
```json
{
  "cors": {
    "enabled": true,
    "origins": ["*"]
  }
}
```

**Production** (Specific domains):
```json
{
  "cors": {
    "enabled": true,
    "origins": ["https://yourdomain.com", "https://app.yourdomain.com"],
    "credentials": true
  }
}
```

### Limits

Configure various limits for channels, events, and presence.

```json
{
  "channel_limits": {
    "max_name_length": 200
  },
  "event_limits": {
    "max_channels_at_once": 100,
    "max_name_length": 200,
    "max_payload_in_kb": 100.0,
    "max_batch_size": 10
  },
  "presence": {
    "max_members_per_channel": 100,
    "max_member_size_in_kb": 2.0
  }
}
```

## Environment Variables

You can override configuration values using environment variables:

```bash
# Server
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_DEBUG=false

# Redis
REDIS_HOST=redis
REDIS_PORT=6379

# PostgreSQL
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_USER=soketi
POSTGRES_PASSWORD=secure-password
POSTGRES_DATABASE=soketi

# Metrics
METRICS_ENABLED=true
METRICS_PORT=9601
```

## Production Checklist

Before deploying to production:

- [ ] Update app credentials (id, key, secret)
- [ ] Configure database connection (PostgreSQL/MySQL/DynamoDB)
- [ ] Enable Redis adapter for clustering
- [ ] Set specific CORS origins
- [ ] Configure SSL/TLS (if not using reverse proxy)
- [ ] Enable metrics
- [ ] Set appropriate rate limits
- [ ] Configure webhooks (if needed)
- [ ] Test configuration with `soketi-rs --config-file config.json --validate`

## Validation

Validate your configuration before starting:

```bash
soketi-rs --config-file config.json --validate
```

## Documentation

For more details, see:

- [Configuration Reference (EN)](../docs/en/configuration.md)
- [Database Setup (EN)](../docs/en/database-setup.md)
- [Docker Deployment (EN)](../docs/en/docker-deployment.md)
- [Yapılandırma (TR)](../docs/tr/yapilandirma.md)

## Support

- [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)
- [GitHub Discussions](https://github.com/ferdiunal/soketi.rs/discussions)
