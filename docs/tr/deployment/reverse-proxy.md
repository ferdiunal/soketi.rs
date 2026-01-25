# soketi.rs için Reverse Proxy Kurulumu

> HTTP/2, HTTP/3 ve WebSocket desteği ile Caddy ve Nginx reverse proxy kurulumu için eksiksiz kılavuz

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Gereksinimler](#ön-gereksinimler)
- [Caddy Kurulumu](#caddy-kurulumu)
- [Nginx Kurulumu](#nginx-kurulumu)
- [WebSocket Yapılandırması](#websocket-yapılandırması)
- [SSL/TLS Kurulumu](#ssltls-kurulumu)
- [Load Balancing](#load-balancing)
- [İzleme ve Loglama](#izleme-ve-loglama)
- [Performans Ayarlama](#performans-ayarlama)
- [Güvenlik En İyi Uygulamaları](#güvenlik-en-iyi-uygulamaları)
- [Maliyet Tahmini](#maliyet-tahmini)
- [Ölçeklendirme Önerileri](#ölçeklendirme-önerileri)
- [Sorun Giderme](#sorun-giderme)

## Genel Bakış

Bir reverse proxy, istemciler ile soketi.rs sunucunuz arasında yer alır ve şu faydaları sağlar:

- **SSL/TLS sonlandırma**: HTTPS şifrelemesini proxy seviyesinde yönetin
- **Load balancing**: Trafiği birden fazla soketi.rs instance'ı arasında dağıtın
- **HTTP/2 ve HTTP/3**: Daha iyi performans için modern protokol desteği
- **Güvenlik**: Güvenlik header'ları, rate limiting ve DDoS koruması ekleyin
- **Önbellekleme**: Statik içeriği ve API yanıtlarını önbelleğe alın
- **WebSocket desteği**: Uygun WebSocket upgrade işleme

Bu kılavuz iki popüler reverse proxy çözümünü kapsar:
- **Caddy**: Modern, otomatik HTTPS, basit yapılandırma
- **Nginx**: Yüksek performans, yaygın kullanım, kapsamlı özellikler

## Ön Gereksinimler

Bir reverse proxy kurmadan önce, aşağıdakilere sahip olduğunuzdan emin olun:

- Docker kurulu bir sunucu (veya native kurulum)
- Sunucunuza işaret eden bir domain adı
- Çalışan soketi.rs sunucusu (veya dağıtmaya hazır)
- Ağ ve DNS hakkında temel bilgi
- Sunucunuza root veya sudo erişimi

## Caddy Kurulumu

Caddy, otomatik HTTPS ve basit yapılandırma ile modern bir web sunucusudur.

### Kurulum

**Seçenek 1: Docker Kullanarak**

`Dockerfile.caddy` oluşturun:

```dockerfile
FROM caddy:2.7-alpine

# Caddyfile'ı kopyala
COPY Caddyfile /etc/caddy/Caddyfile

# Port'ları aç
EXPOSE 80 443 443/udp

# Caddy'yi çalıştır
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
```

**Seçenek 2: Native Kurulum**

```bash
# Debian/Ubuntu
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy

# macOS
brew install caddy

# Kurulumu doğrula
caddy version
```

### Caddyfile Yapılandırması

Geliştirme için `Caddyfile` oluşturun:

```caddyfile
{
    # Global seçenekler
    auto_https off
    admin off
    
    # HTTP/2 ve HTTP/3'ü etkinleştir
    servers {
        protocols h1 h2 h3
    }
}

# Geliştirme yapılandırması (sadece HTTP)
:80 {
    # soketi.rs'ye WebSocket proxy
    reverse_proxy /app/* {
        to soketi:6001
        
        # Header'ları koru
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
        
        # WebSocket desteği
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
    }
    
    # Health check endpoint
    respond /health 200 {
        body "OK"
    }
    
    # Loglama
    log {
        output file /var/log/caddy/access.log
        format json
    }
}
```

Production için `Caddyfile` oluşturun:

```caddyfile
{
    # Global seçenekler
    email admin@example.com
    
    # HTTP/2 ve HTTP/3'ü etkinleştir
    servers {
        protocols h1 h2 h3
    }
}

# Otomatik HTTPS ile production yapılandırması
soketi.example.com {
    # soketi.rs'ye WebSocket proxy
    reverse_proxy /app/* {
        to soketi:6001
        
        # Header'ları koru
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
        
        # WebSocket desteği
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
        
        # Health check
        health_uri /health
        health_interval 10s
        health_timeout 5s
    }
    
    # Güvenlik header'ları
    header {
        # HSTS
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        
        # Clickjacking'i önle
        X-Frame-Options "DENY"
        
        # MIME sniffing'i önle
        X-Content-Type-Options "nosniff"
        
        # XSS koruması
        X-XSS-Protection "1; mode=block"
        
        # Referrer policy
        Referrer-Policy "strict-origin-when-cross-origin"
        
        # Content Security Policy
        Content-Security-Policy "default-src 'self'; connect-src 'self' wss://soketi.example.com"
        
        # Server header'ını kaldır
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
    
    # Loglama
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 10
        }
        format json
    }
}

# www'yi www olmayan'a yönlendir
www.soketi.example.com {
    redir https://soketi.example.com{uri} permanent
}
```

### Caddy için Docker Compose

`docker-compose.caddy.yml` oluşturun:

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

### Caddy'yi Çalıştırma

```bash
# Docker Compose kullanarak
docker-compose -f docker-compose.caddy.yml up -d

# Logları görüntüle
docker-compose -f docker-compose.caddy.yml logs -f caddy

# Yapılandırmayı yeniden yükle
docker-compose -f docker-compose.caddy.yml exec caddy caddy reload --config /etc/caddy/Caddyfile

# Servisleri durdur
docker-compose -f docker-compose.caddy.yml down
```

## Nginx Kurulumu

Nginx, kapsamlı özelliklerle yüksek performanslı bir web sunucusu ve reverse proxy'dir.

### Kurulum

**Seçenek 1: Docker Kullanarak**

`Dockerfile.nginx` oluşturun:

```dockerfile
FROM nginx:1.25-alpine

# HTTP/3 (QUIC) için bağımlılıkları yükle
RUN apk add --no-cache \
    nginx-mod-http-quic \
    openssl

# Yapılandırma dosyalarını kopyala
COPY nginx.conf /etc/nginx/nginx.conf
COPY default.conf /etc/nginx/conf.d/default.conf

# Log dizini oluştur
RUN mkdir -p /var/log/nginx

# Port'ları aç
EXPOSE 80 443 443/udp

# Nginx'i çalıştır
CMD ["nginx", "-g", "daemon off;"]
```

**Seçenek 2: Native Kurulum**

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install -y nginx

# CentOS/RHEL
sudo yum install -y nginx

# macOS
brew install nginx

# Kurulumu doğrula
nginx -v
```

### Nginx Yapılandırması

`nginx.conf` oluşturun:

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

    # Loglama
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

    # Performans optimizasyonları
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    keepalive_requests 100;
    types_hash_max_size 2048;
    server_tokens off;

    # Buffer boyutları
    client_body_buffer_size 128k;
    client_max_body_size 10m;
    client_header_buffer_size 1k;
    large_client_header_buffers 4 16k;

    # Timeout'lar
    client_body_timeout 12;
    client_header_timeout 12;
    send_timeout 10;

    # Gzip sıkıştırma
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml text/javascript 
               application/json application/javascript application/xml+rss
               application/rss+xml font/truetype font/opentype 
               application/vnd.ms-fontobject image/svg+xml;
    gzip_disable "msie6";

    # Virtual host yapılandırmalarını dahil et
    include /etc/nginx/conf.d/*.conf;
}
```

`default.conf` oluşturun:

```nginx
# soketi.rs için upstream yapılandırması
upstream soketi_backend {
    least_conn;
    server soketi:6001 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

# HTTP sunucusu (HTTPS'e yönlendir)
server {
    listen 80;
    listen [::]:80;
    server_name soketi.example.com;
    
    # Let's Encrypt için ACME challenge
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }
    
    # Diğer tüm trafiği HTTPS'e yönlendir
    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTP/2 ve HTTP/3 ile HTTPS sunucusu
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    listen 443 quic reuseport;
    listen [::]:443 quic reuseport;
    
    server_name soketi.example.com;
    
    # SSL sertifikaları
    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    
    # SSL yapılandırması
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
    
    # HTTP/3 reklamı
    add_header Alt-Svc 'h3=":443"; ma=86400';
    
    # Güvenlik header'ları
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; connect-src 'self' wss://soketi.example.com" always;
    
    # Server header'ını kaldır
    more_clear_headers Server;
    
    # soketi.rs için WebSocket proxy
    location /app/ {
        proxy_pass http://soketi_backend;
        proxy_http_version 1.1;
        
        # WebSocket header'ları
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Standart proxy header'ları
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-Port $server_port;
        
        # WebSocket için timeout'lar
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
    
    # Metrics endpoint (opsiyonel)
    location /metrics {
        access_log off;
        allow 127.0.0.1;
        deny all;
        proxy_pass http://soketi_backend/metrics;
    }
}
```

### Nginx için Docker Compose

`docker-compose.nginx.yml` oluşturun:

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

### Nginx'i Çalıştırma

```bash
# Yapılandırmayı test et
nginx -t

# Docker Compose kullanarak
docker-compose -f docker-compose.nginx.yml up -d

# Logları görüntüle
docker-compose -f docker-compose.nginx.yml logs -f nginx

# Yapılandırmayı yeniden yükle
docker-compose -f docker-compose.nginx.yml exec nginx nginx -s reload

# Servisleri durdur
docker-compose -f docker-compose.nginx.yml down
```

## WebSocket Yapılandırması

### WebSocket Header'ları

Hem Caddy hem de Nginx'in uygun WebSocket header'larına ihtiyacı vardır:

**Gerekli Header'lar:**
- `Upgrade: websocket`
- `Connection: Upgrade`
- `Host`: Orijinal host header
- `X-Real-IP`: İstemcinin gerçek IP adresi
- `X-Forwarded-For`: Proxy zinciri
- `X-Forwarded-Proto`: Orijinal protokol (http/https)

### WebSocket Timeout'ları

Uzun süreli WebSocket bağlantıları için uygun timeout'ları yapılandırın:

**Caddy:**
```caddyfile
reverse_proxy /app/* {
    to soketi:6001
    
    # WebSocket için timeout yok
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
    
    # WebSocket için uzun timeout'lar
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

### WebSocket Bağlantısını Test Etme

```bash
# wscat kullanarak
npm install -g wscat
wscat -c wss://soketi.example.com/app/app-key?protocol=7

# curl kullanarak
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: test" \
  https://soketi.example.com/app/app-key
```

## SSL/TLS Kurulumu

### Caddy SSL (Otomatik)

Caddy otomatik olarak SSL sertifikaları alır ve yeniler:

```caddyfile
# Sadece domain'inizi belirtin
soketi.example.com {
    reverse_proxy /app/* soketi:6001
}
```

Caddy şunları yapacaktır:
1. Let's Encrypt'ten sertifika alır
2. HTTPS'i otomatik yapılandırır
3. Sertifikaları otomatik yeniler
4. HTTP'yi HTTPS'e yönlendirir

### Nginx SSL (Manuel)

**Seçenek 1: Certbot ile Let's Encrypt**

```bash
# Certbot'u yükle
sudo apt install certbot python3-certbot-nginx

# Sertifika al
sudo certbot --nginx -d soketi.example.com

# Otomatik yenileme (cron job)
sudo certbot renew --dry-run
```

**Seçenek 2: Manuel Sertifika**

```bash
# Self-signed sertifika oluştur (sadece geliştirme için)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/nginx/ssl/key.pem \
  -out /etc/nginx/ssl/cert.pem

# Mevcut sertifikayı kullan
cp your-cert.pem /etc/nginx/ssl/cert.pem
cp your-key.pem /etc/nginx/ssl/key.pem
chmod 600 /etc/nginx/ssl/key.pem
```

### SSL En İyi Uygulamaları

1. **Sadece TLS 1.2 ve 1.3 kullanın**
2. **Güçlü cipher suite'ler**
3. **OCSP stapling'i etkinleştirin**
4. **HTTP Strict Transport Security (HSTS)**
5. **Düzenli sertifika yenileme**

## Load Balancing

### Caddy Load Balancing

```caddyfile
soketi.example.com {
    reverse_proxy /app/* {
        # Birden fazla backend
        to soketi-1:6001 soketi-2:6001 soketi-3:6001
        
        # Load balancing politikası
        lb_policy least_conn
        
        # Health check'ler
        health_uri /health
        health_interval 10s
        health_timeout 5s
        
        # Retry politikası
        lb_try_duration 5s
        lb_try_interval 250ms
    }
}
```

### Nginx Load Balancing

```nginx
upstream soketi_backend {
    # Load balancing yöntemi
    least_conn;
    
    # Backend sunucuları
    server soketi-1:6001 max_fails=3 fail_timeout=30s;
    server soketi-2:6001 max_fails=3 fail_timeout=30s;
    server soketi-3:6001 max_fails=3 fail_timeout=30s;
    
    # Connection pooling
    keepalive 32;
    keepalive_requests 100;
    keepalive_timeout 60s;
}
```

### Clustering için Redis Adapter

Load balancing yaparken, Redis adapter kullanın:

```bash
# soketi.rs ortam değişkenleri
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=redis-host
SOKETI_REDIS_PORT=6379
SOKETI_REDIS_PASSWORD=your-password
SOKETI_REDIS_DB=0
```

## İzleme ve Loglama

### Caddy İzleme

**Access Logları:**
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

### Nginx İzleme

**Access Logları:**
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

**Stub Status Modülü:**
```nginx
location /nginx_status {
    stub_status;
    allow 127.0.0.1;
    deny all;
}
```

### Log Analizi

```bash
# Gerçek zamanlı logları görüntüle
tail -f /var/log/nginx/access.log

# Durum koduna göre istekleri say
awk '{print $9}' /var/log/nginx/access.log | sort | uniq -c

# En çok istek yapan 10 IP adresi
awk '{print $1}' /var/log/nginx/access.log | sort | uniq -c | sort -rn | head -10
```

## Performans Ayarlama

### Caddy Performans

```caddyfile
{
    # Maksimum bağlantıları artır
    servers {
        max_header_size 16kb
        read_timeout 30s
        write_timeout 30s
    }
}
```

### Nginx Performans

```nginx
# Worker process'leri
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
    
    # Buffer'lar
    client_body_buffer_size 128k;
    client_max_body_size 10m;
    
    # Önbellekleme
    open_file_cache max=10000 inactive=20s;
    open_file_cache_valid 30s;
    open_file_cache_min_uses 2;
}
```

## Güvenlik En İyi Uygulamaları

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
# Rate limit zone tanımla
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

# Rate limit uygula
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

### DDoS Koruması

1. **Rate limiting**: IP başına istekleri sınırla
2. **Connection limitleri**: Eşzamanlı bağlantıları sınırla
3. **Timeout yapılandırması**: Slowloris saldırılarını önle
4. **Fail2ban**: Kötü niyetli IP'leri yasakla
5. **CloudFlare**: DDoS koruması ile CDN kullan

## Maliyet Tahmini

### Self-Hosted (VPS)

**DigitalOcean Droplet:**
- Basic: $6/ay (1 GB RAM, 1 vCPU)
- Standard: $12/ay (2 GB RAM, 1 vCPU)
- Performance: $24/ay (4 GB RAM, 2 vCPU)

**AWS EC2:**
- t3.micro: ~$7.50/ay (1 GB RAM, 2 vCPU)
- t3.small: ~$15/ay (2 GB RAM, 2 vCPU)
- t3.medium: ~$30/ay (4 GB RAM, 2 vCPU)

**Hetzner Cloud:**
- CX11: €3.79/ay (2 GB RAM, 1 vCPU)
- CX21: €5.83/ay (4 GB RAM, 2 vCPU)
- CX31: €10.59/ay (8 GB RAM, 2 vCPU)

### Yönetilen Çözümler

**Cloudflare (CDN + DDoS):**
- Free: Temel DDoS koruması
- Pro: $20/ay
- Business: $200/ay

**AWS Application Load Balancer:**
- ~$16/ay + veri transfer maliyetleri

### Toplam Maliyet Tahmini

**Küçük Proje:**
- VPS: $6/ay
- Domain: $12/yıl (~$1/ay)
- **Toplam**: ~$7/ay

**Production:**
- VPS: $24/ay
- Cloudflare Pro: $20/ay
- İzleme: $10/ay
- **Toplam**: ~$54/ay

## Ölçeklendirme Önerileri

### Dikey Ölçeklendirme

Sunucu kaynaklarını yükseltin:
- Daha fazla bağlantı için RAM'i artırın
- Daha iyi performans için CPU çekirdekleri ekleyin
- Daha hızlı I/O için SSD depolama kullanın

### Yatay Ölçeklendirme

Birden fazla instance dağıtın:
1. **Birden fazla soketi.rs instance'ı** Redis adapter ile
2. **Load balancer** (Nginx/Caddy) trafiği dağıtıyor
3. **Redis cluster** yüksek erişilebilirlik için
4. **Veritabanı replikasyonu** kalıcı depolama kullanıyorsanız

### Otomatik Ölçeklendirme

Konteyner orkestrasyon kullanın:
- **Kubernetes**: Metriklere göre otomatik ölçeklendirme
- **Docker Swarm**: Basit orkestrasyon
- **Nomad**: Hafif alternatif

## Sorun Giderme

### Yaygın Sorunlar

**1. 502 Bad Gateway**

```
nginx: [error] connect() failed (111: Connection refused)
```

**Çözüm:**
- soketi.rs'nin çalıştığını kontrol edin
- Upstream yapılandırmasını doğrulayın
- Firewall kurallarını kontrol edin

**2. WebSocket Bağlantısı Başarısız**

```
WebSocket connection to 'wss://...' failed
```

**Çözüm:**
- WebSocket header'larının ayarlandığını doğrulayın
- Timeout yapılandırmasını kontrol edin
- SSL sertifikasının geçerli olduğundan emin olun

**3. SSL Sertifika Hatası**

```
SSL certificate problem: unable to get local issuer certificate
```

**Çözüm:**
- Sertifika zincirinin tam olduğunu doğrulayın
- Sertifika süresini kontrol edin
- Uygun dosya izinlerini sağlayın

**4. Yüksek CPU Kullanımı**

**Çözüm:**
- Worker process'leri artırın
- Önbelleklemeyi etkinleştirin
- Upstream bağlantılarını optimize edin
- Connection pooling kullanın

**5. Bellek Sızıntıları**

**Çözüm:**
- Hatalar için logları izleyin
- Bağlantı sızıntılarını kontrol edin
- Servisleri periyodik olarak yeniden başlatın
- En son versiyonlara güncelleyin

### Debug Komutları

```bash
# Nginx yapılandırmasını test et
nginx -t

# Nginx'i yeniden yükle
nginx -s reload

# Caddy yapılandırmasını kontrol et
caddy validate --config Caddyfile

# Caddy'yi yeniden yükle
caddy reload --config Caddyfile

# Açık bağlantıları kontrol et
netstat -an | grep :443 | wc -l

# Kaynak kullanımını izle
htop

# Logları kontrol et
tail -f /var/log/nginx/error.log
tail -f /var/log/caddy/error.log
```

## İlgili Kaynaklar

- [Caddy Dokümantasyonu](https://caddyserver.com/docs/)
- [Nginx Dokümantasyonu](https://nginx.org/en/docs/)
- [soketi.rs Dokümantasyonu](https://docs.soketi.app)
- [Let's Encrypt](https://letsencrypt.org/)
- [Vercel Dağıtım Kılavuzu](./vercel.md)
- [Netlify Dağıtım Kılavuzu](./netlify.md)
- [Başlangıç Kılavuzu](../baslangic.md)
- [API Referansı](../api-referans.md)
