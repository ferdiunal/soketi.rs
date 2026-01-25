# Database Setup Guide

This guide covers database configuration for different app managers in Soketi.rs.

## Supported Databases

Soketi.rs supports multiple database backends for app configuration:

- **PostgreSQL** - Recommended for production
- **MySQL** - Alternative SQL database
- **DynamoDB** - AWS-native NoSQL solution
- **Array** - In-memory configuration (development only)

## PostgreSQL Setup

### Installation

**macOS (Homebrew):**
```bash
brew install postgresql@16
brew services start postgresql@16
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Docker:**
```bash
docker run --name soketi-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=soketi \
  -p 5432:5432 \
  -d postgres:16
```

### Database and Table Creation

```sql
-- Create database
CREATE DATABASE soketi;

-- Connect to database
\c soketi

-- Create apps table
CREATE TABLE apps (
    id VARCHAR(255) PRIMARY KEY,
    key VARCHAR(255) NOT NULL UNIQUE,
    secret VARCHAR(255) NOT NULL,
    max_connections BIGINT,
    enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    max_backend_events_per_second BIGINT,
    max_client_events_per_second BIGINT,
    max_read_requests_per_second BIGINT,
    webhooks JSONB,
    max_presence_members_per_channel BIGINT,
    max_presence_member_size_in_kb DOUBLE PRECISION,
    max_channel_name_length BIGINT,
    max_event_channels_at_once BIGINT,
    max_event_name_length BIGINT,
    max_event_payload_in_kb DOUBLE PRECISION,
    max_event_batch_size BIGINT,
    enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create index for faster lookups
CREATE INDEX idx_apps_key ON apps(key);
```

### Sample Data

```sql
INSERT INTO apps (id, key, secret, enabled, enable_client_messages)
VALUES ('app-1', 'app-key-1', 'app-secret-1', true, false);
```

### Connection String

```
postgresql://username:password@host:port/database
```

Example:
```
postgresql://postgres:password@localhost:5432/soketi
```

## MySQL Setup

### Installation

**macOS (Homebrew):**
```bash
brew install mysql
brew services start mysql
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install mysql-server
sudo systemctl start mysql
```

**Docker:**
```bash
docker run --name soketi-mysql \
  -e MYSQL_ROOT_PASSWORD=password \
  -e MYSQL_DATABASE=soketi \
  -p 3306:3306 \
  -d mysql:8.0
```

### Database and Table Creation

```sql
-- Create database
CREATE DATABASE soketi CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

USE soketi;

-- Create apps table
CREATE TABLE apps (
    id VARCHAR(255) PRIMARY KEY,
    `key` VARCHAR(255) NOT NULL UNIQUE,
    secret VARCHAR(255) NOT NULL,
    max_connections BIGINT UNSIGNED,
    enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    max_backend_events_per_second BIGINT UNSIGNED,
    max_client_events_per_second BIGINT UNSIGNED,
    max_read_requests_per_second BIGINT UNSIGNED,
    webhooks JSON,
    max_presence_members_per_channel BIGINT UNSIGNED,
    max_presence_member_size_in_kb DOUBLE,
    max_channel_name_length BIGINT UNSIGNED,
    max_event_channels_at_once BIGINT UNSIGNED,
    max_event_name_length BIGINT UNSIGNED,
    max_event_payload_in_kb DOUBLE,
    max_event_batch_size BIGINT UNSIGNED,
    enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE,
    INDEX idx_key (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

### Connection String

```
mysql://username:password@host:port/database
```

Example:
```
mysql://root:password@localhost:3306/soketi
```

## DynamoDB Setup (AWS)

### Table Schema

**Primary Key:**
- Partition Key: `id` (String)

**Global Secondary Index:**
- Index Name: `key-index`
- Partition Key: `key` (String)

### Creating the Table

**AWS CLI:**
```bash
aws dynamodb create-table \
    --table-name apps \
    --attribute-definitions \
        AttributeName=id,AttributeType=S \
        AttributeName=key,AttributeType=S \
    --key-schema \
        AttributeName=id,KeyType=HASH \
    --global-secondary-indexes \
        "[
            {
                \"IndexName\": \"key-index\",
                \"KeySchema\": [{\"AttributeName\":\"key\",\"KeyType\":\"HASH\"}],
                \"Projection\":{\"ProjectionType\":\"ALL\"},
                \"ProvisionedThroughput\": {
                    \"ReadCapacityUnits\": 5,
                    \"WriteCapacityUnits\": 5
                }
            }
        ]" \
    --provisioned-throughput \
        ReadCapacityUnits=5,WriteCapacityUnits=5 \
    --region us-east-1
```

**Terraform:**
```hcl
resource "aws_dynamodb_table" "apps" {
  name           = "apps"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "id"

  attribute {
    name = "id"
    type = "S"
  }

  attribute {
    name = "key"
    type = "S"
  }

  global_secondary_index {
    name            = "key-index"
    hash_key        = "key"
    projection_type = "ALL"
  }
}
```

### Local Development with DynamoDB Local

```bash
# Start DynamoDB Local
docker run -p 8000:8000 amazon/dynamodb-local

# Create table with local endpoint
aws dynamodb create-table \
    --endpoint-url http://localhost:8000 \
    --table-name apps \
    # ... (same parameters as above)
```

### Environment Variables

```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1
export DYNAMODB_TABLE=apps
export DYNAMODB_ENDPOINT=http://localhost:8000  # For local development
```

## Configuration in Soketi.rs

### PostgreSQL

```json
{
  "app_manager": {
    "driver": "postgres",
    "postgres": {
      "connection_string": "postgresql://postgres:password@localhost/soketi",
      "table_name": "apps"
    }
  }
}
```

### MySQL

```json
{
  "app_manager": {
    "driver": "mysql",
    "mysql": {
      "connection_string": "mysql://root:password@localhost/soketi",
      "table_name": "apps"
    }
  }
}
```

### DynamoDB

```json
{
  "app_manager": {
    "driver": "dynamodb",
    "dynamodb": {
      "table_name": "apps",
      "region": "us-east-1"
    }
  }
}
```

## Connection Pooling

### PostgreSQL/MySQL

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .min_connections(2)
    .max_connections(10)
    .connect_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect("postgresql://...")
    .await?;
```

**Recommended Settings:**
- Development: 2-5 connections
- Production (single instance): 10-20 connections
- Production (multiple instances): 5-10 connections per instance

## Caching

Enable caching to reduce database load:

```json
{
  "cache": {
    "driver": "redis",
    "redis": {
      "host": "localhost",
      "port": 6379
    }
  }
}
```

Cache TTL: 3600 seconds (1 hour) by default.

## Security Best Practices

1. **Use strong passwords** - Never use default passwords in production
2. **Enable SSL/TLS** - Use encrypted connections
3. **Limit permissions** - Create dedicated users with minimal permissions
4. **Network security** - Restrict database access to trusted IPs
5. **Regular backups** - Set up automated backups

### Creating Dedicated Users

**PostgreSQL:**
```sql
CREATE USER soketi_app WITH PASSWORD 'strong_password';
GRANT CONNECT ON DATABASE soketi TO soketi_app;
GRANT SELECT ON apps TO soketi_app;
```

**MySQL:**
```sql
CREATE USER 'soketi_app'@'localhost' IDENTIFIED BY 'strong_password';
GRANT SELECT ON soketi.apps TO 'soketi_app'@'localhost';
FLUSH PRIVILEGES;
```

## Monitoring

### PostgreSQL

```sql
-- View active connections
SELECT * FROM pg_stat_activity WHERE datname = 'soketi';

-- Check index usage
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
WHERE tablename = 'apps';
```

### MySQL

```sql
-- View active connections
SHOW PROCESSLIST;

-- Check index usage
SHOW INDEX FROM apps;
```

### DynamoDB

Monitor CloudWatch metrics:
- `ConsumedReadCapacityUnits`
- `ConsumedWriteCapacityUnits`
- `UserErrors` (throttling)
- `SystemErrors`

## Troubleshooting

### Connection Refused

1. Verify database is running
2. Check port is accessible
3. Verify firewall rules
4. Check connection string

### Authentication Failed

1. Verify credentials
2. Check user permissions
3. Ensure user can connect from application host

### Slow Queries

1. Verify indexes exist
2. Analyze query execution plans
3. Enable caching
4. Consider read replicas

---

For more information, see [Configuration](configuration.md) and [Deployment](deployment.md).
