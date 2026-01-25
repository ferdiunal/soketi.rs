# Nginx Reverse Proxy Deployment

This directory contains Docker deployment with Nginx as a reverse proxy.

## Features

- SSL/TLS termination
- WebSocket support
- Load balancing
- Static file serving
- Automatic HTTPS redirect

## Quick Start

```bash
# Start all services
docker-compose -f docker-compose.nginx.yml up -d

# View logs
docker-compose -f docker-compose.nginx.yml logs -f

# Stop services
docker-compose -f docker-compose.nginx.yml down
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

Copy `.env.nginx.example` to `.env` and configure:

```bash
cp .env.nginx.example .env
```

Edit `.env`:
```env
DOMAIN=your-domain.com
SSL_CERT_PATH=/path/to/cert.pem
SSL_KEY_PATH=/path/to/key.pem
```

### Nginx Configuration

The `nginx.conf` file contains the main Nginx configuration:

- WebSocket proxy settings
- SSL/TLS configuration
- Upstream server definitions
- Health checks

### Default Configuration

The `default.conf` file contains the server block configuration:

- HTTP to HTTPS redirect
- SSL certificate paths
- Proxy headers
- WebSocket upgrade handling

## SSL/TLS Setup

### Using Let's Encrypt

```bash
# Install certbot
sudo apt-get install certbot

# Get certificate
sudo certbot certonly --standalone -d your-domain.com

# Update .env with certificate paths
SSL_CERT_PATH=/etc/letsencrypt/live/your-domain.com/fullchain.pem
SSL_KEY_PATH=/etc/letsencrypt/live/your-domain.com/privkey.pem
```

### Using Self-Signed Certificate (Development)

```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx-selfsigned.key \
  -out nginx-selfsigned.crt
```

## Ports

- **80** - HTTP (redirects to HTTPS)
- **443** - HTTPS
- **6001** - Soketi WebSocket (internal)
- **9601** - Metrics (internal)

## Load Balancing

To scale Soketi instances, update `docker-compose.nginx.yml`:

```yaml
services:
  soketi:
    deploy:
      replicas: 3
```

Nginx will automatically load balance across all instances.

## Documentation

- [Reverse Proxy Guide (EN)](../../docs/en/deployment/reverse-proxy.md)
- [Docker Deployment (EN)](../../docs/en/docker-deployment.md)

## Troubleshooting

### WebSocket Connection Issues

Ensure these headers are set in `nginx.conf`:
```nginx
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
```

### SSL Certificate Errors

Verify certificate paths in `.env` and ensure files are readable by Nginx.

## Support

- [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)
- [Nginx Documentation](https://nginx.org/en/docs/)
