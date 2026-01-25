# Yapılandırma Referansı

> Tüm Soketi yapılandırma seçenekleri için eksiksiz referans.

## İçindekiler

- [Yapılandırma Dosyası](#yapılandırma-dosyası)
- [Ortam Değişkenleri](#ortam-değişkenleri)
- [App Manager Yapılandırması](#app-manager-yapılandırması)
- [Sunucu Yapılandırması](#sunucu-yapılandırması)
- [Metrik Yapılandırması](#metrik-yapılandırması)
- [Adapter Yapılandırması](#adapter-yapılandırması)
- [Kuyruk Yapılandırması](#kuyruk-yapılandırması)
- [Hız Sınırlama](#hız-sınırlama)
- [SSL/TLS Yapılandırması](#ssltls-yapılandırması)

## Yapılandırma Dosyası

Soketi, JSON yapılandırma dosyası kullanılarak yapılandırılabilir. Varsayılan olarak, mevcut dizinde `config.json` dosyasını arar.

Özel bir yapılandırma dosyası belirtin:

```bash
soketi --config /path/to/config.json
```

### Temel Yapılandırma Örneği

```json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "max_connections": 100,
          "enable_client_messages": true,
          "enabled": true,
          "max_backend_events_per_second": 100,
          "max_client_events_per_second": 100,
          "max_read_requests_per_second": 100
        }
      ]
    }
  },
  "host": "0.0.0.0",
  "port": 6001,
  "metrics": {
    "enabled": true,
    "port": 9601,
    "host": "0.0.0.0"
  }
}
```

## Ortam Değişkenleri

Tüm yapılandırma seçenekleri `PUSHER_` öneki ile ortam değişkenleri kullanılarak geçersiz kılınabilir.

### Sunucu Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_HOST` | Sunucu host | `0.0.0.0` |
| `PUSHER_PORT` | Sunucu portu | `6001` |
| `PUSHER_DEBUG` | Debug modunu etkinleştir | `false` |
| `PUSHER_MODE` | Sunucu modu (full, server, worker) | `full` |

### App Manager Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_APP_MANAGER_DRIVER` | App manager driver (array, dynamodb, mysql, postgres) | `array` |
| `PUSHER_APP_MANAGER_ARRAY_APPS` | JSON array formatında app listesi (array driver için) | `[]` |
| `PUSHER_DEFAULT_APP_ID` | Varsayılan app ID (tek app oluşturur) | - |
| `PUSHER_DEFAULT_APP_KEY` | Varsayılan app key (tek app oluşturur) | - |
| `PUSHER_DEFAULT_APP_SECRET` | Varsayılan app secret (tek app oluşturur) | - |
| `PUSHER_APP_MANAGER_CACHE_ENABLED` | App manager cache'i etkinleştir | `false` |
| `PUSHER_APP_MANAGER_CACHE_TTL` | Cache TTL (saniye) | `3600` |

### Adapter Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_ADAPTER_DRIVER` | Adapter driver (local, cluster, redis, nats) | `local` |
| `PUSHER_ADAPTER_REDIS_HOST` | Redis adapter host | `127.0.0.1` |
| `PUSHER_ADAPTER_REDIS_PORT` | Redis adapter port | `6379` |
| `PUSHER_ADAPTER_REDIS_DB` | Redis adapter veritabanı | `0` |
| `PUSHER_ADAPTER_REDIS_PASSWORD` | Redis adapter şifresi | - |
| `PUSHER_ADAPTER_CLUSTER_PORT` | Cluster adapter portu | `11002` |
| `PUSHER_ADAPTER_NATS_SERVERS` | NATS sunucuları (virgülle ayrılmış) | `127.0.0.1:4222` |

### Cache Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_CACHE_DRIVER` | Cache driver (memory, redis) | `memory` |
| `PUSHER_CACHE_REDIS_HOST` | Redis cache host | `127.0.0.1` |
| `PUSHER_CACHE_REDIS_PORT` | Redis cache port | `6379` |
| `PUSHER_CACHE_REDIS_PASSWORD` | Redis cache şifresi | - |

### Metrik Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_METRICS_ENABLED` | Metrikleri etkinleştir | `false` |
| `PUSHER_METRICS_PORT` | Metrik portu | `9601` |
| `PUSHER_METRICS_PREFIX` | Metrik öneki | `pusher` |

### Veritabanı Yapılandırması (MySQL)

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_MYSQL_HOST` | MySQL host | `127.0.0.1` |
| `PUSHER_MYSQL_PORT` | MySQL port | `3306` |
| `PUSHER_MYSQL_USER` | MySQL kullanıcı | `root` |
| `PUSHER_MYSQL_PASSWORD` | MySQL şifre | - |
| `PUSHER_MYSQL_DATABASE` | MySQL veritabanı | `main` |

### Veritabanı Yapılandırması (PostgreSQL)

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_POSTGRES_HOST` | PostgreSQL host | `127.0.0.1` |
| `PUSHER_POSTGRES_PORT` | PostgreSQL port | `5432` |
| `PUSHER_POSTGRES_USER` | PostgreSQL kullanıcı | `postgres` |
| `PUSHER_POSTGRES_PASSWORD` | PostgreSQL şifre | - |
| `PUSHER_POSTGRES_DATABASE` | PostgreSQL veritabanı | `main` |

### Veritabanı Yapılandırması (DynamoDB)

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_DYNAMODB_TABLE` | DynamoDB tablo adı | `apps` |
| `PUSHER_DYNAMODB_REGION` | AWS bölgesi | `us-east-1` |

### Kuyruk Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_QUEUE_DRIVER` | Kuyruk driver (sync, redis, sqs) | `sync` |
| `PUSHER_QUEUE_REDIS_HOST` | Redis kuyruk host | `127.0.0.1` |
| `PUSHER_QUEUE_REDIS_PORT` | Redis kuyruk port | `6379` |
| `PUSHER_QUEUE_SQS_URL` | SQS kuyruk URL | - |
| `PUSHER_QUEUE_SQS_REGION` | SQS bölgesi | `us-east-1` |

### SSL Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_SSL_ENABLED` | SSL'i etkinleştir | `false` |
| `PUSHER_SSL_CERT_PATH` | SSL sertifika yolu | - |
| `PUSHER_SSL_KEY_PATH` | SSL anahtar yolu | - |

### CORS Yapılandırması

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `PUSHER_CORS_ENABLED` | CORS'u etkinleştir | `true` |
| `PUSHER_CORS_ORIGINS` | İzin verilen origin'ler (virgülle ayrılmış) | `*` |

### Örnekler

#### Varsayılan App Kullanımı (Basit Kurulum)

```bash
PUSHER_DEFAULT_APP_ID=my-app \
PUSHER_DEFAULT_APP_KEY=my-key \
PUSHER_DEFAULT_APP_SECRET=my-secret \
soketi-rs
```

#### JSON Array ile Birden Fazla App

```bash
PUSHER_APP_MANAGER_ARRAY_APPS='[{"id":"app1","key":"key1","secret":"secret1","enabled":true}]' \
soketi-rs
```

#### Redis Adapter ile

```bash
PUSHER_ADAPTER_DRIVER=redis \
PUSHER_ADAPTER_REDIS_HOST=localhost \
PUSHER_ADAPTER_REDIS_PORT=6379 \
PUSHER_ADAPTER_REDIS_PASSWORD=mypassword \
soketi-rs
```

#### Docker Compose Örneği

```yaml
environment:
  PUSHER_DEFAULT_APP_ID: "shopilens"
  PUSHER_DEFAULT_APP_KEY: "shopilens-key"
  PUSHER_DEFAULT_APP_SECRET: "shopilens-secret"
  PUSHER_ADAPTER_DRIVER: "redis"
  PUSHER_ADAPTER_REDIS_HOST: "redis"
  PUSHER_ADAPTER_REDIS_PORT: "6379"
  PUSHER_ADAPTER_REDIS_PASSWORD: "mypassword"
  PUSHER_CACHE_DRIVER: "redis"
  PUSHER_CACHE_REDIS_HOST: "redis"
  PUSHER_METRICS_ENABLED: "true"
  PUSHER_METRICS_PORT: "9601"
```

## App Manager Yapılandırması

App manager, uygulamaların nasıl saklandığını ve yönetildiğini kontrol eder.

### Array Driver (Bellekte)

Geliştirme ve tek instance deployment'lar için en iyisi:

```json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "max_connections": 100,
          "enable_client_messages": true,
          "enabled": true,
          "max_backend_events_per_second": 100,
          "max_client_events_per_second": 100,
          "max_read_requests_per_second": 100,
          "webhooks": []
        }
      ]
    }
  }
}
```

### MySQL Driver

MySQL ile production deployment'lar için:

```json
{
  "app_manager": {
    "driver": "mysql",
    "mysql": {
      "host": "localhost",
      "port": 3306,
      "database": "soketi",
      "username": "soketi",
      "password": "password"
    }
  }
}
```

### PostgreSQL Driver

PostgreSQL ile production deployment'lar için:

```json
{
  "app_manager": {
    "driver": "postgres",
    "postgres": {
      "host": "localhost",
      "port": 5432,
      "database": "soketi",
      "username": "soketi",
      "password": "password"
    }
  }
}
```

### DynamoDB Driver

AWS deployment'ları için:

```json
{
  "app_manager": {
    "driver": "dynamodb",
    "dynamodb": {
      "table": "soketi_apps",
      "region": "us-east-1"
    }
  }
}
```

## Sunucu Yapılandırması

### Temel Sunucu Seçenekleri

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "path": "/",
  "max_payload_size": 100000
}
```

| Seçenek | Açıklama | Varsayılan |
|---------|----------|------------|
| `host` | Sunucu bind adresi | `0.0.0.0` |
| `port` | Sunucu portu | `6001` |
| `path` | WebSocket yolu | `/` |
| `max_payload_size` | Maksimum mesaj boyutu (byte) | `100000` |

## Metrik Yapılandırması

Prometheus uyumlu metrikleri etkinleştirin:

```json
{
  "metrics": {
    "enabled": true,
    "port": 9601,
    "host": "0.0.0.0"
  }
}
```

Metriklere `http://localhost:9601/metrics` adresinden erişin.

## Adapter Yapılandırması

Adapter'lar, birden fazla Soketi instance'ı arasında event'leri senkronize ederek yatay ölçeklendirmeyi sağlar.

### Local Adapter (Varsayılan)

Yapılandırma gerekmez. Sadece tek instance ile çalışır:

```json
{
  "adapter": {
    "driver": "local"
  }
}
```

### Redis Adapter

Çoklu instance deployment'lar için:

```json
{
  "adapter": {
    "driver": "redis",
    "redis": {
      "host": "localhost",
      "port": 6379,
      "password": "password",
      "db": 0,
      "key_prefix": "soketi"
    }
  }
}
```

### NATS Adapter

NATS tabanlı clustering için:

```json
{
  "adapter": {
    "driver": "nats",
    "nats": {
      "servers": ["nats://localhost:4222"],
      "prefix": "soketi"
    }
  }
}
```

## Kuyruk Yapılandırması

Webhook teslimi için kuyruk yapılandırması:

### Local Queue

```json
{
  "queue": {
    "driver": "local"
  }
}
```

### SQS Queue

AWS SQS için:

```json
{
  "queue": {
    "driver": "sqs",
    "sqs": {
      "queue_url": "https://sqs.us-east-1.amazonaws.com/123456789/soketi-webhooks",
      "region": "us-east-1"
    }
  }
}
```

## Hız Sınırlama

Uygulama başına hız limitleri yapılandırın:

```json
{
  "id": "app-id",
  "max_connections": 100,
  "max_backend_events_per_second": 100,
  "max_client_events_per_second": 100,
  "max_read_requests_per_second": 100
}
```

| Seçenek | Açıklama | Varsayılan |
|---------|----------|------------|
| `max_connections` | Maksimum eşzamanlı bağlantı | `100` |
| `max_backend_events_per_second` | Backend event hız limiti | `100` |
| `max_client_events_per_second` | Client event hız limiti | `100` |
| `max_read_requests_per_second` | Okuma isteği hız limiti | `100` |

## SSL/TLS Yapılandırması

Güvenli bağlantılar için SSL/TLS'yi etkinleştirin:

```json
{
  "ssl": {
    "enabled": true,
    "cert_file": "/path/to/cert.pem",
    "key_file": "/path/to/key.pem"
  }
}
```

**Not**: Production'da, SSL/TLS sonlandırması için reverse proxy (Caddy veya Nginx) kullanılması önerilir.

## Eksiksiz Yapılandırma Örneği

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "path": "/",
  "max_payload_size": 100000,
  
  "app_manager": {
    "driver": "postgres",
    "postgres": {
      "host": "localhost",
      "port": 5432,
      "database": "soketi",
      "username": "soketi",
      "password": "password"
    }
  },
  
  "adapter": {
    "driver": "redis",
    "redis": {
      "host": "localhost",
      "port": 6379,
      "password": "password",
      "db": 0,
      "key_prefix": "soketi"
    }
  },
  
  "queue": {
    "driver": "sqs",
    "sqs": {
      "queue_url": "https://sqs.us-east-1.amazonaws.com/123456789/soketi-webhooks",
      "region": "us-east-1"
    }
  },
  
  "metrics": {
    "enabled": true,
    "port": 9601,
    "host": "0.0.0.0"
  },
  
  "ssl": {
    "enabled": false
  }
}
```

## Sonraki Adımlar

- **[Başlangıç](baslangic)** - Hızlı başlangıç kılavuzu
- **[API Referansı](api-referans)** - API dokümantasyonu
- **[Deployment Kılavuzu](deployment/reverse-proxy)** - Production deployment

## İlgili Kaynaklar

- [MySQL Kurulum Kılavuzu](../MYSQL_SETUP)
- [PostgreSQL Kurulum Kılavuzu](../POSTGRES_SETUP)
- [DynamoDB Kurulum Kılavuzu](../DYNAMODB_SETUP)
- [Redis Adapter Implementasyonu](../REDIS_ADAPTER_IMPLEMENTATION)
- [NATS Adapter Implementasyonu](../NATS_ADAPTER_IMPLEMENTATION)
