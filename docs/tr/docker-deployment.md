# Docker Deployment Rehberi

## Genel Bakış

Soketi.rs, Docker Hub üzerinde Docker imajı olarak sunulmaktadır ve bu sayede kurulum oldukça basit ve hızlıdır.

**Docker Hub Repository**: [ferdiunal/soketi-rs](https://hub.docker.com/r/funal/soketi-rs)

## Hızlı Başlangıç

### İndirme ve Çalıştırma

```bash
# En son imajı indir
docker pull funal/soketi-rs:latest

# Sunucuyu çalıştır
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest
```

### Docker Compose Kullanımı

`docker-compose.yml` dosyası oluşturun:

```yaml
version: '3.8'

services:
  soketi:
    image: funal/soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    volumes:
      - ./config.json:/app/config.json
    restart: unless-stopped
    
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    restart: unless-stopped
```

Servisleri başlatın:

```bash
docker-compose up -d
```

## Yapılandırma

### config.json Kullanımı

Proje dizininizde `config.json` dosyası oluşturun:

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "metrics_port": 9601,
  "apps": [
    {
      "id": "app-id",
      "key": "app-key",
      "secret": "app-secret",
      "max_connections": 100,
      "enable_client_messages": true,
      "enabled": true
    }
  ]
}
```

Container'a bağlayın:

```bash
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  funal/soketi-rs:latest
```

### Ortam Değişkenleri Kullanımı

```bash
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -e SOKETI_HOST=0.0.0.0 \
  -e SOKETI_PORT=6001 \
  -e SOKETI_METRICS_PORT=9601 \
  funal/soketi-rs:latest
```

## Mevcut Etiketler

- `latest` - En son kararlı sürüm
- `v1.x.x` - Belirli bir semantik sürüm
- `main` - En son geliştirme sürümü

## Desteklenen Platformlar

- `linux/amd64` - x86_64 mimarisi
- `linux/arm64` - ARM64 mimarisi (Apple Silicon, AWS Graviton, vb.)

## Portlar

- **6001** - WebSocket sunucusu (varsayılan)
- **9601** - Metrik uç noktası (Prometheus uyumlu)

## Özellikler

- ✅ Pusher protokolü uyumlu
- ✅ WebSocket desteği
- ✅ Public, private ve presence kanalları
- ✅ İstemci olayları
- ✅ Webhook'lar
- ✅ Prometheus metrikleri
- ✅ Çoklu adaptörler (Local, Redis, NATS, Cluster)
- ✅ Yüksek performans
- ✅ Düşük bellek kullanımı

## Production Deployment

### Redis Adaptörü ile

```yaml
version: '3.8'

services:
  soketi:
    image: funal/soketi-rs:latest
    ports:
      - "6001:6001"
      - "9601:9601"
    volumes:
      - ./config.json:/app/config.json
    environment:
      - REDIS_HOST=redis
      - REDIS_PORT=6379
    depends_on:
      - redis
    restart: unless-stopped
    
  redis:
    image: redis:alpine
    restart: unless-stopped
```

### Ölçeklendirme

Birden fazla instance ile yatay ölçeklendirme:

```bash
docker-compose up -d --scale soketi=3
```

### Sağlık Kontrolleri

```bash
# Sunucu durumunu kontrol et
curl http://localhost:6001/health

# Metrikleri kontrol et
curl http://localhost:9601/metrics
```

## İzleme

### Prometheus Entegrasyonu

`docker-compose.yml` dosyanıza Prometheus ekleyin:

```yaml
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    restart: unless-stopped
```

`prometheus.yml` oluşturun:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['soketi:9601']
```

## Sorun Giderme

### Logları Görüntüleme

```bash
# Docker
docker logs soketi

# Docker Compose
docker-compose logs -f soketi
```

### Container'ı Yeniden Başlatma

```bash
# Docker
docker restart soketi

# Docker Compose
docker-compose restart soketi
```

### Her Şeyi Sıfırlama

```bash
docker-compose down -v
docker-compose up -d
```

## CI/CD ve Otomatik Build'ler

### GitHub Actions Workflow

Proje, otomatik Docker imaj build'leri ve deployment'ları için GitHub Actions kullanır. İmajlar şu durumlarda otomatik olarak build edilir ve Docker Hub'a push edilir:

- **Main branch'e push** - `main` etiketi oluşturur
- **Versiyon etiketleri** - Versiyonlu etiketler oluşturur (örn: `v1.0.0`, `1.0`, `1`)
- **Pull request'ler** - Sadece build yapar (push yapmaz)

### Çoklu Platform Build'leri

İmajlar birden fazla mimari için build edilir:
- `linux/amd64` - Standart x86_64 sunucular
- `linux/arm64` - ARM tabanlı sunucular (AWS Graviton, Apple Silicon, vb.)

### Build Optimizasyonu

- **Multi-stage build'ler** - Daha küçük final imajlar (~100-200MB)
- **Build cache** - Sonraki build'ler daha hızlı
- **Katman optimizasyonu** - Verimli Docker katman önbellekleme

### Otomatik README Senkronizasyonu

Docker Hub README'si her release'de repository'den otomatik olarak güncellenir.

## Katkıda Bulunanlar İçin

### Yerel Build

```bash
# Platformunuz için build edin
docker build -t soketi-rs .

# Birden fazla platform için build edin
docker buildx build --platform linux/amd64,linux/arm64 -t soketi-rs .
```

### Docker Hub'a Yayınlama

CI/CD pipeline bunu otomatik olarak halleder, ancak manuel yayınlama için:

```bash
# Docker Hub'a giriş yapın
docker login

# Build edin ve push edin
docker buildx build --platform linux/amd64,linux/arm64 \
  -t funal/soketi-rs:latest \
  --push .
```

### Release Oluşturma

```bash
# Yeni bir versiyon etiketle
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions otomatik olarak:
# 1. Docker imajını build eder
# 2. Versiyon etiketleriyle Docker Hub'a push eder
# 3. Docker Hub README'sini günceller
```

## Destek

- **GitHub Issues**: [Hata bildir](https://github.com/ferdiunal/soketi.rs/issues)
- **GitHub Discussions**: [Soru sor](https://github.com/ferdiunal/soketi.rs/discussions)
- **Dokümantasyon**: [Tam dokümantasyon](https://github.com/ferdiunal/soketi.rs/tree/main/docs)

## Lisans

MIT Lisansı - detaylar için [LICENSE](https://github.com/ferdiunal/soketi.rs/blob/main/LICENSE) dosyasına bakın.
