# Reverse Proxy Setup for soketi.rs

> Complete guide for setting up Caddy and Nginx reverse proxies with HTTP/2, HTTP/3, and WebSocket support

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Caddy Setup](#caddy-setup)
- [Nginx Setup](#nginx-setup)
- [WebSocket Configuration](#websocket-configuration)
- [SSL/TLS Setup](#ssltls-setup)
- [Load Balancing](#load-balancing)
- [Monitoring and Logging](#monitoring-and-logging)
- [Performance Tuning](#performance-tuning)
- [Security Best Practices](#security-best-practices)
- [Cost Estimation](#cost-estimation)
- [Scaling Recommendations](#scaling-recommendations)
- [Troubleshooting](#troubleshooting)

## Overview

A reverse proxy sits between clients and your soketi.rs server, providing benefits such as:

- **SSL/TLS termination**: Handle HTTPS encryption at the proxy level
- **Load balancing**: Distribute traffic across multiple soketi.rs instances
- **HTTP/2 and HTTP/3**: Modern protocol support for better performance
- **Security**: Add security headers, rate limiting, and DDoS protection
- **Caching**: Cache static content and API responses
- **WebSocket support**: Proper WebSocket upgrade handling

This guide covers two popular reverse proxy solutions:
- **Caddy**: Modern, automatic HTTPS, simple configuration
- **Nginx**: High performance, widely used, extensive features

## Prerequisites

Before setting up a reverse proxy, ensure you have:

- A server with Docker installed (or native installation)
- A domain name pointing to your server
- soketi.rs server running (or ready to deploy)
- Basic understanding of networking and DNS
- Root or sudo access to your server

## Caddy Setup

Caddy is a modern web server with automatic HTTPS and simple configuration.

### Installation

**Option 1: Using Docker**

Create `Dockerfile.caddy`:

```dockerfile
FROM caddy:2.7-alpine

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Expose ports
EXPOSE 80 443 443/udp

# Run Caddy
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
```

**Option 2: Native Installation**

```bash
# Debian/Ubuntu
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy

# macOS
brew install caddy

# Verify installation
caddy version
```

### Caddyfile Configuration

Create `Caddyfile` for development:

```caddyfile
{
    # Global options
    auto_https off
    admin off
    
    # Enable HTTP/2 and HTTP/3
    servers {
        protocols h1 h2 h3
    }
}

# Development configuration (HTTP only)
:80 {
    # WebSocket proxy to soketi.rs
    reverse_proxy /app/* {
        to soketi:6001
        
        # Preserve headers
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
        
        # WebSocket support
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
    }
    
    # Health check endpoint
    respond /health 200 {
        body "OK"
    }
    
    # Logging
    log {
        output file /var/log/caddy/access.log
        format json
    }
}
```

Create `Caddyfile` for production:

```caddyfile
{
    # Global options
    email admin@example.com
    
    # Enable HTTP/2 and HTTP/3
    servers {
        protocols h1 h2 h3
    }
}

# Production configuration with automatic HTTPS
soketi.example.com {
    # WebSocket proxy to soketi.rs
    reverse_proxy /app/* {
        to soketi:6001
        
        # Preserve headers
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
        
        # WebSocket support
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
        
        # Health check
        health_uri /health
        health_interval 10s
        health_timeout 5s
    }
    
    # Security headers
    header {
        # HSTS
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        
        # Prevent clickjacking
        X-Frame-Options "DENY"
        
        # Prevent MIME sniffing
        X-Content-Type-Options "nosniff"
        
        # XSS protection
        X-XSS-Protection "1; mode=block"
        
        # Referrer policy
        Referrer-Policy "strict-origin-when-cross-origin"
        
        # Content Security Policy
        Content-Security-Policy "default-src 'self'; connect-src 'self' wss://soketi.example.com"
        
        # Remove server header
        -Server
    }
    
    # Rate limiting
    rate_limit {
        zone dynamic {
            key {remote_host}
            events 100
            window 1m
        }
    }
    
    # Logging
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 10
        }
        format json
    }
}

# Redirect www to non-www
www.soketi.example.com {
    redir https://soketi.example.com{uri} permanent
}
```

### Docker Compose for Caddy

Create `docker-compose.caddy.yml`:

```yaml
version: '3.8'

services:
  caddy:
    build:
      context: .
      dockerfile: Dockerfile.caddy
    container_name: caddy-proxy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"  # HTTP/3
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
      - caddy_logs:/var/log/caddy
    networks:
      - soketi-network
    depends_on:
      - soketi

  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    container_name: soketi-server
    restart: unless-stopped
    environment:
      - SOKETI_DEFAULT_APP_ID=app-id
      - SOKETI_DEFAULT_APP_KEY=app-key
      - SOKETI_DEFAULT_APP_SECRET=app-secret
      - SOKETI_HOST=0.0.0.0
      - SOKETI_PORT=6001
      - SOKETI_DEBUG=true
    networks:
      - soketi-network

volumes:
  caddy_data:
  caddy_config:
  caddy_logs:

networks:
  soketi-network:
    driver: bridge
```

### Running Caddy

```bash
# Using Docker Compose
docker-compose -f docker-compose.caddy.yml up -d

# View logs
docker-compose -f docker-compose.caddy.yml logs -f caddy

# Reload configuration
docker-compose -f docker-compose.caddy.yml exec caddy caddy reload --config /etc/caddy/Caddyfile

# Stop services
docker-compose -f docker-compose.caddy.yml down
```

## Nginx Setup

Nginx is a high-performance web server and reverse proxy with extensive features.

### Installation

**Option 1: Using Docker**

Create `Dockerfile.nginx`:

```dockerfile
FROM nginx:1.25-alpine

# Install dependencies for HTTP/3 (QUIC)
RUN apk add --no-cache \
    nginx-mod-http-quic \
    openssl

# Copy configuration files
COPY nginx.conf /etc/nginx/nginx.conf
COPY default.conf /etc/nginx/conf.d/default.conf

# Create log directory
RUN mkdir -p /var/log/nginx

# Expose ports
EXPOSE 80 443 443/udp

# Run Nginx
CMD ["nginx", "-g", "daemon off;"]
```

**Option 2: Native Installation**

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install -y nginx

# CentOS/RHEL
sudo yum install -y nginx

# macOS
brew install nginx

# Verify installation
nginx -v
```

### Nginx Configuration

Create `nginx.conf`:

```nginx
user nginx;
worker_processes auto;
worker_rlimit_nofile 65535;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 4096;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    log_format json escape=json '{'
        '"time":"$time_iso8601",'
        '"remote_addr":"$remote_addr",'
        '"request":"$request",'
        '"status":$status,'
        '"body_bytes_sent":$body_bytes_sent,'
        '"request_time":$request_time,'
        '"upstream_response_time":"$upstream_response_time"'
    '}';

    access_log /var/log/nginx/access.log json;

    # Performance optimizations
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    keepalive_requests 100;
    types_hash_max_size 2048;
    server_tokens off;

    # Buffer sizes
    client_body_buffer_size 128k;
    client_max_body_size 10m;
    client_header_buffer_size 1k;
    large_client_header_buffers 4 16k;

    # Timeouts
    client_body_timeout 12;
    client_header_timeout 12;
    send_timeout 10;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml text/javascript 
               application/json application/javascript application/xml+rss
               application/rss+xml font/truetype font/opentype 
               application/vnd.ms-fontobject image/svg+xml;
    gzip_disable "msie6";

    # Include virtual host configs
    include /etc/nginx/conf.d/*.conf;
}
```

Create `default.conf`:

```nginx
# Upstream configuration for soketi.rs
upstream soketi_backend {
    least_conn;
    server soketi:6001 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

# HTTP server (redirect to HTTPS)
server {
    listen 80;
    listen [::]:80;
    server_name soketi.example.com;
    
    # ACME challenge for Let's Encrypt
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }
    
    # Redirect all other traffic to HTTPS
    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS server with HTTP/2 and HTTP/3
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    listen 443 quic reuseport;
    listen [::]:443 quic reuseport;
    
    server_name soketi.example.com;
    
    # SSL certificates
    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    
    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_session_tickets off;
    
    # OCSP stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    ssl_trusted_certificate /etc/nginx/ssl/chain.pem;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;
    
    # HTTP/3 advertisement
    add_header Alt-Svc 'h3=":443"; ma=86400';
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; connect-src 'self' wss://soketi.example.com" always;
    
    # Remove server header
    more_clear_headers Server;
    
    # WebSocket proxy for soketi.rs
    location /app/ {
        proxy_pass http://soketi_backend;
        proxy_http_version 1.1;
        
        # WebSocket headers
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Standard proxy headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-Port $server_port;
        
        # Timeouts for WebSocket
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
        
        # Buffering
        proxy_buffering off;
        proxy_request_buffering off;
    }
    
    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
    
    # Metrics endpoint (optional)
    location /metrics {
        access_log off;
        allow 127.0.0.1;
        deny all;
        proxy_pass http://soketi_backend/metrics;
    }
}
```

### Docker Compose for Nginx

Create `docker-compose.nginx.yml`:

```yaml
version: '3.8'

services:
  nginx:
    build:
      context: .
      dockerfile: Dockerfile.nginx
    container_name: nginx-proxy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"  # HTTP/3
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./default.conf:/etc/nginx/conf.d/default.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
      - nginx_logs:/var/log/nginx
    networks:
      - soketi-network
    depends_on:
      - soketi

  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    container_name: soketi-server
    restart: unless-stopped
    environment:
      - SOKETI_DEFAULT_APP_ID=app-id
      - SOKETI_DEFAULT_APP_KEY=app-key
      - SOKETI_DEFAULT_APP_SECRET=app-secret
      - SOKETI_HOST=0.0.0.0
      - SOKETI_PORT=6001
      - SOKETI_DEBUG=true
    networks:
      - soketi-network

volumes:
  nginx_logs:

networks:
  soketi-network:
    driver: bridge
```

### Running Nginx

```bash
# Test configuration
nginx -t

# Using Docker Compose
docker-compose -f docker-compose.nginx.yml up -d

# View logs
docker-compose -f docker-compose.nginx.yml logs -f nginx

# Reload configuration
docker-compose -f docker-compose.nginx.yml exec nginx nginx -s reload

# Stop services
docker-compose -f docker-compose.nginx.yml down
```

## WebSocket Configuration

### WebSocket Headers

Both Caddy and Nginx need proper WebSocket headers:

**Required Headers:**
- `Upgrade: websocket`
- `Connection: Upgrade`
- `Host`: Original host header
- `X-Real-IP`: Client's real IP address
- `X-Forwarded-For`: Proxy chain
- `X-Forwarded-Proto`: Original protocol (http/https)

### WebSocket Timeouts

Configure appropriate timeouts for long-lived WebSocket connections:

**Caddy:**
```caddyfile
reverse_proxy /app/* {
    to soketi:6001
    
    # No timeout for WebSocket
    transport http {
        read_timeout 0
        write_timeout 0
    }
}
```

**Nginx:**
```nginx
location /app/ {
    proxy_pass http://soketi_backend;
    
    # Long timeouts for WebSocket
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

### Testing WebSocket Connection

```bash
# Using wscat
npm install -g wscat
wscat -c wss://soketi.example.com/app/app-key?protocol=7

# Using curl
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: test" \
  https://soketi.example.com/app/app-key
```

## SSL/TLS Setup

### Caddy SSL (Automatic)

Caddy automatically obtains and renews SSL certificates:

```caddyfile
# Just specify your domain
soketi.example.com {
    reverse_proxy /app/* soketi:6001
}
```

Caddy will:
1. Obtain certificate from Let's Encrypt
2. Configure HTTPS automatically
3. Renew certificates automatically
4. Redirect HTTP to HTTPS

### Nginx SSL (Manual)

**Option 1: Let's Encrypt with Certbot**

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d soketi.example.com

# Auto-renewal (cron job)
sudo certbot renew --dry-run
```

**Option 2: Manual Certificate**

```bash
# Generate self-signed certificate (development only)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/nginx/ssl/key.pem \
  -out /etc/nginx/ssl/cert.pem

# Use existing certificate
cp your-cert.pem /etc/nginx/ssl/cert.pem
cp your-key.pem /etc/nginx/ssl/key.pem
chmod 600 /etc/nginx/ssl/key.pem
```

### SSL Best Practices

1. **Use TLS 1.2 and 1.3 only**
2. **Strong cipher suites**
3. **Enable OCSP stapling**
4. **HTTP Strict Transport Security (HSTS)**
5. **Regular certificate renewal**

## Load Balancing

### Caddy Load Balancing

```caddyfile
soketi.example.com {
    reverse_proxy /app/* {
        # Multiple backends
        to soketi-1:6001 soketi-2:6001 soketi-3:6001
        
        # Load balancing policy
        lb_policy least_conn
        
        # Health checks
        health_uri /health
        health_interval 10s
        health_timeout 5s
        
        # Retry policy
        lb_try_duration 5s
        lb_try_interval 250ms
    }
}
```

### Nginx Load Balancing

```nginx
upstream soketi_backend {
    # Load balancing method
    least_conn;
    
    # Backend servers
    server soketi-1:6001 max_fails=3 fail_timeout=30s;
    server soketi-2:6001 max_fails=3 fail_timeout=30s;
    server soketi-3:6001 max_fails=3 fail_timeout=30s;
    
    # Connection pooling
    keepalive 32;
    keepalive_requests 100;
    keepalive_timeout 60s;
}
```

### Redis Adapter for Clustering

When load balancing, use Redis adapter:

```bash
# soketi.rs environment variables
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=redis-host
SOKETI_REDIS_PORT=6379
SOKETI_REDIS_PASSWORD=your-password
SOKETI_REDIS_DB=0
```

## Monitoring and Logging

### Caddy Monitoring

**Access Logs:**
```caddyfile
log {
    output file /var/log/caddy/access.log {
        roll_size 100mb
        roll_keep 10
    }
    format json
}
```

**Metrics Endpoint:**
```caddyfile
:2019 {
    metrics /metrics
}
```

### Nginx Monitoring

**Access Logs:**
```nginx
log_format json escape=json '{'
    '"time":"$time_iso8601",'
    '"remote_addr":"$remote_addr",'
    '"request":"$request",'
    '"status":$status,'
    '"body_bytes_sent":$body_bytes_sent,'
    '"request_time":$request_time'
'}';

access_log /var/log/nginx/access.log json;
```

**Stub Status Module:**
```nginx
location /nginx_status {
    stub_status;
    allow 127.0.0.1;
    deny all;
}
```

### Log Analysis

```bash
# View real-time logs
tail -f /var/log/nginx/access.log

# Count requests by status code
awk '{print $9}' /var/log/nginx/access.log | sort | uniq -c

# Top 10 IP addresses
awk '{print $1}' /var/log/nginx/access.log | sort | uniq -c | sort -rn | head -10
```

## Performance Tuning

### Caddy Performance

```caddyfile
{
    # Increase max connections
    servers {
        max_header_size 16kb
        read_timeout 30s
        write_timeout 30s
    }
}
```

### Nginx Performance

```nginx
# Worker processes
worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 4096;
    use epoll;
    multi_accept on;
}

http {
    # Keepalive
    keepalive_timeout 65;
    keepalive_requests 100;
    
    # Buffers
    client_body_buffer_size 128k;
    client_max_body_size 10m;
    
    # Caching
    open_file_cache max=10000 inactive=20s;
    open_file_cache_valid 30s;
    open_file_cache_min_uses 2;
}
```

## Security Best Practices

### Rate Limiting

**Caddy:**
```caddyfile
rate_limit {
    zone dynamic {
        key {remote_host}
        events 100
        window 1m
    }
}
```

**Nginx:**
```nginx
# Define rate limit zone
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

# Apply rate limit
location /app/ {
    limit_req zone=api burst=20 nodelay;
    proxy_pass http://soketi_backend;
}
```

### IP Whitelisting

**Caddy:**
```caddyfile
@allowed {
    remote_ip 192.168.1.0/24 10.0.0.0/8
}
handle @allowed {
    reverse_proxy soketi:6001
}
handle {
    respond "Forbidden" 403
}
```

**Nginx:**
```nginx
location /admin/ {
    allow 192.168.1.0/24;
    allow 10.0.0.0/8;
    deny all;
    proxy_pass http://soketi_backend;
}
```

### DDoS Protection

1. **Rate limiting**: Limit requests per IP
2. **Connection limits**: Limit concurrent connections
3. **Timeout configuration**: Prevent slowloris attacks
4. **Fail2ban**: Ban malicious IPs
5. **CloudFlare**: Use CDN with DDoS protection

## Cost Estimation

### Self-Hosted (VPS)

**DigitalOcean Droplet:**
- Basic: $6/month (1 GB RAM, 1 vCPU)
- Standard: $12/month (2 GB RAM, 1 vCPU)
- Performance: $24/month (4 GB RAM, 2 vCPU)

**AWS EC2:**
- t3.micro: ~$7.50/month (1 GB RAM, 2 vCPU)
- t3.small: ~$15/month (2 GB RAM, 2 vCPU)
- t3.medium: ~$30/month (4 GB RAM, 2 vCPU)

**Hetzner Cloud:**
- CX11: €3.79/month (2 GB RAM, 1 vCPU)
- CX21: €5.83/month (4 GB RAM, 2 vCPU)
- CX31: €10.59/month (8 GB RAM, 2 vCPU)

### Managed Solutions

**Cloudflare (CDN + DDoS):**
- Free: Basic DDoS protection
- Pro: $20/month
- Business: $200/month

**AWS Application Load Balancer:**
- ~$16/month + data transfer costs

### Total Cost Estimate

**Small Project:**
- VPS: $6/month
- Domain: $12/year (~$1/month)
- **Total**: ~$7/month

**Production:**
- VPS: $24/month
- Cloudflare Pro: $20/month
- Monitoring: $10/month
- **Total**: ~$54/month

## Scaling Recommendations

### Vertical Scaling

Upgrade server resources:
- Increase RAM for more connections
- Add CPU cores for better performance
- Use SSD storage for faster I/O

### Horizontal Scaling

Deploy multiple instances:
1. **Multiple soketi.rs instances** with Redis adapter
2. **Load balancer** (Nginx/Caddy) distributing traffic
3. **Redis cluster** for high availability
4. **Database replication** if using persistent storage

### Auto-Scaling

Use container orchestration:
- **Kubernetes**: Auto-scale based on metrics
- **Docker Swarm**: Simple orchestration
- **Nomad**: Lightweight alternative

## Troubleshooting

### Common Issues

**1. 502 Bad Gateway**

```
nginx: [error] connect() failed (111: Connection refused)
```

**Solution:**
- Check if soketi.rs is running
- Verify upstream configuration
- Check firewall rules

**2. WebSocket Connection Failed**

```
WebSocket connection to 'wss://...' failed
```

**Solution:**
- Verify WebSocket headers are set
- Check timeout configuration
- Ensure SSL certificate is valid

**3. SSL Certificate Error**

```
SSL certificate problem: unable to get local issuer certificate
```

**Solution:**
- Verify certificate chain is complete
- Check certificate expiration
- Ensure proper file permissions

**4. High CPU Usage**

**Solution:**
- Increase worker processes
- Enable caching
- Optimize upstream connections
- Use connection pooling

**5. Memory Leaks**

**Solution:**
- Monitor logs for errors
- Check for connection leaks
- Restart services periodically
- Update to latest versions

### Debug Commands

```bash
# Test Nginx configuration
nginx -t

# Reload Nginx
nginx -s reload

# Check Caddy configuration
caddy validate --config Caddyfile

# Reload Caddy
caddy reload --config Caddyfile

# Check open connections
netstat -an | grep :443 | wc -l

# Monitor resource usage
htop

# Check logs
tail -f /var/log/nginx/error.log
tail -f /var/log/caddy/error.log
```

## Related Resources

- [Caddy Documentation](https://caddyserver.com/docs/)
- [Nginx Documentation](https://nginx.org/en/docs/)
- [soketi.rs Documentation](https://docs.soketi.app)
- [Let's Encrypt](https://letsencrypt.org/)
- [Vercel Deployment Guide](./vercel.md)
- [Netlify Deployment Guide](./netlify.md)
- [Getting Started Guide](../getting-started.md)
- [API Reference](../api-reference.md)
