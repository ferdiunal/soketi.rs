# Veritabanı Kurulum Rehberi

Bu rehber, Soketi.rs'de farklı uygulama yöneticileri için veritabanı yapılandırmasını kapsar.

## Desteklenen Veritabanları

Soketi.rs, uygulama yapılandırması için birden fazla veritabanı backend'ini destekler:

- **PostgreSQL** - Production için önerilir
- **MySQL** - Alternatif SQL veritabanı
- **DynamoDB** - AWS-native NoSQL çözümü
- **Array** - Bellek içi yapılandırma (sadece geliştirme)

## PostgreSQL Kurulumu

### Kurulum

**macOS (Homebrew):**
```bash
brew install postgresql@16
brew services start postgresql@16
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Docker:**
```bash
docker run --name soketi-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=soketi \
  -p 5432:5432 \
  -d postgres:16
```

### Veritabanı ve Tablo Oluşturma

```sql
-- Veritabanı oluştur
CREATE DATABASE soketi;

-- Veritabanına bağlan
\c soketi

-- Apps tablosu oluştur
CREATE TABLE apps (
    id VARCHAR(255) PRIMARY KEY,
    key VARCHAR(255) NOT NULL UNIQUE,
    secret VARCHAR(255) NOT NULL,
    max_connections BIGINT,
    enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    max_backend_events_per_second BIGINT,
    max_client_events_per_second BIGINT,
    max_read_requests_per_second BIGINT,
    webhooks JSONB,
    max_presence_members_per_channel BIGINT,
    max_presence_member_size_in_kb DOUBLE PRECISION,
    max_channel_name_length BIGINT,
    max_event_channels_at_once BIGINT,
    max_event_name_length BIGINT,
    max_event_payload_in_kb DOUBLE PRECISION,
    max_event_batch_size BIGINT,
    enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE
);

-- Hızlı sorgular için index oluştur
CREATE INDEX idx_apps_key ON apps(key);
```

### Örnek Veri

```sql
INSERT INTO apps (id, key, secret, enabled, enable_client_messages)
VALUES ('app-1', 'app-key-1', 'app-secret-1', true, false);
```

### Bağlantı Dizesi

```
postgresql://kullanici_adi:sifre@host:port/veritabani
```

Örnek:
```
postgresql://postgres:password@localhost:5432/soketi
```

## MySQL Kurulumu

### Kurulum

**macOS (Homebrew):**
```bash
brew install mysql
brew services start mysql
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install mysql-server
sudo systemctl start mysql
```

**Docker:**
```bash
docker run --name soketi-mysql \
  -e MYSQL_ROOT_PASSWORD=password \
  -e MYSQL_DATABASE=soketi \
  -p 3306:3306 \
  -d mysql:8.0
```

### Veritabanı ve Tablo Oluşturma

```sql
-- Veritabanı oluştur
CREATE DATABASE soketi CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

USE soketi;

-- Apps tablosu oluştur
CREATE TABLE apps (
    id VARCHAR(255) PRIMARY KEY,
    `key` VARCHAR(255) NOT NULL UNIQUE,
    secret VARCHAR(255) NOT NULL,
    max_connections BIGINT UNSIGNED,
    enable_client_messages BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    max_backend_events_per_second BIGINT UNSIGNED,
    max_client_events_per_second BIGINT UNSIGNED,
    max_read_requests_per_second BIGINT UNSIGNED,
    webhooks JSON,
    max_presence_members_per_channel BIGINT UNSIGNED,
    max_presence_member_size_in_kb DOUBLE,
    max_channel_name_length BIGINT UNSIGNED,
    max_event_channels_at_once BIGINT UNSIGNED,
    max_event_name_length BIGINT UNSIGNED,
    max_event_payload_in_kb DOUBLE,
    max_event_batch_size BIGINT UNSIGNED,
    enable_user_authentication BOOLEAN NOT NULL DEFAULT FALSE,
    INDEX idx_key (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

### Bağlantı Dizesi

```
mysql://kullanici_adi:sifre@host:port/veritabani
```

Örnek:
```
mysql://root:password@localhost:3306/soketi
```

## DynamoDB Kurulumu (AWS)

### Tablo Şeması

**Primary Key:**
- Partition Key: `id` (String)

**Global Secondary Index:**
- Index Name: `key-index`
- Partition Key: `key` (String)

### Tablo Oluşturma

**AWS CLI:**
```bash
aws dynamodb create-table \
    --table-name apps \
    --attribute-definitions \
        AttributeName=id,AttributeType=S \
        AttributeName=key,AttributeType=S \
    --key-schema \
        AttributeName=id,KeyType=HASH \
    --global-secondary-indexes \
        "[
            {
                \"IndexName\": \"key-index\",
                \"KeySchema\": [{\"AttributeName\":\"key\",\"KeyType\":\"HASH\"}],
                \"Projection\":{\"ProjectionType\":\"ALL\"},
                \"ProvisionedThroughput\": {
                    \"ReadCapacityUnits\": 5,
                    \"WriteCapacityUnits\": 5
                }
            }
        ]" \
    --provisioned-throughput \
        ReadCapacityUnits=5,WriteCapacityUnits=5 \
    --region us-east-1
```

### Yerel Geliştirme için DynamoDB Local

```bash
# DynamoDB Local'i başlat
docker run -p 8000:8000 amazon/dynamodb-local

# Yerel endpoint ile tablo oluştur
aws dynamodb create-table \
    --endpoint-url http://localhost:8000 \
    --table-name apps \
    # ... (yukarıdaki parametrelerle aynı)
```

## Soketi.rs'de Yapılandırma

### PostgreSQL

```json
{
  "app_manager": {
    "driver": "postgres",
    "postgres": {
      "connection_string": "postgresql://postgres:password@localhost/soketi",
      "table_name": "apps"
    }
  }
}
```

### MySQL

```json
{
  "app_manager": {
    "driver": "mysql",
    "mysql": {
      "connection_string": "mysql://root:password@localhost/soketi",
      "table_name": "apps"
    }
  }
}
```

### DynamoDB

```json
{
  "app_manager": {
    "driver": "dynamodb",
    "dynamodb": {
      "table_name": "apps",
      "region": "us-east-1"
    }
  }
}
```

## Bağlantı Havuzu (Connection Pooling)

### PostgreSQL/MySQL

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .min_connections(2)
    .max_connections(10)
    .connect_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect("postgresql://...")
    .await?;
```

**Önerilen Ayarlar:**
- Geliştirme: 2-5 bağlantı
- Production (tek instance): 10-20 bağlantı
- Production (çoklu instance): Instance başına 5-10 bağlantı

## Önbellekleme

Veritabanı yükünü azaltmak için önbelleklemeyi etkinleştirin:

```json
{
  "cache": {
    "driver": "redis",
    "redis": {
      "host": "localhost",
      "port": 6379
    }
  }
}
```

Önbellek TTL: Varsayılan olarak 3600 saniye (1 saat).

## Güvenlik En İyi Uygulamaları

1. **Güçlü şifreler kullanın** - Production'da asla varsayılan şifreleri kullanmayın
2. **SSL/TLS etkinleştirin** - Şifreli bağlantılar kullanın
3. **İzinleri sınırlayın** - Minimal izinlerle özel kullanıcılar oluşturun
4. **Ağ güvenliği** - Veritabanı erişimini güvenilir IP'lerle sınırlayın
5. **Düzenli yedeklemeler** - Otomatik yedekleme ayarlayın

## İzleme

### PostgreSQL

```sql
-- Aktif bağlantıları görüntüle
SELECT * FROM pg_stat_activity WHERE datname = 'soketi';

-- Index kullanımını kontrol et
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
WHERE tablename = 'apps';
```

### MySQL

```sql
-- Aktif bağlantıları görüntüle
SHOW PROCESSLIST;

-- Index kullanımını kontrol et
SHOW INDEX FROM apps;
```

### DynamoDB

CloudWatch metriklerini izleyin:
- `ConsumedReadCapacityUnits`
- `ConsumedWriteCapacityUnits`
- `UserErrors` (throttling)
- `SystemErrors`

---

Daha fazla bilgi için [Yapılandırma](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/yapilandirma.md) ve [Deployment](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/docker-deployment.md) sayfalarına bakın.
