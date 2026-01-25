# Docker Hızlı Başlangıç Rehberi

## Hızlı Komutlar

### Sunucuyu Başlat
```bash
docker-compose up -d
```

### Logları Görüntüle
```bash
docker-compose logs -f soketi
```

### Sunucuyu Durdur
```bash
docker-compose down
```

### Yeniden Oluştur
```bash
docker-compose build --no-cache
docker-compose up -d
```

## Mevcut Servisler

| Servis | Port | Açıklama |
|--------|------|----------|
| Soketi Sunucusu | 6001 | WebSocket sunucusu |
| Metrikler | 9601 | Prometheus metrikleri |
| Redis | 6379 | Önbellek & kümeleme |

## Docker Hub İmajını Kullanma

### İmajı İndirme ve Çalıştırma
```bash
# En son imajı indir
docker pull funal/soketi-rs:latest

# Varsayılan yapılandırma ile çalıştır
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest

# Özel yapılandırma ile çalıştır
docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  funal/soketi-rs:latest
```

### Docker Compose Kullanımı

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
```

## Yapılandırma

`config.json` dosyası oluşturun:

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

## Ortam Değişkenleri

`.env` dosyası oluşturun:

```env
# Soketi
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_METRICS_PORT=9601

# Uygulama Yapılandırması
APP_ID=uygulama-id
APP_KEY=uygulama-anahtari
APP_SECRET=uygulama-sifresi

# Redis (opsiyonel)
REDIS_HOST=redis
REDIS_PORT=6379
```

## Sağlık Kontrolleri

```bash
# Sunucu durumunu kontrol et
curl http://localhost:6001/health

# Metrikleri kontrol et
curl http://localhost:9601/metrics
```

## Sorun Giderme

### Port Zaten Kullanımda
```bash
# Portu kullanan işlemi bul
lsof -i :6001

# İşlemi durdur
kill -9 <PID>
```

### Container Başlamıyor
```bash
# Logları görüntüle
docker-compose logs soketi

# Container'ı yeniden başlat
docker-compose restart soketi
```

### Her Şeyi Sıfırla
```bash
docker-compose down -v
docker-compose up -d
```

## Desteklenen Mimariler

- `linux/amd64`
- `linux/arm64`

## Mevcut Etiketler

- `latest` - En son kararlı sürüm
- `v1.x.x` - Belirli bir sürüm
- `main` - En son geliştirme sürümü

## Docker Hub

**Repository**: [ferdiunal/soketi-rs](https://hub.docker.com/r/funal/soketi-rs)

---

Detaylı dokümantasyon için [Başlangıç](baslangic.md) ve [Yapılandırma](yapilandirma.md) sayfalarına bakın.
