# Caddy Reverse Proxy Deployment

This directory contains Docker deployment with Caddy as a reverse proxy.

## Features

- Automatic HTTPS with Let's Encrypt
- WebSocket support
- HTTP/2 and HTTP/3 support
- Zero-config SSL
- Automatic certificate renewal

## Quick Start

```bash
# Start all services
docker-compose -f docker-compose.caddy.yml up -d

# View logs
docker-compose -f docker-compose.caddy.yml logs -f

# Stop services
docker-compose -f docker-compose.caddy.yml down
```

## Configuration

### Soketi Configuration

Copy and configure the Soketi config file:

```bash
# For development
cp config.example.json config.json

# For production
cp config.production.json config.json
```

Edit `config.json` with your settings.

### Environment Variables

Copy `.env.caddy.example` to `.env` and configure:

```bash
cp .env.caddy.example .env
```

Edit `.env`:
```env
DOMAIN=your-domain.com
EMAIL=your-email@example.com
```

### Caddyfile

The `Caddyfile` contains the Caddy configuration:

- Automatic HTTPS
- WebSocket proxy
- Reverse proxy settings
- Header configuration

## Automatic HTTPS

Caddy automatically obtains and renews SSL certificates from Let's Encrypt:

1. Set your domain in `.env`
2. Ensure DNS points to your server
3. Start Caddy - it handles the rest!

No manual certificate management needed.

## Ports

- **80** - HTTP (auto-redirects to HTTPS)
- **443** - HTTPS
- **6001** - Soketi WebSocket (internal)
- **9601** - Metrics (internal)

## Custom Domain

Update `Caddyfile` with your domain:

```caddyfile
your-domain.com {
    reverse_proxy soketi:6001
}
```

## Load Balancing

To scale Soketi instances, update `docker-compose.caddy.yml`:

```yaml
services:
  soketi:
    deploy:
      replicas: 3
```

Caddy will automatically load balance across all instances.

## HTTP/3 Support

Caddy supports HTTP/3 (QUIC) by default. No additional configuration needed!

## Development Mode

For local development without a domain:

```caddyfile
:80 {
    reverse_proxy soketi:6001
}
```

This disables automatic HTTPS for local testing.

## Documentation

- [Reverse Proxy Guide (EN)](../../docs/en/deployment/reverse-proxy.md)
- [Docker Deployment (EN)](../../docs/en/docker-deployment.md)
- [Caddy Documentation](https://caddyserver.com/docs/)

## Advantages Over Nginx

- **Zero-config HTTPS** - Automatic certificate management
- **Simpler configuration** - More readable Caddyfile syntax
- **HTTP/3 support** - Built-in QUIC protocol support
- **Automatic renewal** - No cron jobs needed

## Troubleshooting

### Certificate Issues

Ensure:
1. Domain DNS points to your server
2. Ports 80 and 443 are accessible
3. Email is set in `.env`

### WebSocket Connection Issues

Caddy handles WebSocket upgrades automatically. If issues persist, check:
- Firewall rules
- Domain configuration
- Soketi logs

## Support

- [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)
- [Caddy Community](https://caddy.community/)
