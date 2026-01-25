# Deployment Files

This directory contains all deployment-related files for Soketi.rs.

## Directory Structure

```
deployment/
├── docker/              # Standard Docker deployment
│   ├── Dockerfile
│   └── docker-compose.yml
├── nginx/               # Nginx reverse proxy setup
│   ├── Dockerfile.nginx
│   ├── nginx.conf
│   ├── default.conf
│   ├── docker-compose.nginx.yml
│   └── docker-compose.nginx.README.md
└── caddy/               # Caddy reverse proxy setup
    ├── Dockerfile.caddy
    ├── Caddyfile
    ├── docker-compose.caddy.yml
    └── docker-compose.caddy.README.md
```

## Quick Start

### Standard Docker Deployment

```bash
cd deployment/docker
docker-compose up -d
```

### With Nginx Reverse Proxy

```bash
cd deployment/nginx
docker-compose -f docker-compose.nginx.yml up -d
```

### With Caddy Reverse Proxy

```bash
cd deployment/caddy
docker-compose -f docker-compose.caddy.yml up -d
```

## Documentation

For detailed deployment guides, see:

- [Docker Guide (EN)](../docs/en/docker-guide.md) | [Docker Rehberi (TR)](../docs/tr/docker-rehberi.md)
- [Docker Deployment (EN)](../docs/en/docker-deployment.md) | [Docker Deployment (TR)](../docs/tr/docker-deployment.md)
- [Deployment Guide (EN)](../docs/en/deployment.md)

## Docker Hub

Pre-built images are available on Docker Hub:

**Repository**: [ferdiunal/soketi-rs](https://hub.docker.com/r/funal/soketi-rs)

```bash
docker pull funal/soketi-rs:latest
```

## Configuration

Each deployment directory includes example configuration files:

- `config.example.json` - Basic development configuration
- `config.production.json` - Production-ready configuration with Redis and PostgreSQL

For detailed configuration guide, see [CONFIGURATION.md](CONFIGURATION.md).

### Quick Setup

```bash
# Navigate to your chosen deployment directory
cd deployment/docker  # or nginx, or caddy

# Copy example config
cp config.example.json config.json

# Edit configuration
nano config.json
```

## Support

- **GitHub Issues**: [Report a bug](https://github.com/ferdiunal/soketi.rs/issues)
- **GitHub Discussions**: [Ask questions](https://github.com/ferdiunal/soketi.rs/discussions)
- **Documentation**: [Full docs](../docs/)
