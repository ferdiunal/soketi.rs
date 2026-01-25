# Nginx Reverse Proxy Docker Compose Setup

Bu dosya, Soketi WebSocket sunucusu için Nginx reverse proxy yapılandırmasını içeren Docker Compose kurulumunu sağlar.

## Özellikler

- **HTTP/2 Desteği**: Modern HTTP/2 protokolü ile gelişmiş performans
- **HTTP/3 (QUIC) Desteği**: En yeni HTTP/3 protokolü ile ultra-düşük gecikme
- **SSL/TLS Şifreleme**: Güvenli bağlantılar için SSL/TLS desteği
- **WebSocket Proxy**: Soketi sunucusuna WebSocket bağlantıları için proxy
- **Güvenlik Başlıkları**: HSTS, X-Frame-Options, X-Content-Type-Options ve daha fazlası
- **Health Check**: Otomatik sağlık kontrolü ve servis izleme
- **Load Balancing**: Birden fazla Soketi instance için hazır

## Gereksinimler

- Docker 20.10+
- Docker Compose 2.0+
- SSL/TLS sertifikaları (production için)

## Hızlı Başlangıç

### 1. SSL Sertifikalarını Hazırlayın

#### Development için (Self-Signed Sertifika)

```bash
# Sertifika dizini oluştur
mkdir -p certs

# Self-signed sertifika oluştur
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout certs/key.pem \
  -out certs/cert.pem \
  -subj "/C=TR/ST=Istanbul/L=Istanbul/O=Development/CN=soketi.example.com"
```

#### Production için

Production ortamında Let's Encrypt veya başka bir CA'dan alınmış geçerli sertifikalar kullanın:

```bash
# Sertifikaları certs/ dizinine kopyalayın
cp /path/to/your/fullchain.pem certs/cert.pem
cp /path/to/your/privkey.pem certs/key.pem
```

### 2. Environment Değişkenlerini Yapılandırın

`.env` dosyası oluşturun:

```bash
# Soketi Configuration
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_DEBUG=false

# Soketi App Configuration
SOKETI_DEFAULT_APP_ID=app-id
SOKETI_DEFAULT_APP_KEY=app-key
SOKETI_DEFAULT_APP_SECRET=app-secret

# Nginx Configuration
HTTP_PORT=80
HTTPS_PORT=443
HTTP3_PORT=443
NGINX_HOST=soketi.example.com

# SSL Certificate Paths
SSL_CERT_PATH=./certs/cert.pem
SSL_KEY_PATH=./certs/key.pem

# Network Configuration
SOKETI_NETWORK_SUBNET=172.20.0.0/16
```

### 3. Servisleri Başlatın

```bash
# Servisleri arka planda başlat
docker-compose -f docker-compose.nginx.yml up -d

# Logları izle
docker-compose -f docker-compose.nginx.yml logs -f

# Servis durumunu kontrol et
docker-compose -f docker-compose.nginx.yml ps
```

### 4. Bağlantıyı Test Edin

```bash
# Health check endpoint'ini test et
curl http://localhost/health

# HTTPS bağlantısını test et
curl -k https://localhost/health

# WebSocket bağlantısını test et (wscat gerekli)
wscat -c wss://localhost/app/app-key?protocol=7
```

## Yapılandırma

### Port Yapılandırması

Varsayılan portlar:
- **80**: HTTP (HTTPS'e yönlendirir)
- **443**: HTTPS (HTTP/2)
- **443/udp**: HTTP/3 (QUIC)
- **6001**: Soketi WebSocket (isteğe bağlı doğrudan erişim)
- **9601**: Soketi Metrics (isteğe bağlı)

### SSL/TLS Sertifikaları

Sertifika dosyaları şu konumlara mount edilir:
- `/etc/nginx/ssl/cert.pem`: SSL sertifikası
- `/etc/nginx/ssl/key.pem`: SSL private key

Environment değişkenleri ile özelleştirilebilir:
```bash
SSL_CERT_PATH=./path/to/cert.pem
SSL_KEY_PATH=./path/to/key.pem
```

### Nginx Yapılandırması

Nginx yapılandırma dosyaları:
- `nginx.conf`: Ana Nginx yapılandırması
- `default.conf`: Server block yapılandırması

Özel yapılandırma kullanmak için:
```bash
NGINX_CONF_PATH=./custom-nginx.conf
NGINX_DEFAULT_CONF_PATH=./custom-default.conf
```

### Soketi Yapılandırması

Soketi yapılandırması environment değişkenleri veya config dosyası ile yapılabilir:

```bash
# Config dosyası kullan
SOKETI_CONFIG_PATH=./custom-config.json
```

## Production Deployment

### 1. Güvenlik

- **Gerçek SSL Sertifikaları**: Let's Encrypt veya ticari CA kullanın
- **Güçlü Secrets**: Güvenli app-id, app-key ve app-secret oluşturun
- **Firewall**: Sadece gerekli portları açın
- **Rate Limiting**: Nginx'te rate limiting yapılandırın

### 2. Performans

- **Worker Processes**: `nginx.conf`'ta worker_processes sayısını CPU çekirdeği sayısına göre ayarlayın
- **Connection Limits**: Soketi max_connections değerini ayarlayın
- **Caching**: Static asset'ler için caching ekleyin

### 3. Monitoring

```bash
# Nginx loglarını izle
docker-compose -f docker-compose.nginx.yml logs -f nginx

# Soketi loglarını izle
docker-compose -f docker-compose.nginx.yml logs -f soketi

# Metrics endpoint'ini kontrol et
curl http://localhost:9601/metrics
```

### 4. Scaling

Birden fazla Soketi instance için:

```yaml
services:
  soketi:
    deploy:
      replicas: 3
```

Nginx otomatik olarak load balancing yapacaktır.

## Sorun Giderme

### SSL Sertifika Hataları

```bash
# Sertifika geçerliliğini kontrol et
openssl x509 -in certs/cert.pem -text -noout

# Sertifika ve key eşleşmesini kontrol et
openssl x509 -noout -modulus -in certs/cert.pem | openssl md5
openssl rsa -noout -modulus -in certs/key.pem | openssl md5
```

### Nginx Yapılandırma Hataları

```bash
# Nginx yapılandırmasını test et
docker-compose -f docker-compose.nginx.yml exec nginx nginx -t

# Nginx'i reload et
docker-compose -f docker-compose.nginx.yml exec nginx nginx -s reload
```

### WebSocket Bağlantı Sorunları

```bash
# Nginx access loglarını kontrol et
docker-compose -f docker-compose.nginx.yml exec nginx tail -f /var/log/nginx/access.log

# Soketi loglarını kontrol et
docker-compose -f docker-compose.nginx.yml logs soketi
```

### Health Check Başarısız

```bash
# Servislerin durumunu kontrol et
docker-compose -f docker-compose.nginx.yml ps

# Health check endpoint'ini manuel test et
docker-compose -f docker-compose.nginx.yml exec nginx curl http://localhost/health
docker-compose -f docker-compose.nginx.yml exec soketi curl http://localhost:6001/health
```

## Servisleri Durdurma

```bash
# Servisleri durdur
docker-compose -f docker-compose.nginx.yml down

# Servisleri durdur ve volume'leri sil
docker-compose -f docker-compose.nginx.yml down -v
```

## İleri Düzey Yapılandırma

### Custom Error Pages

```yaml
volumes:
  - ./error-pages:/usr/share/nginx/html/errors:ro
```

### CA Bundle Ekleme

```yaml
volumes:
  - ${SSL_CA_PATH:-./certs/ca.pem}:/etc/nginx/ssl/ca.pem:ro
```

`default.conf`'a ekleyin:
```nginx
ssl_client_certificate /etc/nginx/ssl/ca.pem;
ssl_verify_client optional;
```

### Rate Limiting

`nginx.conf`'a ekleyin:
```nginx
http {
    limit_req_zone $binary_remote_addr zone=websocket:10m rate=10r/s;
    
    # ... diğer yapılandırma
}
```

`default.conf`'ta kullanın:
```nginx
location /app/ {
    limit_req zone=websocket burst=20 nodelay;
    # ... diğer yapılandırma
}
```

## Referanslar

- [Nginx HTTP/3 Documentation](https://nginx.org/en/docs/http/ngx_http_v3_module.html)
- [Soketi Documentation](https://docs.soketi.app/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Let's Encrypt](https://letsencrypt.org/)

## Lisans

Bu yapılandırma dosyaları soketi.rs projesi ile aynı lisans altındadır.
