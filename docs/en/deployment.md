# Deployment Guide

This guide covers production deployment strategies for Soketi.rs.

## Table of Contents

- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Cloud Providers](#cloud-providers)
- [Load Balancing](#load-balancing)
- [Monitoring](#monitoring)
- [Security](#security)

## Docker Deployment

### Single Instance

```bash
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json:ro \
  --restart unless-stopped \
  soketi-rs:latest
```

### Docker Compose (Recommended)

```yaml
version: '3.8'

services:
  soketi:
    image: soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    volumes:
      - ./config.json:/app/config.json:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
```

### Multi-Instance with Redis

```yaml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes

  soketi:
    image: soketi-rs:latest
    ports:
      - "6001-6003:6001"
    volumes:
      - ./config.json:/app/config.json:ro
    environment:
      - RUST_LOG=info
    depends_on:
      - redis
    deploy:
      replicas: 3

volumes:
  redis-data:
```

## Kubernetes Deployment

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: soketi
  labels:
    app: soketi
spec:
  replicas: 3
  selector:
    matchLabels:
      app: soketi
  template:
    metadata:
      labels:
        app: soketi
    spec:
      containers:
      - name: soketi
        image: soketi-rs:latest
        ports:
        - containerPort: 6001
          name: websocket
        - containerPort: 9601
          name: metrics
        env:
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /app/config.json
          subPath: config.json
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /
            port: 6001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 6001
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: soketi-config
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: soketi
spec:
  type: LoadBalancer
  ports:
  - port: 6001
    targetPort: 6001
    name: websocket
  - port: 9601
    targetPort: 9601
    name: metrics
  selector:
    app: soketi
```

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: soketi-config
data:
  config.json: |
    {
      "host": "0.0.0.0",
      "port": 6001,
      "adapter": {
        "driver": "Redis",
        "redis": {
          "host": "redis-service",
          "port": 6379
        }
      }
    }
```

### HorizontalPodAutoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: soketi-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: soketi
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Cloud Providers

### AWS

#### ECS Fargate

```json
{
  "family": "soketi",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "containerDefinitions": [
    {
      "name": "soketi",
      "image": "soketi-rs:latest",
      "portMappings": [
        {
          "containerPort": 6001,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/soketi",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### Application Load Balancer

- Target Group: WebSocket protocol
- Health Check: HTTP on port 6001
- Sticky Sessions: Enabled
- Connection Draining: 300 seconds

### Google Cloud Platform

#### Cloud Run

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: soketi
spec:
  template:
    spec:
      containers:
      - image: gcr.io/project-id/soketi-rs:latest
        ports:
        - containerPort: 6001
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            memory: "1Gi"
            cpu: "2000m"
```

### Azure

#### Container Instances

```bash
az container create \
  --resource-group soketi-rg \
  --name soketi \
  --image soketi-rs:latest \
  --ports 6001 9601 \
  --cpu 2 \
  --memory 2 \
  --environment-variables RUST_LOG=info
```

## Load Balancing

### Nginx

```nginx
upstream soketi {
    least_conn;
    server soketi1:6001;
    server soketi2:6001;
    server soketi3:6001;
}

server {
    listen 80;
    server_name ws.example.com;

    location / {
        proxy_pass http://soketi;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket timeouts
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
    }
}
```

### HAProxy

```
frontend websocket
    bind *:80
    mode http
    option http-server-close
    option forwardfor
    default_backend soketi_servers

backend soketi_servers
    mode http
    balance leastconn
    option httpchk GET /
    server soketi1 soketi1:6001 check
    server soketi2 soketi2:6001 check
    server soketi3 soketi3:6001 check
```

## Monitoring

### Prometheus

```yaml
scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['soketi:9601']
    metrics_path: '/metrics'
    scrape_interval: 10s
```

### Grafana Dashboard

Import dashboard ID: `soketi-rs-dashboard`

Key metrics to monitor:
- Active connections
- Messages per second
- CPU usage
- Memory usage
- Error rate
- Latency

### Alerts

```yaml
groups:
  - name: soketi
    rules:
      - alert: HighConnectionCount
        expr: soketi_connections > 80000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High connection count"
          
      - alert: HighErrorRate
        expr: rate(soketi_errors_total[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate"
```

## Security

### SSL/TLS

```json
{
  "ssl": {
    "enabled": true,
    "cert_path": "/etc/ssl/certs/server.crt",
    "key_path": "/etc/ssl/private/server.key"
  }
}
```

### Firewall Rules

```bash
# Allow WebSocket
ufw allow 6001/tcp

# Allow Metrics (internal only)
ufw allow from 10.0.0.0/8 to any port 9601

# Deny all other traffic
ufw default deny incoming
```

### Network Policies (Kubernetes)

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: soketi-policy
spec:
  podSelector:
    matchLabels:
      app: soketi
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: nginx
    ports:
    - protocol: TCP
      port: 6001
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

## Best Practices

1. **Use Redis for clustering** in production
2. **Enable metrics** and set up monitoring
3. **Configure health checks** for automatic recovery
4. **Set resource limits** to prevent resource exhaustion
5. **Use SSL/TLS** for secure connections
6. **Implement rate limiting** to prevent abuse
7. **Set up log aggregation** for debugging
8. **Use sticky sessions** with load balancers
9. **Configure connection timeouts** appropriately
10. **Regular backups** of configuration and data

## Troubleshooting

### High Memory Usage

- Check connection count
- Review message size limits
- Enable connection pooling
- Increase instance count

### Connection Drops

- Check load balancer timeout settings
- Verify network stability
- Review rate limiting configuration
- Check Redis connectivity

### Slow Performance

- Enable Redis adapter
- Increase CPU allocation
- Review database queries
- Check network latency

---

For more information, see the [main documentation](../README.md).
