# Caddy + Soketi Docker Compose Setup

This docker-compose configuration provides a production-ready setup for running soketi.rs with Caddy as a reverse proxy, featuring HTTP/2, HTTP/3 (QUIC), and automatic HTTPS support.

## Architecture

```
Client (Browser/App)
    ↓
Caddy Reverse Proxy (HTTP/2, HTTP/3)
    ↓
Soketi WebSocket Server
```

## Features

- **Modern Protocol Support**: HTTP/1.1, HTTP/2, and HTTP/3 (QUIC)
- **Automatic HTTPS**: Caddy automatically obtains and renews SSL certificates
- **WebSocket Proxying**: Properly configured WebSocket upgrade handling
- **Health Checks**: Built-in health monitoring for both services
- **Security Headers**: HSTS, X-Frame-Options, X-Content-Type-Options, etc.
- **Persistent Storage**: Volumes for Caddy certificates and configuration
- **Network Isolation**: Dedicated Docker network for service communication

## Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- (Optional) A domain name pointing to your server for automatic HTTPS

## Quick Start

### 1. Configure Environment Variables (Recommended)

Copy the example environment file and customize it:

```bash
cp .env.caddy.example .env
```

Edit `.env` and update the values, especially:
- `SOKETI_DEFAULT_APP_ID`
- `SOKETI_DEFAULT_APP_KEY`
- `SOKETI_DEFAULT_APP_SECRET`
- `DOMAIN` (for production with HTTPS)

### 2. Development Mode (HTTP only)

For local development without HTTPS:

```bash
# Start services
docker-compose -f docker-compose.caddy.yml up -d

# View logs
docker-compose -f docker-compose.caddy.yml logs -f

# Stop services
docker-compose -f docker-compose.caddy.yml down
```

Access soketi at: `http://localhost/app/`

### 3. Using Environment Variables

The docker-compose configuration supports environment variables for easy customization:

**Option 1: Using .env file (Recommended)**
```bash
cp .env.caddy.example .env
# Edit .env with your values
docker-compose -f docker-compose.caddy.yml up -d
```

**Option 2: Inline environment variables**
```bash
SOKETI_DEFAULT_APP_KEY=my-key docker-compose -f docker-compose.caddy.yml up -d
```

**Option 3: Export environment variables**
```bash
export SOKETI_DEFAULT_APP_KEY=my-key
export SOKETI_DEFAULT_APP_SECRET=my-secret
docker-compose -f docker-compose.caddy.yml up -d
```

### 3. Production Mode (HTTPS with custom domain)

For production with automatic HTTPS:

1. **Configure environment variables** in `.env`:
   ```bash
   DOMAIN=your-domain.com
   SOKETI_DEFAULT_APP_ID=your-app-id
   SOKETI_DEFAULT_APP_KEY=your-app-key
   SOKETI_DEFAULT_APP_SECRET=your-secure-secret
   ```

2. **Update Caddyfile** with your domain:
   ```caddyfile
   your-domain.com {
       reverse_proxy /app/* {
           to soketi:6001
           # ... rest of config
       }
   }
   ```

3. **Start services**:
   ```bash
   docker-compose -f docker-compose.caddy.yml up -d
   ```

Caddy will automatically obtain SSL certificates from Let's Encrypt.

## Configuration

### Environment Variables

The setup supports configuration via environment variables. See `.env.caddy.example` for all available options.

**Key Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `SOKETI_DEFAULT_APP_ID` | `app-id` | Application ID |
| `SOKETI_DEFAULT_APP_KEY` | `app-key` | Application key |
| `SOKETI_DEFAULT_APP_SECRET` | `app-secret` | Application secret |
| `SOKETI_DEFAULT_APP_MAX_CONNECTIONS` | `1000` | Max concurrent connections |
| `DOMAIN` | - | Domain for automatic HTTPS |
| `HTTP_PORT` | `80` | HTTP port |
| `HTTPS_PORT` | `443` | HTTPS port |
| `SOKETI_WS_PORT` | `6001` | Soketi WebSocket port |
| `SOKETI_METRICS_PORT` | `9601` | Soketi metrics port |

### Soketi Configuration

Configure soketi via environment variables in `docker-compose.caddy.yml`:

```yaml
environment:
  - SOKETI_DEFAULT_APP_ID=your-app-id
  - SOKETI_DEFAULT_APP_KEY=your-app-key
  - SOKETI_DEFAULT_APP_SECRET=your-app-secret
  - SOKETI_DEFAULT_APP_MAX_CONNECTIONS=1000
```

Or mount a custom `config.json`:

```yaml
volumes:
  - ./custom-config.json:/app/config.json:ro
```

### Caddy Configuration

The Caddyfile is mounted from the project root. Modify it to customize:

- Domain names
- Proxy paths
- Security headers
- SSL/TLS settings

### Port Configuration

Default ports:

- **80**: HTTP (Caddy)
- **443**: HTTPS/HTTP2 (Caddy)
- **443/udp**: HTTP/3 QUIC (Caddy)
- **6001**: Soketi WebSocket (exposed for direct access)
- **9601**: Soketi Metrics (exposed for monitoring)

To change ports, modify the `ports` section in `docker-compose.caddy.yml`.

## Custom SSL Certificates

To use custom SSL certificates instead of automatic Let's Encrypt:

1. **Create a certs directory**:
   ```bash
   mkdir -p certs
   ```

2. **Add your certificates**:
   ```
   certs/
   ├── cert.pem
   └── key.pem
   ```

3. **Uncomment the volume mount** in `docker-compose.caddy.yml`:
   ```yaml
   volumes:
     - ./certs:/etc/caddy/certs:ro
   ```

4. **Update Caddyfile** to use custom certificates:
   ```caddyfile
   your-domain.com {
       tls /etc/caddy/certs/cert.pem /etc/caddy/certs/key.pem
       # ... rest of config
   }
   ```

## Networking

Services communicate via the `soketi-network` bridge network:

- **Subnet**: 172.20.0.0/16
- **Service Discovery**: Services can reach each other by name (e.g., `soketi:6001`)

## Volumes

Persistent volumes for Caddy:

- **caddy_data**: Stores SSL certificates and ACME data
- **caddy_config**: Stores Caddy's configuration cache

To backup certificates:

```bash
docker run --rm -v caddy_data:/data -v $(pwd):/backup alpine tar czf /backup/caddy-data-backup.tar.gz -C /data .
```

To restore:

```bash
docker run --rm -v caddy_data:/data -v $(pwd):/backup alpine tar xzf /backup/caddy-data-backup.tar.gz -C /data
```

## Health Checks

Both services include health checks:

- **Soketi**: Checks `/health` endpoint every 30s
- **Caddy**: Checks `/health` endpoint every 30s

View health status:

```bash
docker-compose -f docker-compose.caddy.yml ps
```

## Monitoring

### View Logs

```bash
# All services
docker-compose -f docker-compose.caddy.yml logs -f

# Specific service
docker-compose -f docker-compose.caddy.yml logs -f caddy
docker-compose -f docker-compose.caddy.yml logs -f soketi
```

### Soketi Metrics

Access Prometheus metrics at: `http://localhost:9601/metrics`

### Caddy Admin API

Access Caddy's admin API at: `http://localhost:2019/`

## Scaling

To run multiple soketi instances:

```bash
docker-compose -f docker-compose.caddy.yml up -d --scale soketi=3
```

Update Caddyfile to load balance:

```caddyfile
:80 {
    reverse_proxy /app/* {
        to soketi:6001 soketi:6001 soketi:6001
        lb_policy round_robin
        # ... rest of config
    }
}
```

## Troubleshooting

### Check Service Status

```bash
docker-compose -f docker-compose.caddy.yml ps
```

### View Service Logs

```bash
docker-compose -f docker-compose.caddy.yml logs caddy
docker-compose -f docker-compose.caddy.yml logs soketi
```

### Test WebSocket Connection

```bash
# Install wscat if needed: npm install -g wscat
wscat -c ws://localhost/app/app-key
```

### Verify HTTP/3 Support

```bash
curl --http3 https://your-domain.com/health
```

### Common Issues

1. **Port already in use**:
   ```bash
   # Check what's using the port
   sudo lsof -i :80
   sudo lsof -i :443
   ```

2. **Certificate errors**:
   ```bash
   # Check Caddy logs
   docker-compose -f docker-compose.caddy.yml logs caddy
   
   # Verify domain DNS
   nslookup your-domain.com
   ```

3. **WebSocket connection fails**:
   - Verify soketi is running: `docker-compose -f docker-compose.caddy.yml ps`
   - Check soketi logs: `docker-compose -f docker-compose.caddy.yml logs soketi`
   - Test direct connection: `wscat -c ws://localhost:6001/app/app-key`

## Security Considerations

1. **Change default credentials**: Update `SOKETI_DEFAULT_APP_*` environment variables
2. **Use strong secrets**: Generate secure random strings for app secrets
3. **Firewall rules**: Only expose necessary ports (80, 443)
4. **Regular updates**: Keep Docker images updated
5. **Monitor logs**: Set up log aggregation and monitoring
6. **Rate limiting**: Configure rate limits in soketi configuration

## Production Checklist

- [ ] Copy `.env.caddy.example` to `.env` and configure
- [ ] Update domain name in Caddyfile
- [ ] Change default soketi credentials in `.env`
- [ ] Generate strong random secrets for `SOKETI_DEFAULT_APP_SECRET`
- [ ] Configure firewall rules
- [ ] Set up log monitoring
- [ ] Configure backup for Caddy volumes
- [ ] Test WebSocket connections
- [ ] Verify HTTPS and HTTP/3 work
- [ ] Set up health check monitoring
- [ ] Configure rate limiting
- [ ] Review security headers

## Client Configuration

### JavaScript/TypeScript (Pusher SDK)

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('your-app-key', {
  wsHost: 'your-domain.com',
  wsPort: 443,
  wssPort: 443,
  forceTLS: true,
  encrypted: true,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1', // Required but not used
});
```

### Development (HTTP)

```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 80,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws'],
  cluster: 'mt1',
});
```

## Additional Resources

- [Soketi Documentation](https://docs.soketi.app/)
- [Caddy Documentation](https://caddyserver.com/docs/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Pusher Protocol Documentation](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol/)

## License

This configuration is part of the soketi.rs project. See the main project LICENSE for details.
