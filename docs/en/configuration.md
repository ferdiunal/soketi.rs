# Configuration Reference

> Complete reference for all Soketi configuration options.

## Table of Contents

- [Configuration File](#configuration-file)
- [Environment Variables](#environment-variables)
- [App Manager Configuration](#app-manager-configuration)
- [Server Configuration](#server-configuration)
- [Metrics Configuration](#metrics-configuration)
- [Adapter Configuration](#adapter-configuration)
- [Queue Configuration](#queue-configuration)
- [Rate Limiting](#rate-limiting)
- [SSL/TLS Configuration](#ssltls-configuration)

## Configuration File

Soketi can be configured using a JSON configuration file. By default, it looks for `config.json` in the current directory.

Specify a custom configuration file:

```bash
soketi --config /path/to/config.json
```

### Basic Configuration Example

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

## Environment Variables

All configuration options can be overridden using environment variables with the `PUSHER_` prefix.

### Server Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_HOST` | Server host | `0.0.0.0` |
| `PUSHER_PORT` | Server port | `6001` |
| `PUSHER_DEBUG` | Enable debug mode | `false` |
| `PUSHER_MODE` | Server mode (full, server, worker) | `full` |

### App Manager Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_APP_MANAGER_DRIVER` | App manager driver (array, dynamodb, mysql, postgres) | `array` |
| `PUSHER_APP_MANAGER_ARRAY_APPS` | JSON array of apps (for array driver) | `[]` |
| `PUSHER_DEFAULT_APP_ID` | Default app ID (creates a single app) | - |
| `PUSHER_DEFAULT_APP_KEY` | Default app key (creates a single app) | - |
| `PUSHER_DEFAULT_APP_SECRET` | Default app secret (creates a single app) | - |
| `PUSHER_APP_MANAGER_CACHE_ENABLED` | Enable app manager cache | `false` |
| `PUSHER_APP_MANAGER_CACHE_TTL` | Cache TTL in seconds | `3600` |

### Adapter Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_ADAPTER_DRIVER` | Adapter driver (local, cluster, redis, nats) | `local` |
| `PUSHER_ADAPTER_REDIS_HOST` | Redis adapter host | `127.0.0.1` |
| `PUSHER_ADAPTER_REDIS_PORT` | Redis adapter port | `6379` |
| `PUSHER_ADAPTER_REDIS_DB` | Redis adapter database | `0` |
| `PUSHER_ADAPTER_REDIS_PASSWORD` | Redis adapter password | - |
| `PUSHER_ADAPTER_CLUSTER_PORT` | Cluster adapter port | `11002` |
| `PUSHER_ADAPTER_NATS_SERVERS` | NATS servers (comma-separated) | `127.0.0.1:4222` |

### Cache Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_CACHE_DRIVER` | Cache driver (memory, redis) | `memory` |
| `PUSHER_CACHE_REDIS_HOST` | Redis cache host | `127.0.0.1` |
| `PUSHER_CACHE_REDIS_PORT` | Redis cache port | `6379` |
| `PUSHER_CACHE_REDIS_PASSWORD` | Redis cache password | - |

### Metrics Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_METRICS_ENABLED` | Enable metrics | `false` |
| `PUSHER_METRICS_PORT` | Metrics port | `9601` |
| `PUSHER_METRICS_PREFIX` | Metrics prefix | `pusher` |

### Database Configuration (MySQL)

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_MYSQL_HOST` | MySQL host | `127.0.0.1` |
| `PUSHER_MYSQL_PORT` | MySQL port | `3306` |
| `PUSHER_MYSQL_USER` | MySQL user | `root` |
| `PUSHER_MYSQL_PASSWORD` | MySQL password | - |
| `PUSHER_MYSQL_DATABASE` | MySQL database | `main` |

### Database Configuration (PostgreSQL)

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_POSTGRES_HOST` | PostgreSQL host | `127.0.0.1` |
| `PUSHER_POSTGRES_PORT` | PostgreSQL port | `5432` |
| `PUSHER_POSTGRES_USER` | PostgreSQL user | `postgres` |
| `PUSHER_POSTGRES_PASSWORD` | PostgreSQL password | - |
| `PUSHER_POSTGRES_DATABASE` | PostgreSQL database | `main` |

### Database Configuration (DynamoDB)

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_DYNAMODB_TABLE` | DynamoDB table name | `apps` |
| `PUSHER_DYNAMODB_REGION` | AWS region | `us-east-1` |

### Queue Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_QUEUE_DRIVER` | Queue driver (sync, redis, sqs) | `sync` |
| `PUSHER_QUEUE_REDIS_HOST` | Redis queue host | `127.0.0.1` |
| `PUSHER_QUEUE_REDIS_PORT` | Redis queue port | `6379` |
| `PUSHER_QUEUE_SQS_URL` | SQS queue URL | - |
| `PUSHER_QUEUE_SQS_REGION` | SQS region | `us-east-1` |

### SSL Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_SSL_ENABLED` | Enable SSL | `false` |
| `PUSHER_SSL_CERT_PATH` | SSL certificate path | - |
| `PUSHER_SSL_KEY_PATH` | SSL key path | - |

### CORS Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PUSHER_CORS_ENABLED` | Enable CORS | `true` |
| `PUSHER_CORS_ORIGINS` | Allowed origins (comma-separated) | `*` |

### Examples

#### Using Default App (Simple Setup)

```bash
PUSHER_DEFAULT_APP_ID=my-app \
PUSHER_DEFAULT_APP_KEY=my-key \
PUSHER_DEFAULT_APP_SECRET=my-secret \
soketi-rs
```

#### Using JSON Array of Apps

```bash
PUSHER_APP_MANAGER_ARRAY_APPS='[{"id":"app1","key":"key1","secret":"secret1","enabled":true}]' \
soketi-rs
```

#### With Redis Adapter

```bash
PUSHER_ADAPTER_DRIVER=redis \
PUSHER_ADAPTER_REDIS_HOST=localhost \
PUSHER_ADAPTER_REDIS_PORT=6379 \
PUSHER_ADAPTER_REDIS_PASSWORD=mypassword \
soketi-rs
```

#### Docker Compose Example

```yaml
environment:
  PUSHER_DEFAULT_APP_ID: "shopilens"
  PUSHER_DEFAULT_APP_KEY: "shopilens-key"
  PUSHER_DEFAULT_APP_SECRET: "shopilens-secret"
  PUSHER_ADAPTER_DRIVER: "redis"
  PUSHER_ADAPTER_REDIS_HOST: "redis"
  PUSHER_ADAPTER_REDIS_PORT: "6379"
  PUSHER_ADAPTER_REDIS_PASSWORD: "mypassword"
  PUSHER_CACHE_DRIVER: "redis"
  PUSHER_CACHE_REDIS_HOST: "redis"
  PUSHER_METRICS_ENABLED: "true"
  PUSHER_METRICS_PORT: "9601"
```

## App Manager Configuration

The app manager controls how applications are stored and managed.

### Array Driver (In-Memory)

Best for development and single-instance deployments:

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
          "max_read_requests_per_second": 100,
          "webhooks": []
        }
      ]
    }
  }
}
```

### MySQL Driver

For production deployments with MySQL:

```json
{
  "app_manager": {
    "driver": "mysql",
    "mysql": {
      "host": "localhost",
      "port": 3306,
      "database": "soketi",
      "username": "soketi",
      "password": "password"
    }
  }
}
```

### PostgreSQL Driver

For production deployments with PostgreSQL:

```json
{
  "app_manager": {
    "driver": "postgres",
    "postgres": {
      "host": "localhost",
      "port": 5432,
      "database": "soketi",
      "username": "soketi",
      "password": "password"
    }
  }
}
```

### DynamoDB Driver

For AWS deployments:

```json
{
  "app_manager": {
    "driver": "dynamodb",
    "dynamodb": {
      "table": "soketi_apps",
      "region": "us-east-1"
    }
  }
}
```

## Server Configuration

### Basic Server Options

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "path": "/",
  "max_payload_size": 100000
}
```

| Option | Description | Default |
|--------|-------------|---------|
| `host` | Server bind address | `0.0.0.0` |
| `port` | Server port | `6001` |
| `path` | WebSocket path | `/` |
| `max_payload_size` | Maximum message size in bytes | `100000` |

## Metrics Configuration

Enable Prometheus-compatible metrics:

```json
{
  "metrics": {
    "enabled": true,
    "port": 9601,
    "host": "0.0.0.0"
  }
}
```

Access metrics at `http://localhost:9601/metrics`.

## Adapter Configuration

Adapters enable horizontal scaling by synchronizing events across multiple Soketi instances.

### Local Adapter (Default)

No configuration needed. Only works with single instance:

```json
{
  "adapter": {
    "driver": "local"
  }
}
```

### Redis Adapter

For multi-instance deployments:

```json
{
  "adapter": {
    "driver": "redis",
    "redis": {
      "host": "localhost",
      "port": 6379,
      "password": "password",
      "db": 0,
      "key_prefix": "soketi"
    }
  }
}
```

### NATS Adapter

For NATS-based clustering:

```json
{
  "adapter": {
    "driver": "nats",
    "nats": {
      "servers": ["nats://localhost:4222"],
      "prefix": "soketi"
    }
  }
}
```

## Queue Configuration

Configure queue for webhook delivery:

### Local Queue

```json
{
  "queue": {
    "driver": "local"
  }
}
```

### SQS Queue

For AWS SQS:

```json
{
  "queue": {
    "driver": "sqs",
    "sqs": {
      "queue_url": "https://sqs.us-east-1.amazonaws.com/123456789/soketi-webhooks",
      "region": "us-east-1"
    }
  }
}
```

## Rate Limiting

Configure rate limits per application:

```json
{
  "id": "app-id",
  "max_connections": 100,
  "max_backend_events_per_second": 100,
  "max_client_events_per_second": 100,
  "max_read_requests_per_second": 100
}
```

| Option | Description | Default |
|--------|-------------|---------|
| `max_connections` | Maximum concurrent connections | `100` |
| `max_backend_events_per_second` | Backend event rate limit | `100` |
| `max_client_events_per_second` | Client event rate limit | `100` |
| `max_read_requests_per_second` | Read request rate limit | `100` |

## SSL/TLS Configuration

Enable SSL/TLS for secure connections:

```json
{
  "ssl": {
    "enabled": true,
    "cert_file": "/path/to/cert.pem",
    "key_file": "/path/to/key.pem"
  }
}
```

**Note**: In production, it's recommended to use a reverse proxy (Caddy or Nginx) for SSL/TLS termination.

## Complete Configuration Example

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "path": "/",
  "max_payload_size": 100000,
  
  "app_manager": {
    "driver": "postgres",
    "postgres": {
      "host": "localhost",
      "port": 5432,
      "database": "soketi",
      "username": "soketi",
      "password": "password"
    }
  },
  
  "adapter": {
    "driver": "redis",
    "redis": {
      "host": "localhost",
      "port": 6379,
      "password": "password",
      "db": 0,
      "key_prefix": "soketi"
    }
  },
  
  "queue": {
    "driver": "sqs",
    "sqs": {
      "queue_url": "https://sqs.us-east-1.amazonaws.com/123456789/soketi-webhooks",
      "region": "us-east-1"
    }
  },
  
  "metrics": {
    "enabled": true,
    "port": 9601,
    "host": "0.0.0.0"
  },
  
  "ssl": {
    "enabled": false
  }
}
```

## Next Steps

- **[Getting Started](getting-started)** - Quick start guide
- **[API Reference](api-reference)** - API documentation
- **[Deployment Guide](deployment/reverse-proxy)** - Production deployment

## Related Resources

- [MySQL Setup Guide](../MYSQL_SETUP)
- [PostgreSQL Setup Guide](../POSTGRES_SETUP)
- [DynamoDB Setup Guide](../DYNAMODB_SETUP)
- [Redis Adapter Implementation](../REDIS_ADAPTER_IMPLEMENTATION)
- [NATS Adapter Implementation](../NATS_ADAPTER_IMPLEMENTATION)
