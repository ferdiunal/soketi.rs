# Soketi.rs - Yüksek Performanslı WebSocket Sunucusu

![Soketi.rs Cover](art/cover.png)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Durumu](https://img.shields.io/github/actions/workflow/status/ferdiunal/soketi-rs/docker-publish.yml?branch=main)](https://github.com/ferdiunal/soketi-rs/actions)
[![Versiyon](https://img.shields.io/github/v/release/ferdiunal/soketi-rs)](https://github.com/ferdiunal/soketi-rs/releases)
[![Lisans](https://img.shields.io/badge/lisans-GPL--3.0-blue.svg)](LICENSE)
[![Docker](https://img.shields.io/docker/pulls/ferdiunal/soketi-rs)](https://hub.docker.com/r/funal/soketi-rs)
[![Docker İmaj Boyutu](https://img.shields.io/docker/image-size/ferdiunal/soketi-rs/latest)](https://hub.docker.com/r/funal/soketi-rs)

Rust ile yazılmış, yüksek performanslı, Pusher uyumlu WebSocket sunucusu. Soketi.rs, public, private ve presence kanalları desteğiyle gerçek zamanlı mesajlaşma yetenekleri sağlar.

[🇬🇧 English Documentation](README.en.md)

## ✨ Özellikler

- **🚀 Yüksek Performans**: Maksimum hız ve verimlilik için Rust ile geliştirildi
- **📡 Pusher Protokolü**: Pusher istemci kütüphaneleriyle %100 uyumlu
- **🔐 Kimlik Doğrulama**: Private ve presence kanalları için yerleşik destek
- **👥 Presence Kanalları**: Gerçek zamanlı kullanıcı takibi ve üye listeleri
- **💬 İstemci Olayları**: Doğrudan istemciden istemciye mesajlaşma
- **📊 Metrikler**: İzleme için Prometheus metrikleri
- **🔄 Yatay Ölçekleme**: Çoklu sunucu dağıtımları için Redis adaptörü
- **🗄️ Çoklu Backend**: MySQL, PostgreSQL, DynamoDB desteği
- **🎯 Hız Sınırlama**: Uygulama başına yapılandırılabilir hız limitleri
- **🪝 Webhook'lar**: HTTP callback'leri ile olay bildirimleri
- **🐳 Docker Hazır**: Production-ready Docker imajları

## 📋 İçindekiler

- [Hızlı Başlangıç](#hızlı-başlangıç)
- [Kurulum](#kurulum)
- [Yapılandırma](#yapılandırma)
- [Kullanım Örnekleri](#kullanım-örnekleri)
- [Docker Dağıtımı](#docker-dağıtımı)
- [Dokümantasyon](#dokümantasyon)
- [API Dokümantasyonu](#api-dokümantasyonu)
- [İstemci Kütüphaneleri](#istemci-kütüphaneleri)
- [Mimari](#mimari)
- [Performans](#performans)
- [Katkıda Bulunma](#katkıda-bulunma)
- [Lisans](#lisans)

## 🚀 Hızlı Başlangıç

### Docker Kullanarak (En Hızlı)

```bash
# Docker Hub'dan en son imajı çekin ve çalıştırın
docker pull funal/soketi-rs:latest

docker run -d \
  --name soketi \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest
```

### Docker Compose Kullanarak (Önerilen)

```bash
# Repository'yi klonlayın
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs

# Tüm servisleri başlatın
docker-compose up -d

# Logları görüntüleyin
docker-compose logs -f soketi

# Demo'ya erişin
open http://localhost:3000
```

### Cargo Kullanarak

```bash
# Rust'ı yükleyin (henüz yüklü değilse)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Derleyin ve çalıştırın
cargo build --release
./target/release/soketi-rs --config-file config.json
```

> **Not:** Yapılandırma dosyasını belirtmek için `--config-file` parametresini kullanın.

## 📦 Kurulum

### Gereksinimler

- **Rust 1.75+** (kaynak koddan derleme için)
- **Docker & Docker Compose** (konteyner dağıtımı için)
- **Redis** (opsiyonel, kümeleme için)
- **PostgreSQL/MySQL/DynamoDB** (opsiyonel, uygulama yönetimi için)

### Kaynak Koddan

```bash
# Repository'yi klonlayın
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs

# Release binary'sini derleyin
cargo build --release

# Testleri çalıştırın
cargo test

# Binary'yi yükleyin
cargo install --path .
```

### Docker Kullanarak

```bash
# İmajı çekin
docker pull funal/soketi-rs:latest

# Konteyneri çalıştırın
docker run -d \
  -p 6001:6001 \
  -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  funal/soketi-rs:latest
```

## ⚙️ Yapılandırma

### Temel Yapılandırma

Bir `config.json` dosyası oluşturun:

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "debug": false,
  "app_manager": {
    "driver": "Array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "enabled": true,
          "enable_client_messages": true,
          "max_connections": 10000
        }
      ]
    }
  }
}
```

### Ortam Değişkenleri

```bash
# Sunucu yapılandırması
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001
SOKETI_DEBUG=false

# Uygulama yapılandırması
APP_ID=your-app-id
APP_KEY=your-app-key
APP_SECRET=your-app-secret

# Redis (kümeleme için)
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_DB=0

# Metrikler
METRICS_ENABLED=true
METRICS_PORT=9601
```

### Gelişmiş Yapılandırma

Detaylı yapılandırma seçenekleri için [Yapılandırma Kılavuzu](docs/tr/yapilandirma.md) dosyasına bakın:
- Uygulama yönetimi (Array, MySQL, PostgreSQL, DynamoDB)
- Adaptörler (Local, Redis, NATS, Cluster)
- Hız sınırlama ve webhook'lar
- Metrikler ve izleme
- SSL/TLS yapılandırması

## 💡 Kullanım Örnekleri

### Temel Chat Uygulaması

```javascript
// Pusher istemcisini başlat
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false
});

// Public kanala abone ol
const channel = pusher.subscribe('chat-room');

// Mesajları dinle
channel.bind('new-message', (data) => {
  console.log('Yeni mesaj:', data.message);
  displayMessage(data);
});

// Mesaj gönder (istemci olayları etkinleştirilmiş olmalı)
channel.trigger('client-new-message', {
  user: 'Ahmet',
  message: 'Herkese merhaba!'
});
```

### Kimlik Doğrulama ile Private Kanallar

```javascript
// Private kanala abone ol
const privateChannel = pusher.subscribe('private-user-123');

// Sunucu tarafı kimlik doğrulama endpoint'i
app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channel = req.body.channel_name;
  
  // Kullanıcının bu kanala erişimi olup olmadığını doğrula
  if (userCanAccessChannel(req.user, channel)) {
    const auth = pusher.authenticate(socketId, channel);
    res.send(auth);
  } else {
    res.status(403).send('Yasak');
  }
});
```

### Presence Kanalları

```javascript
// Presence kanalına abone ol
const presenceChannel = pusher.subscribe('presence-team');

// Üye eklendi
presenceChannel.bind('pusher:member_added', (member) => {
  console.log('Kullanıcı katıldı:', member.info.name);
  updateUserList();
});

// Üye çıkarıldı
presenceChannel.bind('pusher:member_removed', (member) => {
  console.log('Kullanıcı ayrıldı:', member.info.name);
  updateUserList();
});

// Mevcut üyeleri al
presenceChannel.bind('pusher:subscription_succeeded', (members) => {
  members.each((member) => {
    console.log('Çevrimiçi:', member.info.name);
  });
});
```

### Sunucudan Olay Tetikleme

```javascript
// Node.js Pusher kütüphanesini kullanarak
const Pusher = require('pusher');

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false
});

// Bir olay tetikle
pusher.trigger('my-channel', 'my-event', {
  message: 'Sunucudan merhaba!'
});

// Birden fazla kanala tetikle
pusher.trigger(['channel-1', 'channel-2'], 'my-event', {
  message: 'Yayın mesajı'
});
```

Daha fazla örnek için:
- [Temel Chat Örneği](docs/tr/ornekler/temel-chat.md)
- [Kimlik Doğrulama Örnekleri](docs/tr/ornekler/kimlik-dogrulama.md)
- [Özel Kanallar](docs/tr/ornekler/ozel-kanallar.md)
- [Presence Kanalları](docs/tr/ornekler/presence.md)

## 🐳 Docker Dağıtımı

Tüm deployment dosyaları `deployment/` dizininde organize edilmiştir:

```
deployment/
├── docker/    # Standart Docker deployment
├── nginx/     # Nginx reverse proxy ile
└── caddy/     # Caddy reverse proxy ile (otomatik HTTPS)
```

### Standart Docker Deployment

```bash
cd deployment/docker

# Servisleri başlat
docker-compose up -d

# Logları görüntüle
docker-compose logs -f

# Servisleri durdur
docker-compose down
```

### Nginx ile Production Deployment

```bash
cd deployment/nginx

# SSL sertifikalarını yapılandır
cp .env.nginx.example .env
# .env dosyasını düzenle

# Servisleri başlat
docker-compose -f docker-compose.nginx.yml up -d
```

### Caddy ile Production Deployment (Otomatik HTTPS)

```bash
cd deployment/caddy

# Domain yapılandır
cp .env.caddy.example .env
# .env dosyasını düzenle

# Servisleri başlat (otomatik SSL!)
docker-compose -f docker-compose.caddy.yml up -d
```

### Ölçeklendirme

```bash
# Soketi instance'larını ölçeklendirin
docker-compose up -d --scale soketi=3

# Servis durumunu görüntüleyin
docker-compose ps
```

### İzleme Stack'i ile

```bash
# Prometheus ve Grafana ile başlatın
docker-compose --profile monitoring up -d

# Grafana'ya erişin
open http://localhost:3001
# Varsayılan kimlik bilgileri: admin/admin
```

### Production En İyi Uygulamaları

1. **Kümeleme için Redis kullanın**:
   ```json
   {
     "adapter": {
       "driver": "Redis",
       "redis": {
         "host": "redis",
         "port": 6379
       }
     }
   }
   ```

2. **Metrikleri etkinleştirin**:
   ```json
   {
     "metrics": {
       "enabled": true,
       "driver": "Prometheus",
       "port": 9601
     }
   }
   ```

3. **Kaynak limitlerini yapılandırın** `docker-compose.yml` dosyasında

4. **Otomatik kurtarma için health check'leri kullanın**

5. **Log toplama ayarlayın** (ELK, Loki, vb.)

## 📚 Dokümantasyon

Kapsamlı dokümantasyon birden fazla dilde mevcuttur:

### İngilizce Dokümantasyon
- [Getting Started](docs/en/getting-started.md) - Hızlı başlangıç kılavuzu ve temel kavramlar
- [Installation](docs/en/installation.md) - Detaylı kurulum talimatları
- [Configuration](docs/en/configuration.md) - Tam yapılandırma referansı
- [API Reference](docs/en/api-reference.md) - HTTP ve WebSocket API dokümantasyonu
- [Troubleshooting](docs/en/troubleshooting.md) - Yaygın sorunlar ve çözümler

#### Deployment Kılavuzları
- [Vercel Deployment](docs/en/deployment/vercel.md) - Vercel'e deployment
- [Netlify Deployment](docs/en/deployment/netlify.md) - Netlify'a deployment
- [Reverse Proxy Setup](docs/en/deployment/reverse-proxy.md) - HTTP/2 ve HTTP/3 ile Caddy ve Nginx yapılandırması

#### Kod Örnekleri
- [Basic Chat](docs/en/examples/basic-chat.md) - Basit chat uygulaması
- [Authentication](docs/en/examples/authentication.md) - Kullanıcı kimlik doğrulama kalıpları
- [Private Channels](docs/en/examples/private-channels.md) - Güvenli özel mesajlaşma
- [Presence Channels](docs/en/examples/presence.md) - Gerçek zamanlı kullanıcı varlığı

### Türkçe Dokümantasyon
- [Başlangıç](docs/tr/baslangic.md) - Hızlı başlangıç kılavuzu ve temel kavramlar
- [Kurulum](docs/tr/kurulum.md) - Detaylı kurulum talimatları
- [Yapılandırma](docs/tr/yapilandirma.md) - Tam yapılandırma referansı
- [API Referansı](docs/tr/api-referans.md) - HTTP ve WebSocket API dokümantasyonu
- [Sorun Giderme](docs/tr/sorun-giderme.md) - Yaygın sorunlar ve çözümler

#### Deployment Kılavuzları
- [Vercel Deployment](docs/tr/deployment/vercel.md) - Vercel'e deployment
- [Netlify Deployment](docs/tr/deployment/netlify.md) - Netlify'a deployment
- [Reverse Proxy Kurulumu](docs/tr/deployment/reverse-proxy.md) - HTTP/2 ve HTTP/3 ile Caddy ve Nginx yapılandırması

#### Kod Örnekleri
- [Temel Chat](docs/tr/ornekler/temel-chat.md) - Basit chat uygulaması
- [Kimlik Doğrulama](docs/tr/ornekler/kimlik-dogrulama.md) - Kullanıcı kimlik doğrulama kalıpları
- [Özel Kanallar](docs/tr/ornekler/ozel-kanallar.md) - Güvenli özel mesajlaşma
- [Presence](docs/tr/ornekler/presence.md) - Gerçek zamanlı kullanıcı varlığı

### İleri Düzey Konular
- [MySQL Kurulumu](docs/MYSQL_SETUP.md) - MySQL uygulama yöneticisi yapılandırması
- [PostgreSQL Kurulumu](docs/POSTGRES_SETUP.md) - PostgreSQL uygulama yöneticisi yapılandırması
- [DynamoDB Kurulumu](docs/DYNAMODB_SETUP.md) - DynamoDB uygulama yöneticisi yapılandırması
- [Redis Adaptörü](docs/REDIS_ADAPTER_IMPLEMENTATION.md) - Redis kümeleme kurulumu
- [NATS Adaptörü](docs/NATS_ADAPTER_IMPLEMENTATION.md) - NATS mesajlaşma entegrasyonu
- [Cluster Adaptörü](docs/CLUSTER_ADAPTER_IMPLEMENTATION.md) - Yerel kümeleme
- [Lambda Webhook'ları](docs/LAMBDA_WEBHOOKS.md) - AWS Lambda webhook entegrasyonu
- [SQS Kuyruk Yöneticisi](docs/sqs_queue_manager.md) - AWS SQS kuyruk yapılandırması

## 📚 API Dokümantasyonu

### WebSocket Bağlantısı

```javascript
// Pusher.js kullanarak
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false
});

// Kanala abone olun
const channel = pusher.subscribe('my-channel');

// Olayları dinleyin
channel.bind('my-event', (data) => {
  console.log('Alındı:', data);
});
```

### HTTP API

#### Olay Tetikleme

```bash
POST /apps/{app_id}/events
Content-Type: application/json

{
  "name": "my-event",
  "channel": "my-channel",
  "data": "{\"message\":\"Merhaba Dünya\"}"
}
```

#### Kanalları Getir

```bash
GET /apps/{app_id}/channels
```

#### Kanal Bilgisi Getir

```bash
GET /apps/{app_id}/channels/{channel_name}
```

Tam API dokümantasyonu için [API Referansı](docs/tr/api-referans.md) dosyasına bakın.

## 📱 İstemci Kütüphaneleri

Soketi.rs tüm Pusher istemci kütüphaneleriyle uyumludur:

- **JavaScript**: [pusher-js](https://github.com/pusher/pusher-js)
- **Laravel**: [Laravel Echo](https://github.com/laravel/echo)
- **iOS**: [pusher-websocket-swift](https://github.com/pusher/pusher-websocket-swift)
- **Android**: [pusher-websocket-java](https://github.com/pusher/pusher-websocket-java)
- **Python**: [pusher-http-python](https://github.com/pusher/pusher-http-python)
- **PHP**: [pusher-http-php](https://github.com/pusher/pusher-http-php)
- **Ruby**: [pusher-http-ruby](https://github.com/pusher/pusher-http-ruby)
- **Go**: [pusher-http-go](https://github.com/pusher/pusher-http-go)

## 🏗️ Mimari

```
┌─────────────────┐
│  İstemci Uyg.   │
│  (Web/Mobil)    │
└────────┬────────┘
         │ WebSocket
         ▼
┌─────────────────┐
│ Soketi Sunucu   │
│     (Rust)      │
└────────┬────────┘
         │
    ┌────┴────┬────────┬─────────┐
    ▼         ▼        ▼         ▼
┌───────┐ ┌───────┐ ┌──────┐ ┌────────┐
│ Redis │ │  VT   │ │Metrik│ │Webhook │
└───────┘ └───────┘ └──────┘ └────────┘
```

### Bileşenler

- **WebSocket Sunucusu**: İstemci bağlantılarını yönetir
- **HTTP API**: Olay tetikleme için REST API
- **Adaptör**: Mesaj dağıtımı (Local/Redis/NATS/Cluster)
- **Uygulama Yöneticisi**: Uygulama yapılandırması (Array/MySQL/PostgreSQL/DynamoDB)
- **Önbellek Yöneticisi**: Önbellekleme katmanı (Memory/Redis)
- **Hız Sınırlayıcı**: İstek hız sınırlama (Local/Redis)
- **Kuyruk Yöneticisi**: Webhook kuyruğu (Sync/Redis/SQS)
- **Metrikler**: Prometheus metrik dışa aktarıcı

## 📊 Performans

### Benchmark'lar

- **Bağlantılar**: Instance başına 100.000+ eşzamanlı bağlantı
- **Mesajlar**: Saniyede 1M+ mesaj
- **Gecikme**: <1ms ortalama mesaj gecikmesi
- **Bellek**: ~50MB temel + bağlantı başına ~1KB
- **CPU**: Verimli çok çekirdekli kullanım

### Optimizasyon İpuçları

1. Yatay ölçekleme için Redis adaptörü kullanın
2. Bağlantı havuzlamayı etkinleştirin
3. Uygun hız limitlerini yapılandırın
4. Kalıcılık için SSD depolama kullanın
5. İzleme için Prometheus metriklerini etkinleştirin

## 🧪 Test

```bash
# Tüm testleri çalıştır
cargo test

# Belirli bir testi çalıştır
cargo test test_name

# Loglama ile çalıştır
RUST_LOG=debug cargo test

# Benchmark'ları çalıştır
cargo bench
```

## 🤝 Katkıda Bulunma

Katkılar memnuniyetle karşılanır! Detaylar için [CONTRIBUTING.md](CONTRIBUTING.md) dosyasını okuyun.

### Geliştirme Ortamı Kurulumu

```bash
# Repository'yi klonlayın
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs

# Bağımlılıkları yükleyin
cargo build

# Testleri çalıştırın
cargo test

# Demo'yu çalıştırın
cargo run -- --config-file test-config.json
```

## 📄 Lisans

Bu proje GPL-3.0 Lisansı altında lisanslanmıştır - detaylar için [LICENSE](LICENSE) dosyasına bakın.

## 🙏 Teşekkürler

- [Soketi](https://github.com/soketi/soketi)'den ilham alınmıştır
- [Tokio](https://tokio.rs/) ile geliştirilmiştir
- [Pusher Protokolü](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol) ile uyumludur

## 📞 Destek

- **Dokümantasyon**: [English](docs/en/getting-started.md) | [Türkçe](docs/tr/baslangic.md)
- **Sorunlar**: [GitHub Issues](https://github.com/ferdiunal/soketi-rs/issues)
- **Tartışmalar**: [GitHub Discussions](https://github.com/ferdiunal/soketi-rs/discussions)
- **Discord**: [Discord'umuza katılın](https://discord.gg/soketi)

---

[Ferdi ÜNAL](https://github.com/ferdiunal) tarafından ❤️ ile yapıldı
