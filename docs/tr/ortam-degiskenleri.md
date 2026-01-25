# Ortam Değişkenleri Rehberi

Bu rehber, Soketi'yi ortam değişkenleri kullanarak nasıl yapılandıracağınızı açıklar.

## Genel Bakış

Soketi şu yöntemlerle yapılandırılabilir:
1. **Yapılandırma dosyaları** (JSON, YAML, TOML) - `--config` flag'i ile yüklenir
2. **Ortam değişkenleri** - Önek: `PUSHER_`
3. **Komut satırı argümanları** - En yüksek öncelik

Öncelik sırası (en yüksekten en düşüğe):
1. Komut satırı argümanları
2. Ortam değişkenleri
3. Yapılandırma dosyası
4. Varsayılan değerler

## Hızlı Başlangıç

### Varsayılan App ile Basit Kurulum

Başlamanın en kolay yolu varsayılan app ortam değişkenlerini kullanmaktır:

```bash
PUSHER_DEFAULT_APP_ID=my-app \
PUSHER_DEFAULT_APP_KEY=my-key \
PUSHER_DEFAULT_APP_SECRET=my-secret \
./soketi-rs
```

### Docker Compose Kurulumu

```yaml
services:
  soketi:
    image: funal/soketi-rs:latest
    environment:
      PUSHER_DEFAULT_APP_ID: "my-app"
      PUSHER_DEFAULT_APP_KEY: "my-key"
      PUSHER_DEFAULT_APP_SECRET: "my-secret"
      PUSHER_ADAPTER_DRIVER: "redis"
      PUSHER_ADAPTER_REDIS_HOST: "redis"
      PUSHER_METRICS_ENABLED: "true"
```

## App Yapılandırması

### Seçenek 1: Varsayılan App (Tek App için Önerilen)

Üç ortam değişkeni kullanarak tek bir app oluşturun:

```bash
PUSHER_DEFAULT_APP_ID=my-app
PUSHER_DEFAULT_APP_KEY=my-key
PUSHER_DEFAULT_APP_SECRET=my-secret
```

Bu, tek bir uygulama yapılandırmanın en basit yoludur.

### Seçenek 2: JSON Array (Birden Fazla App)

Birden fazla app için JSON array kullanın:

```bash
PUSHER_APP_MANAGER_ARRAY_APPS='[
  {
    "id": "app1",
    "key": "key1",
    "secret": "secret1",
    "enabled": true,
    "enable_client_messages": true,
    "max_connections": 10000
  },
  {
    "id": "app2",
    "key": "key2",
    "secret": "secret2",
    "enabled": true
  }
]'
```

### Seçenek 3: Yapılandırma Dosyası + Ortam Değişkenleri

Config dosyasını mount edin ve belirli değerleri override edin:

```yaml
services:
  soketi:
    image: funal/soketi-rs:latest
    volumes:
      - ./config.json:/app/config.json:ro
    environment:
      # Adapter'ı Redis kullanacak şekilde override et
      PUSHER_ADAPTER_DRIVER: "redis"
      PUSHER_ADAPTER_REDIS_HOST: "redis"
      PUSHER_ADAPTER_REDIS_PASSWORD: "mypassword"
```

## Tüm Ortam Değişkenleri Referansı

### Sunucu Yapılandırması

```bash
PUSHER_HOST=0.0.0.0                    # Sunucu bind adresi
PUSHER_PORT=6001                       # Sunucu portu
PUSHER_DEBUG=true                      # Debug loglamayı etkinleştir
PUSHER_MODE=full                       # Sunucu modu: full, server, worker
PUSHER_SHUTDOWN_GRACE_PERIOD_MS=3000   # Graceful shutdown timeout
```

### App Manager

```bash
PUSHER_APP_MANAGER_DRIVER=array        # Driver: array, dynamodb, mysql, postgres
PUSHER_APP_MANAGER_CACHE_ENABLED=true  # App cache'i etkinleştir
PUSHER_APP_MANAGER_CACHE_TTL=3600      # Cache TTL (saniye)

# Varsayılan app (tek app oluşturur)
PUSHER_DEFAULT_APP_ID=my-app
PUSHER_DEFAULT_APP_KEY=my-key
PUSHER_DEFAULT_APP_SECRET=my-secret

# Veya JSON array (birden fazla app)
PUSHER_APP_MANAGER_ARRAY_APPS='[...]'
```

### Adapter Yapılandırması

```bash
PUSHER_ADAPTER_DRIVER=redis            # Driver: local, cluster, redis, nats

# Redis Adapter
PUSHER_ADAPTER_REDIS_HOST=127.0.0.1
PUSHER_ADAPTER_REDIS_PORT=6379
PUSHER_ADAPTER_REDIS_DB=0
PUSHER_ADAPTER_REDIS_PASSWORD=secret

# Cluster Adapter
PUSHER_ADAPTER_CLUSTER_PORT=11002

# NATS Adapter
PUSHER_ADAPTER_NATS_SERVERS=nats://localhost:4222,nats://localhost:4223
```

### Cache Yapılandırması

```bash
PUSHER_CACHE_DRIVER=redis              # Driver: memory, redis

# Redis Cache
PUSHER_CACHE_REDIS_HOST=127.0.0.1
PUSHER_CACHE_REDIS_PORT=6379
PUSHER_CACHE_REDIS_PASSWORD=secret
```

### Rate Limiter

```bash
PUSHER_RATE_LIMITER_DRIVER=local       # Driver: local, cluster, redis

# Redis Rate Limiter
PUSHER_RATE_LIMITER_REDIS_HOST=127.0.0.1
PUSHER_RATE_LIMITER_REDIS_PORT=6379
```

### Kuyruk Yapılandırması

```bash
PUSHER_QUEUE_DRIVER=sync               # Driver: sync, redis, sqs

# Redis Queue
PUSHER_QUEUE_REDIS_HOST=127.0.0.1
PUSHER_QUEUE_REDIS_PORT=6379

# SQS Queue
PUSHER_QUEUE_SQS_URL=https://sqs.us-east-1.amazonaws.com/123/queue
PUSHER_QUEUE_SQS_REGION=us-east-1
```

### Metrikler

```bash
PUSHER_METRICS_ENABLED=true
PUSHER_METRICS_PORT=9601
PUSHER_METRICS_PREFIX=soketi
```

### Veritabanı Yapılandırması

#### MySQL

```bash
PUSHER_MYSQL_HOST=127.0.0.1
PUSHER_MYSQL_PORT=3306
PUSHER_MYSQL_USER=root
PUSHER_MYSQL_PASSWORD=secret
PUSHER_MYSQL_DATABASE=soketi
```

#### PostgreSQL

```bash
PUSHER_POSTGRES_HOST=127.0.0.1
PUSHER_POSTGRES_PORT=5432
PUSHER_POSTGRES_USER=postgres
PUSHER_POSTGRES_PASSWORD=secret
PUSHER_POSTGRES_DATABASE=soketi
```

#### DynamoDB

```bash
PUSHER_DYNAMODB_TABLE=soketi_apps
PUSHER_DYNAMODB_REGION=us-east-1
```

### SSL/TLS

```bash
PUSHER_SSL_ENABLED=true
PUSHER_SSL_CERT_PATH=/path/to/cert.pem
PUSHER_SSL_KEY_PATH=/path/to/key.pem
```

### CORS

```bash
PUSHER_CORS_ENABLED=true
PUSHER_CORS_ORIGINS=https://example.com,https://app.example.com
```

### Limitler

```bash
PUSHER_CHANNEL_MAX_NAME_LENGTH=200
PUSHER_EVENT_MAX_NAME_LENGTH=200
PUSHER_EVENT_MAX_PAYLOAD_KB=100
PUSHER_EVENT_MAX_BATCH_SIZE=10
PUSHER_PRESENCE_MAX_MEMBERS=100
PUSHER_PRESENCE_MAX_MEMBER_SIZE_KB=2
PUSHER_HTTP_MAX_REQUEST_SIZE_KB=100
PUSHER_HTTP_MEMORY_THRESHOLD_MB=512
PUSHER_USER_AUTH_TIMEOUT_MS=30000
```

## Yaygın Senaryolar

### Geliştirme (Tek Instance)

```bash
PUSHER_DEFAULT_APP_ID=dev-app
PUSHER_DEFAULT_APP_KEY=dev-key
PUSHER_DEFAULT_APP_SECRET=dev-secret
PUSHER_DEBUG=true
```

### Production (Redis ile Yatay Ölçeklendirme)

```bash
PUSHER_DEFAULT_APP_ID=prod-app
PUSHER_DEFAULT_APP_KEY=prod-key
PUSHER_DEFAULT_APP_SECRET=prod-secret
PUSHER_ADAPTER_DRIVER=redis
PUSHER_ADAPTER_REDIS_HOST=redis.example.com
PUSHER_ADAPTER_REDIS_PASSWORD=secret
PUSHER_CACHE_DRIVER=redis
PUSHER_CACHE_REDIS_HOST=redis.example.com
PUSHER_METRICS_ENABLED=true
```

### Production (Veritabanı Destekli App'ler)

```bash
PUSHER_APP_MANAGER_DRIVER=postgres
PUSHER_POSTGRES_HOST=db.example.com
PUSHER_POSTGRES_USER=soketi
PUSHER_POSTGRES_PASSWORD=secret
PUSHER_POSTGRES_DATABASE=soketi
PUSHER_ADAPTER_DRIVER=redis
PUSHER_ADAPTER_REDIS_HOST=redis.example.com
```

## İpuçları

1. **.env dosyaları kullanın**: Yerel geliştirme için `.env` dosyası oluşturun
2. **Secret yönetimi**: Production için secret yönetim araçları kullanın (AWS Secrets Manager, HashiCorp Vault)
3. **Karmaşık kurulumlar için config dosyası**: Webhook'lu karmaşık app yapılandırmaları için config dosyaları kullanın
4. **Deployment için ortam değişkenleri**: Deployment'a özgü ayarlar için ortam değişkenleri kullanın (host'lar, şifreler)
5. **Boolean değerler**: `true` veya `false` kullanın (`1` veya `0` değil)

## Sorun Giderme

### App Bulunamadı Hatası

"App not found" hatası görüyorsanız:

1. Şunlardan birini ayarladığınızdan emin olun:
   - `PUSHER_DEFAULT_APP_*` değişkenleri, VEYA
   - `PUSHER_APP_MANAGER_ARRAY_APPS`, VEYA
   - App'lerin tanımlandığı bir config dosyası mount edilmiş

2. App key'in client'ınızın kullandığı ile eşleştiğini doğrulayın

3. Yapılandırma yükleme hatalarını kontrol etmek için logları inceleyin:
   ```bash
   PUSHER_DEBUG=true ./soketi-rs
   ```

### Boolean Değer Hataları

"invalid value '1' for '--debug'" hatası görüyorsanız:

`1`/`0` yerine `true`/`false` kullanın:
```bash
# ❌ Yanlış
PUSHER_DEBUG=1

# ✅ Doğru
PUSHER_DEBUG=true
```

## Ayrıca Bakınız

- [Yapılandırma Referansı](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/yapilandirma.md)
- [Docker Deployment Rehberi](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/docker-deployment.md)
- [Başlangıç](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/baslangic.md)
