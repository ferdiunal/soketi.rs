# Environment Variables Guide

This guide explains how to configure Soketi using environment variables.

## Overview

Soketi supports configuration through:
1. **Configuration files** (JSON, YAML, TOML) - Loaded via `--config` flag
2. **Environment variables** - Prefix: `PUSHER_`
3. **Command-line arguments** - Highest priority

Priority order (highest to lowest):
1. Command-line arguments
2. Environment variables
3. Configuration file
4. Default values

## Quick Start

### Simple Setup with Default App

The easiest way to get started is using the default app environment variables:

```bash
PUSHER_DEFAULT_APP_ID=my-app \
PUSHER_DEFAULT_APP_KEY=my-key \
PUSHER_DEFAULT_APP_SECRET=my-secret \
./soketi-rs
```

### Docker Compose Setup

```yaml
services:
  soketi:
    image: funal/soketi-rs:latest
    environment:
      PUSHER_DEFAULT_APP_ID: "my-app"
      PUSHER_DEFAULT_APP_KEY: "my-key"
      PUSHER_DEFAULT_APP_SECRET: "my-secret"
      PUSHER_ADAPTER_DRIVER: "redis"
      PUSHER_ADAPTER_REDIS_HOST: "redis"
      PUSHER_METRICS_ENABLED: "true"
```

## App Configuration

### Option 1: Default App (Recommended for Single App)

Create a single app using three environment variables:

```bash
PUSHER_DEFAULT_APP_ID=my-app
PUSHER_DEFAULT_APP_KEY=my-key
PUSHER_DEFAULT_APP_SECRET=my-secret
```

This is the simplest way to configure a single application.

### Option 2: JSON Array (Multiple Apps)

For multiple apps, use a JSON array:

```bash
PUSHER_APP_MANAGER_ARRAY_APPS='[
  {
    "id": "app1",
    "key": "key1",
    "secret": "secret1",
    "enabled": true,
    "enable_client_messages": true,
    "max_connections": 10000
  },
  {
    "id": "app2",
    "key": "key2",
    "secret": "secret2",
    "enabled": true
  }
]'
```

### Option 3: Configuration File + Environment Variables

Mount a config file and override specific values:

```yaml
services:
  soketi:
    image: funal/soketi-rs:latest
    volumes:
      - ./config.json:/app/config.json:ro
    environment:
      # Override adapter to use Redis
      PUSHER_ADAPTER_DRIVER: "redis"
      PUSHER_ADAPTER_REDIS_HOST: "redis"
      PUSHER_ADAPTER_REDIS_PASSWORD: "mypassword"
```

## Complete Environment Variables Reference

### Server Configuration

```bash
PUSHER_HOST=0.0.0.0                    # Server bind address
PUSHER_PORT=6001                       # Server port
PUSHER_DEBUG=true                      # Enable debug logging
PUSHER_MODE=full                       # Server mode: full, server, worker
PUSHER_SHUTDOWN_GRACE_PERIOD_MS=3000   # Graceful shutdown timeout
```

### App Manager

```bash
PUSHER_APP_MANAGER_DRIVER=array        # Driver: array, dynamodb, mysql, postgres
PUSHER_APP_MANAGER_CACHE_ENABLED=true  # Enable app caching
PUSHER_APP_MANAGER_CACHE_TTL=3600      # Cache TTL in seconds

# Default app (creates single app)
PUSHER_DEFAULT_APP_ID=my-app
PUSHER_DEFAULT_APP_KEY=my-key
PUSHER_DEFAULT_APP_SECRET=my-secret

# Or JSON array (multiple apps)
PUSHER_APP_MANAGER_ARRAY_APPS='[...]'
```

### Adapter Configuration

```bash
PUSHER_ADAPTER_DRIVER=redis            # Driver: local, cluster, redis, nats

# Redis Adapter
PUSHER_ADAPTER_REDIS_HOST=127.0.0.1
PUSHER_ADAPTER_REDIS_PORT=6379
PUSHER_ADAPTER_REDIS_DB=0
PUSHER_ADAPTER_REDIS_PASSWORD=secret

# Cluster Adapter
PUSHER_ADAPTER_CLUSTER_PORT=11002

# NATS Adapter
PUSHER_ADAPTER_NATS_SERVERS=nats://localhost:4222,nats://localhost:4223
```

### Cache Configuration

```bash
PUSHER_CACHE_DRIVER=redis              # Driver: memory, redis

# Redis Cache
PUSHER_CACHE_REDIS_HOST=127.0.0.1
PUSHER_CACHE_REDIS_PORT=6379
PUSHER_CACHE_REDIS_PASSWORD=secret
```

### Rate Limiter

```bash
PUSHER_RATE_LIMITER_DRIVER=local       # Driver: local, cluster, redis

# Redis Rate Limiter
PUSHER_RATE_LIMITER_REDIS_HOST=127.0.0.1
PUSHER_RATE_LIMITER_REDIS_PORT=6379
```

### Queue Configuration

```bash
PUSHER_QUEUE_DRIVER=sync               # Driver: sync, redis, sqs

# Redis Queue
PUSHER_QUEUE_REDIS_HOST=127.0.0.1
PUSHER_QUEUE_REDIS_PORT=6379

# SQS Queue
PUSHER_QUEUE_SQS_URL=https://sqs.us-east-1.amazonaws.com/123/queue
PUSHER_QUEUE_SQS_REGION=us-east-1
```

### Metrics

```bash
PUSHER_METRICS_ENABLED=true
PUSHER_METRICS_PORT=9601
PUSHER_METRICS_PREFIX=soketi
```

### Database Configuration

#### MySQL

```bash
PUSHER_MYSQL_HOST=127.0.0.1
PUSHER_MYSQL_PORT=3306
PUSHER_MYSQL_USER=root
PUSHER_MYSQL_PASSWORD=secret
PUSHER_MYSQL_DATABASE=soketi
```

#### PostgreSQL

```bash
PUSHER_POSTGRES_HOST=127.0.0.1
PUSHER_POSTGRES_PORT=5432
PUSHER_POSTGRES_USER=postgres
PUSHER_POSTGRES_PASSWORD=secret
PUSHER_POSTGRES_DATABASE=soketi
```

#### DynamoDB

```bash
PUSHER_DYNAMODB_TABLE=soketi_apps
PUSHER_DYNAMODB_REGION=us-east-1
```

### SSL/TLS

```bash
PUSHER_SSL_ENABLED=true
PUSHER_SSL_CERT_PATH=/path/to/cert.pem
PUSHER_SSL_KEY_PATH=/path/to/key.pem
```

### CORS

```bash
PUSHER_CORS_ENABLED=true
PUSHER_CORS_ORIGINS=https://example.com,https://app.example.com
```

### Limits

```bash
PUSHER_CHANNEL_MAX_NAME_LENGTH=200
PUSHER_EVENT_MAX_NAME_LENGTH=200
PUSHER_EVENT_MAX_PAYLOAD_KB=100
PUSHER_EVENT_MAX_BATCH_SIZE=10
PUSHER_PRESENCE_MAX_MEMBERS=100
PUSHER_PRESENCE_MAX_MEMBER_SIZE_KB=2
PUSHER_HTTP_MAX_REQUEST_SIZE_KB=100
PUSHER_HTTP_MEMORY_THRESHOLD_MB=512
PUSHER_USER_AUTH_TIMEOUT_MS=30000
```

## Common Scenarios

### Development (Single Instance)

```bash
PUSHER_DEFAULT_APP_ID=dev-app
PUSHER_DEFAULT_APP_KEY=dev-key
PUSHER_DEFAULT_APP_SECRET=dev-secret
PUSHER_DEBUG=true
```

### Production (Horizontal Scaling with Redis)

```bash
PUSHER_DEFAULT_APP_ID=prod-app
PUSHER_DEFAULT_APP_KEY=prod-key
PUSHER_DEFAULT_APP_SECRET=prod-secret
PUSHER_ADAPTER_DRIVER=redis
PUSHER_ADAPTER_REDIS_HOST=redis.example.com
PUSHER_ADAPTER_REDIS_PASSWORD=secret
PUSHER_CACHE_DRIVER=redis
PUSHER_CACHE_REDIS_HOST=redis.example.com
PUSHER_METRICS_ENABLED=true
```

### Production (Database-backed Apps)

```bash
PUSHER_APP_MANAGER_DRIVER=postgres
PUSHER_POSTGRES_HOST=db.example.com
PUSHER_POSTGRES_USER=soketi
PUSHER_POSTGRES_PASSWORD=secret
PUSHER_POSTGRES_DATABASE=soketi
PUSHER_ADAPTER_DRIVER=redis
PUSHER_ADAPTER_REDIS_HOST=redis.example.com
```

## Tips

1. **Use .env files**: Create a `.env` file for local development
2. **Secrets management**: Use secret management tools (AWS Secrets Manager, HashiCorp Vault) for production
3. **Config file for complex setups**: Use config files for complex app configurations with webhooks
4. **Environment variables for deployment**: Use environment variables for deployment-specific settings (hosts, passwords)
5. **Boolean values**: Use `true` or `false` (not `1` or `0`)

## Troubleshooting

### App Not Found Error

If you see "App not found" errors:

1. Check that you've set either:
   - `PUSHER_DEFAULT_APP_*` variables, OR
   - `PUSHER_APP_MANAGER_ARRAY_APPS`, OR
   - Mounted a config file with apps defined

2. Verify the app key matches what your client is using

3. Check logs for configuration loading errors:
   ```bash
   PUSHER_DEBUG=true ./soketi-rs
   ```

### Boolean Value Errors

If you see "invalid value '1' for '--debug'" errors:

Use `true`/`false` instead of `1`/`0`:
```bash
# ❌ Wrong
PUSHER_DEBUG=1

# ✅ Correct
PUSHER_DEBUG=true
```

## See Also

- [Configuration Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md)
- [Docker Deployment Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/docker-deployment.md)
- [Getting Started](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md)
