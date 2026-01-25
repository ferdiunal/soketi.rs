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

You can override configuration using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SOKETI_HOST` | Server host | `0.0.0.0` |
| `SOKETI_PORT` | Server port | `6001` |
| `SOKETI_DEFAULT_APP_ID` | Default app ID | - |
| `SOKETI_DEFAULT_APP_KEY` | Default app key | - |
| `SOKETI_DEFAULT_APP_SECRET` | Default app secret | - |
| `SOKETI_METRICS_ENABLED` | Enable metrics | `true` |
| `SOKETI_METRICS_PORT` | Metrics port | `9601` |

Example:

```bash
SOKETI_PORT=8080 soketi --config config.json
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

- **[Getting Started](getting-started.md)** - Quick start guide
- **[API Reference](api-reference.md)** - API documentation
- **[Deployment Guide](deployment/reverse-proxy.md)** - Production deployment

## Related Resources

- [MySQL Setup Guide](../MYSQL_SETUP.md)
- [PostgreSQL Setup Guide](../POSTGRES_SETUP.md)
- [DynamoDB Setup Guide](../DYNAMODB_SETUP.md)
- [Redis Adapter Implementation](../REDIS_ADAPTER_IMPLEMENTATION.md)
- [NATS Adapter Implementation](../NATS_ADAPTER_IMPLEMENTATION.md)
