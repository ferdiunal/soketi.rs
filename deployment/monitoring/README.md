# Soketi Monitoring Stack

Prometheus + Grafana monitoring setup for Soketi WebSocket server.

## Quick Start

```bash
# Create network if not exists
docker network create soketi-network

# Start monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

# Access dashboards
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000 (admin/admin)
```

## Services

- **Prometheus**: Metrics collection and storage (port 9090)
- **Grafana**: Visualization and dashboards (port 3000)
- **Redis Exporter**: Redis metrics (port 9121)

## Grafana Setup

1. Login to Grafana (admin/admin)
2. Add Prometheus data source: http://prometheus:9090
3. Import dashboard or create custom panels

## Metrics Available

- WebSocket connections
- Message throughput
- Channel subscriptions
- Error rates
- Redis performance

## Stop Monitoring

```bash
docker-compose -f docker-compose.monitoring.yml down
```
