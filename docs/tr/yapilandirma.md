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

Ortam değişkenlerini kullanarak yapılandırmayı geçersiz kılabilirsiniz:

| Değişken | Açıklama | Varsayılan |
|----------|----------|------------|
| `SOKETI_HOST` | Sunucu host | `0.0.0.0` |
| `SOKETI_PORT` | Sunucu portu | `6001` |
| `SOKETI_DEFAULT_APP_ID` | Varsayılan app ID | - |
| `SOKETI_DEFAULT_APP_KEY` | Varsayılan app key | - |
| `SOKETI_DEFAULT_APP_SECRET` | Varsayılan app secret | - |
| `SOKETI_METRICS_ENABLED` | Metrikleri etkinleştir | `true` |
| `SOKETI_METRICS_PORT` | Metrik portu | `9601` |

Örnek:

```bash
SOKETI_PORT=8080 soketi --config config.json
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

- **[Başlangıç](baslangic.md)** - Hızlı başlangıç kılavuzu
- **[API Referansı](api-referans.md)** - API dokümantasyonu
- **[Deployment Kılavuzu](deployment/reverse-proxy.md)** - Production deployment

## İlgili Kaynaklar

- [MySQL Kurulum Kılavuzu](../MYSQL_SETUP.md)
- [PostgreSQL Kurulum Kılavuzu](../POSTGRES_SETUP.md)
- [DynamoDB Kurulum Kılavuzu](../DYNAMODB_SETUP.md)
- [Redis Adapter Implementasyonu](../REDIS_ADAPTER_IMPLEMENTATION.md)
- [NATS Adapter Implementasyonu](../NATS_ADAPTER_IMPLEMENTATION.md)
