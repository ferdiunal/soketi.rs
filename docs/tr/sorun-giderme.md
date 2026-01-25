# Sorun Giderme Kılavuzu

> soketi.rs WebSocket sunucusu ile ilgili yaygın sorunları teşhis etme ve çözme konusunda kapsamlı kılavuz

## İçindekiler

- [Bağlantı Sorunları](#bağlantı-sorunları)
- [Kimlik Doğrulama Sorunları](#kimlik-doğrulama-sorunları)
- [Kanal Abonelik Hataları](#kanal-abonelik-hataları)
- [Performans Sorunları](#performans-sorunları)
- [Deployment Sorunları](#deployment-sorunları)
- [Yapılandırma Hataları](#yapılandırma-hataları)
- [Debug Teknikleri](#debug-teknikleri)
- [İzleme ve Loglama](#izleme-ve-loglama)
- [Yaygın Hata Mesajları](#yaygın-hata-mesajları)
- [Yardım Alma](#yardım-alma)

## Bağlantı Sorunları

### WebSocket Bağlantısı Başarısız

**Belirti:**
```
Error: WebSocket connection to 'ws://localhost:6001/app/...' failed
```

**Olası Nedenler:**
1. soketi.rs sunucusu çalışmıyor
2. Yanlış host veya port yapılandırması
3. Firewall WebSocket bağlantılarını engelliyor
4. SSL/TLS yapılandırma uyuşmazlığı

**Çözümler:**

**1. Sunucunun Çalıştığını Doğrulayın:**
```bash
# soketi.rs'nin çalışıp çalışmadığını kontrol edin
curl http://localhost:6001/health

# Docker container durumunu kontrol edin
docker ps | grep soketi

# Process'i kontrol edin
ps aux | grep soketi
```

**2. Yapılandırmayı Doğrulayın:**
```typescript
// Pusher client yapılandırmanızı kontrol edin
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',  // Sunucu host'unuzla eşleşmeli
  wsPort: 6001,         // Sunucu port'unuzla eşleşmeli
  forceTLS: false,      // HTTPS kullanıyorsanız true yapın
  encrypted: false,     // HTTPS kullanıyorsanız true yapın
  enabledTransports: ['ws', 'wss'],
});
```

**3. Firewall Kurallarını Kontrol Edin:**
```bash
# Linux - Port'un açık olup olmadığını kontrol edin
sudo netstat -tulpn | grep 6001

# Firewall'dan port'a izin verin (Ubuntu/Debian)
sudo ufw allow 6001/tcp

# Port'un erişilebilir olup olmadığını kontrol edin
telnet localhost 6001
```

**4. SSL/TLS Uyuşmazlığı:**
```typescript
// Sunucunuz HTTPS kullanıyorsa, client'ın eşleştiğinden emin olun
const pusher = new Pusher('app-key', {
  wsHost: 'your-domain.com',
  wsPort: 443,
  wssPort: 443,
  forceTLS: true,      // HTTPS için true olmalı
  encrypted: true,     // HTTPS için true olmalı
  enabledTransports: ['wss'],  // Güvenli bağlantılar için 'wss' kullanın
});
```

### Bağlantı Sık Sık Kopuyor

**Belirti:**
Client sürekli bağlantıyı kesiyor ve yeniden bağlanıyor.

**Olası Nedenler:**
1. Ağ kararsızlığı
2. Timeout ayarları çok agresif
3. Load balancer timeout sorunları
4. Sunucu kaynak kısıtlamaları

**Çözümler:**

**1. Timeout Ayarlarını Düzenleyin:**
```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  activityTimeout: 120000,  // 120 saniyeye çıkarın
  pongTimeout: 30000,       // 30 saniyeye çıkarın
});
```

**2. Load Balancer Timeout'larını Yapılandırın:**
```nginx
# Nginx yapılandırması
location /app/ {
    proxy_pass http://soketi_backend;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    
    # Timeout'ları artırın
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

**3. Sunucu Kaynaklarını İzleyin:**
```bash
# CPU ve bellek kullanımını kontrol edin
top

# soketi.rs kaynak kullanımını kontrol edin
docker stats soketi

# Sistem loglarını kontrol edin
journalctl -u soketi -f
```

### Tarayıcıdan Bağlanılamıyor

**Belirti:**
Bağlantı sunucu/backend'den çalışıyor ancak tarayıcıdan başarısız oluyor.

**Olası Nedenler:**
1. CORS yapılandırma sorunları
2. Karışık içerik (HTTP sayfası WS'ye bağlanmaya çalışıyor)
3. Tarayıcı güvenlik politikaları

**Çözümler:**

**1. CORS'u Yapılandırın:**
```bash
# Environment variable
SOKETI_CORS_ALLOWED_ORIGINS=https://your-domain.com,https://app.your-domain.com

# Veya config.json'da
{
  "cors": {
    "credentials": true,
    "origin": ["https://your-domain.com"],
    "methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
  }
}
```

**2. Karışık İçeriği Düzeltin:**
```typescript
// Sayfanız HTTPS ise, WSS kullanın
const pusher = new Pusher('app-key', {
  wsHost: 'your-domain.com',
  forceTLS: true,  // Güvenli bağlantıyı zorla
  encrypted: true,
});
```

**3. Tarayıcı Konsolunu Kontrol Edin:**
```javascript
// Pusher debug loglamasını etkinleştirin
Pusher.logToConsole = true;

const pusher = new Pusher('app-key', {
  // ... config
});
```

## Kimlik Doğrulama Sorunları

### Private Channel Yetkilendirmesi Başarısız

**Belirti:**
```
Error: Unable to retrieve auth string from auth endpoint
```

**Olası Nedenler:**
1. Auth endpoint yapılandırılmamış veya yanlış
2. Auth endpoint yanlış format döndürüyor
3. Session/token geçerli değil
4. Auth endpoint ile CORS sorunları

**Çözümler:**

**1. Auth Endpoint Yapılandırmasını Doğrulayın:**
```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  authEndpoint: '/api/pusher/auth',  // Doğru path olmalı
  auth: {
    headers: {
      'Authorization': `Bearer ${token}`  // Auth token'ı ekleyin
    }
  }
});
```

**2. Auth Endpoint Yanıtını Kontrol Edin:**
```typescript
// Doğru auth endpoint yanıt formatı
export async function POST(req: Request) {
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  // Private channel'lar için
  const authSignature = pusher.authorizeChannel(socketId, channelName);
  
  // Bu tam formatı döndürmelidir
  return Response.json({
    auth: authSignature.auth
  });
}
```

**3. Auth Endpoint'i Manuel Test Edin:**
```bash
# Auth endpoint'inizi test edin
curl -X POST http://localhost:3000/api/pusher/auth \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -H "Authorization: Bearer your-token" \
  -d "socket_id=123.456&channel_name=private-test"

# Beklenen yanıt:
# {"auth":"app-key:signature"}
```

**4. Auth Endpoint için CORS'u Etkinleştirin:**
```typescript
// app/api/pusher/auth/route.ts
export async function OPTIONS(req: Request) {
  return new Response(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  });
}
```

### Presence Channel Kullanıcı Bilgisi Görünmüyor

**Belirti:**
Kullanıcı presence channel'a başarıyla abone oluyor ancak kullanıcı bilgisi eksik.

**Olası Nedenler:**
1. Auth endpoint user_info döndürmüyor
2. Yanlış presence data formatı

**Çözümler:**

**1. Doğru Presence Data Döndürün:**
```typescript
// app/api/pusher/auth/route.ts
export async function POST(req: Request) {
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  if (channelName?.startsWith('presence-')) {
    // Presence channel'lar için presenceData eklenmeli
    const presenceData = {
      user_id: session.user.id,
      user_info: {
        name: session.user.name,
        email: session.user.email,
        avatar: session.user.avatar,
      }
    };
    
    const authResponse = pusher.authorizeChannel(
      socketId,
      channelName,
      presenceData  // Bu gereklidir!
    );
    
    return Response.json(authResponse);
  }
}
```

**2. Client-Side İşlemeyi Doğrulayın:**
```typescript
const channel = pusher.subscribe('presence-chat');

channel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Üyeler:', members.count);
  members.each((member: any) => {
    console.log('Üye ID:', member.id);
    console.log('Üye Bilgisi:', member.info);  // user_info içermeli
  });
});
```

### Geçersiz İmza Hatası

**Belirti:**
```
Error: Invalid signature
```

**Olası Nedenler:**
1. Client ve sunucu arasında app secret uyuşmazlığı
2. Yanlış imza oluşturma
3. Sunucular arasında saat farkı

**Çözümler:**

**1. App Kimlik Bilgilerini Doğrulayın:**
```bash
# Sunucu tarafı (soketi.rs)
SOKETI_DEFAULT_APP_ID=app-id
SOKETI_DEFAULT_APP_KEY=app-key
SOKETI_DEFAULT_APP_SECRET=app-secret

# Client tarafı
NEXT_PUBLIC_PUSHER_KEY=app-key  # Sunucu ile eşleşmeli

# Server SDK (auth endpoint için)
PUSHER_APP_ID=app-id
PUSHER_SECRET=app-secret  # Sunucu ile eşleşmeli
```

**2. Pusher Server Yapılandırmasını Kontrol Edin:**
```typescript
// Server SDK'nın doğru kimlik bilgilerini kullandığından emin olun
const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID!,
  key: process.env.NEXT_PUBLIC_PUSHER_KEY!,
  secret: process.env.PUSHER_SECRET!,  // soketi.rs secret ile eşleşmeli
  host: process.env.PUSHER_HOST,
  port: process.env.PUSHER_PORT,
  useTLS: true,
});
```

**3. Sunucu Saatlerini Senkronize Edin:**
```bash
# NTP'yi yükleyin
sudo apt-get install ntp

# Zamanı senkronize edin
sudo ntpdate -s time.nist.gov

# NTP servisini etkinleştirin
sudo systemctl enable ntp
sudo systemctl start ntp
```

## Kanal Abonelik Hataları

### Abonelik Timeout

**Belirti:**
```
Error: Subscription timeout
```

**Olası Nedenler:**
1. Sunucu abonelik isteğine yanıt vermiyor
2. Ağ gecikmesi çok yüksek
3. Sunucu aşırı yüklenmiş

**Çözümler:**

**1. Abonelik Timeout'unu Artırın:**
```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  activityTimeout: 120000,  // 120 saniye
});
```

**2. Sunucu Loglarını Kontrol Edin:**
```bash
# Docker logları
docker logs soketi -f

# Abonelik hatalarını kontrol edin
docker logs soketi 2>&1 | grep -i "subscription"
```

**3. Sunucu Yükünü İzleyin:**
```bash
# Sunucunun aşırı yüklenip yüklenmediğini kontrol edin
docker stats soketi

# Gerekirse ölçeklendirin
docker-compose up --scale soketi=3
```

### Client Event'ler Çalışmıyor

**Belirti:**
Client event'ler (client-*) diğer client'lar tarafından alınmıyor.

**Olası Nedenler:**
1. App yapılandırmasında client event'ler etkin değil
2. Public channel'larda client event kullanımı (izin verilmez)
3. Event adı "client-" ile başlamıyor

**Çözümler:**

**1. Client Event'leri Etkinleştirin:**
```json
// config.json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "enable_client_messages": true  // True olmalı
        }
      ]
    }
  }
}
```

**2. Private veya Presence Channel'ları Kullanın:**
```typescript
// Client event'ler sadece private/presence channel'larda çalışır
const channel = pusher.subscribe('private-chat');  // ✅ Çalışır
// const channel = pusher.subscribe('public-chat');  // ❌ Çalışmaz

channel.bind('pusher:subscription_succeeded', () => {
  // Client event tetikle ('client-' ile başlamalı)
  channel.trigger('client-typing', { user: 'John', isTyping: true });
});

// Client event'leri dinle
channel.bind('client-typing', (data) => {
  console.log('Kullanıcı yazıyor:', data);
});
```

**3. Event Adı Formatını Kontrol Edin:**
```typescript
// ✅ Doğru - 'client-' ile başlıyor
channel.trigger('client-message', data);
channel.trigger('client-typing', data);

// ❌ Yanlış - 'client-' ile başlamıyor
channel.trigger('message', data);  // Client event olarak çalışmaz
```

### Maksimum Bağlantı Sayısı Aşıldı

**Belirti:**
```
Error: Connection limit exceeded
```

**Olası Nedenler:**
1. Çok fazla eşzamanlı bağlantı
2. Bağlantı limiti çok düşük ayarlanmış
3. Bağlantılar düzgün kapatılmıyor

**Çözümler:**

**1. Bağlantı Limitini Artırın:**
```json
// config.json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",
          "key": "app-key",
          "secret": "app-secret",
          "max_connections": 1000  // Limiti artırın
        }
      ]
    }
  }
}
```

**2. Client'ları Düzgün Bağlantıyı Kesin:**
```typescript
// Component unmount olduğunda bağlantıyı kesin
useEffect(() => {
  const pusher = new Pusher('app-key', { /* config */ });
  const channel = pusher.subscribe('my-channel');

  return () => {
    channel.unbind_all();
    pusher.unsubscribe('my-channel');
    pusher.disconnect();  // Önemli!
  };
}, []);
```

**3. Aktif Bağlantıları İzleyin:**
```bash
# Metrics endpoint'i kontrol edin
curl http://localhost:9601/metrics

# Bağlantı sayısına bakın
# soketi_connections_total
```

## Performans Sorunları

### Yüksek Gecikme

**Belirti:**
Mesajların gelmesi birkaç saniye sürüyor.

**Olası Nedenler:**
1. Ağ gecikmesi
2. Sunucu aşırı yüklenmiş
3. Verimsiz event işleme
4. Coğrafi mesafe

**Çözümler:**

**1. Kullanıcılara Daha Yakın Deploy Edin:**
```bash
# CDN kullanın veya birden fazla bölgede deploy edin
# Kullanıcılarınıza en yakın bölgeyi seçin
```

**2. Event İşlemeyi Optimize Edin:**
```typescript
// ❌ Kötü - Event handler'da işleme
channel.bind('message', (data) => {
  // Ağır işlemler burada diğer event'leri bloklar
  processHeavyData(data);
});

// ✅ İyi - Worker'a yükle
channel.bind('message', (data) => {
  // Arka plan işleme için kuyruğa al
  queueForProcessing(data);
});
```

**3. Connection Pooling Kullanın:**
```typescript
// Tek Pusher instance'ını yeniden kullanın
const pusher = new Pusher('app-key', { /* config */ });

// Her component için yeni instance oluşturmayın
// ❌ Kötü
function Component1() {
  const pusher = new Pusher('app-key', { /* config */ });
}

// ✅ İyi - Paylaşılan instance'ı import edin
import { pusher } from '@/lib/pusher';
```

**4. Sıkıştırmayı Etkinleştirin:**
```nginx
# Nginx yapılandırması
gzip on;
gzip_vary on;
gzip_proxied any;
gzip_comp_level 6;
gzip_types text/plain application/json;
```

### Yüksek Bellek Kullanımı

**Belirti:**
soketi.rs aşırı bellek tüketiyor.

**Olası Nedenler:**
1. Çok fazla eşzamanlı bağlantı
2. Büyük mesaj payload'ları
3. Bellek sızıntısı
4. Yetersiz garbage collection

**Çözümler:**

**1. Mesaj Boyutunu Sınırlayın:**
```json
// config.json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "max_event_payload_in_kb": 100  // 100KB ile sınırla
        }
      ]
    }
  }
}
```

**2. Bellek Kullanımını İzleyin:**
```bash
# Bellek kullanımını kontrol edin
docker stats soketi

# Bellek limitlerini ayarlayın
docker run -m 512m soketi
```

**3. Periyodik Olarak Yeniden Başlatın:**
```bash
# Docker restart policy kullanın
docker run --restart=unless-stopped soketi

# Veya periyodik yeniden başlatmalar için systemd timer kullanın
```

**4. Yatay Ölçeklendirin:**
```yaml
# docker-compose.yml
version: '3.8'
services:
  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    deploy:
      replicas: 3  # Birden fazla instance çalıştırın
      resources:
        limits:
          memory: 512M
```

### Yavaş Event Yayını

**Belirti:**
Event'lerin tüm abonelere yayınlanması uzun sürüyor.

**Olası Nedenler:**
1. Tek thread'li darboğaz
2. Yatay ölçeklendirme yok
3. Verimsiz adapter

**Çözümler:**

**1. Redis Adapter Kullanın:**
```bash
# Yatay ölçeklendirme için Redis'i etkinleştirin
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=redis
SOKETI_REDIS_PORT=6379
SOKETI_REDIS_DB=0
```

**2. Birden Fazla Instance Deploy Edin:**
```yaml
# docker-compose.yml
version: '3.8'
services:
  redis:
    image: redis:alpine
    
  soketi-1:
    image: quay.io/soketi/soketi:latest-16-alpine
    environment:
      - SOKETI_ADAPTER_DRIVER=redis
      - SOKETI_REDIS_HOST=redis
      
  soketi-2:
    image: quay.io/soketi/soketi:latest-16-alpine
    environment:
      - SOKETI_ADAPTER_DRIVER=redis
      - SOKETI_REDIS_HOST=redis
      
  nginx:
    image: nginx:alpine
    ports:
      - "6001:6001"
    depends_on:
      - soketi-1
      - soketi-2
```

**3. Event Payload'unu Optimize Edin:**
```typescript
// ❌ Kötü - Büyük payload
channel.trigger('update', {
  fullDocument: largeObject,
  metadata: moreData,
  history: evenMoreData
});

// ✅ İyi - Minimal payload
channel.trigger('update', {
  id: documentId,
  type: 'modified'
});
// Client gerekirse tam veriyi çeker
```

## Deployment Sorunları

### Docker Container Başlamıyor

**Belirti:**
```
Error: Container exits immediately after starting
```

**Olası Nedenler:**
1. Yapılandırma hatası
2. Port zaten kullanımda
3. Eksik environment variable'lar
4. Geçersiz kimlik bilgileri

**Çözümler:**

**1. Container Loglarını Kontrol Edin:**
```bash
# Logları görüntüleyin
docker logs soketi

# Logları gerçek zamanlı takip edin
docker logs -f soketi
```

**2. Port Kullanılabilirliğini Kontrol Edin:**
```bash
# Port'un zaten kullanımda olup olmadığını kontrol edin
sudo lsof -i :6001

# Port'u kullanan process'i sonlandırın
sudo kill -9 <PID>

# Veya farklı port kullanın
docker run -p 6002:6001 soketi
```

**3. Environment Variable'ları Doğrulayın:**
```bash
# Gerekli değişkenlerin ayarlandığını kontrol edin
docker run --rm soketi env | grep SOKETI

# Açık değişkenlerle çalıştırın
docker run \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  -p 6001:6001 \
  quay.io/soketi/soketi:latest-16-alpine
```

**4. Yapılandırmayı Test Edin:**
```bash
# Minimal config ile test edin
docker run -p 6001:6001 \
  -e SOKETI_DEFAULT_APP_ID=test \
  -e SOKETI_DEFAULT_APP_KEY=test \
  -e SOKETI_DEFAULT_APP_SECRET=test \
  quay.io/soketi/soketi:latest-16-alpine
```

### Reverse Proxy Çalışmıyor

**Belirti:**
Nginx/Caddy reverse proxy üzerinden bağlanılamıyor.

**Olası Nedenler:**
1. WebSocket upgrade header'ları eksik
2. Yanlış proxy yapılandırması
3. SSL/TLS sorunları
4. Timeout çok kısa

**Çözümler:**

**1. Nginx Yapılandırmasını Doğrulayın:**
```nginx
location /app/ {
    proxy_pass http://soketi:6001;
    proxy_http_version 1.1;
    
    # WebSocket için gerekli
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    
    # Standart header'lar
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    
    # Timeout'lar
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;
}
```

**2. Nginx Yapılandırmasını Test Edin:**
```bash
# Yapılandırma syntax'ını test edin
nginx -t

# Yapılandırmayı yeniden yükleyin
nginx -s reload

# Hata loglarını kontrol edin
tail -f /var/log/nginx/error.log
```

**3. Caddy Yapılandırmasını Doğrulayın:**
```caddyfile
your-domain.com {
    reverse_proxy /app/* {
        to soketi:6001
        
        # WebSocket header'ları
        header_up Host {host}
        header_up Connection {>Connection}
        header_up Upgrade {>Upgrade}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-For {remote}
        header_up X-Forwarded-Proto {scheme}
    }
}
```

**4. Reverse Proxy'yi Test Edin:**
```bash
# Proxy üzerinden WebSocket bağlantısını test edin
wscat -c wss://your-domain.com/app/app-key

# Proxy loglarını kontrol edin
docker logs nginx -f
docker logs caddy -f
```

### SSL Sertifika Sorunları

**Belirti:**
```
Error: SSL certificate problem: unable to get local issuer certificate
```

**Olası Nedenler:**
1. Self-signed sertifika
2. Sertifika zinciri eksik
3. Sertifika süresi dolmuş
4. Yanlış sertifika yolu

**Çözümler:**

**1. Let's Encrypt Kullanın:**
```bash
# Caddy ile (otomatik)
# Caddy SSL'i otomatik olarak halleder

# Certbot ile (Nginx)
sudo certbot --nginx -d your-domain.com
```

**2. Sertifikayı Doğrulayın:**
```bash
# Sertifika süresini kontrol edin
openssl x509 -in /path/to/cert.pem -noout -dates

# Sertifika zincirini kontrol edin
openssl s_client -connect your-domain.com:443 -showcerts

# SSL yapılandırmasını test edin
curl -vI https://your-domain.com
```

**3. Sertifika Yolunu Yapılandırın:**
```bash
# soketi.rs ile SSL
SOKETI_SSL_CERT=/etc/ssl/certs/cert.pem
SOKETI_SSL_KEY=/etc/ssl/private/key.pem
SOKETI_SSL_PASSPHRASE=your-passphrase  # Şifreli ise
```

**4. Self-Signed Sertifikalara İzin Verin (Sadece Geliştirme):**
```typescript
// ⚠️ Sadece geliştirme için!
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  forceTLS: true,
  // Sadece Node.js
  // process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
});
```

## Yapılandırma Hataları

### Geçersiz Yapılandırma Dosyası

**Belirti:**
```
Error: Failed to parse configuration file
```

**Olası Nedenler:**
1. Geçersiz JSON syntax
2. Eksik gerekli alanlar
3. Yanlış veri tipleri

**Çözümler:**

**1. JSON'u Doğrulayın:**
```bash
# JSON syntax'ını doğrulayın
cat config.json | jq .

# Veya online validator kullanın
# https://jsonlint.com/
```

**2. Gerekli Alanları Kontrol Edin:**
```json
{
  "app_manager": {
    "driver": "array",
    "array": {
      "apps": [
        {
          "id": "app-id",           // Gerekli
          "key": "app-key",         // Gerekli
          "secret": "app-secret",   // Gerekli
          "enabled": true           // Gerekli
        }
      ]
    }
  }
}
```

**3. Bunun Yerine Environment Variable'ları Kullanın:**
```bash
# Config dosyasına daha basit alternatif
docker run \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  -p 6001:6001 \
  quay.io/soketi/soketi:latest-16-alpine
```

### App Kimlik Bilgileri Çalışmıyor

**Belirti:**
```
Error: Invalid app credentials
```

**Olası Nedenler:**
1. Client ve sunucu arasında uyuşmayan kimlik bilgileri
2. Kimlik bilgilerinde yazım hatası
3. Yanlış app ID kullanımı

**Çözümler:**

**1. Tüm Kimlik Bilgilerinin Eşleştiğini Doğrulayın:**
```bash
# Sunucu (soketi.rs)
SOKETI_DEFAULT_APP_ID=my-app
SOKETI_DEFAULT_APP_KEY=my-key
SOKETI_DEFAULT_APP_SECRET=my-secret

# Client
NEXT_PUBLIC_PUSHER_KEY=my-key  # SOKETI_DEFAULT_APP_KEY ile eşleşmeli

# Server SDK (auth endpoint)
PUSHER_APP_ID=my-app           # SOKETI_DEFAULT_APP_ID ile eşleşmeli
PUSHER_SECRET=my-secret        # SOKETI_DEFAULT_APP_SECRET ile eşleşmeli
```

**2. Kimlik Bilgilerini Test Edin:**
```bash
# Kimlik bilgileri ile bağlantıyı test edin
curl -X POST http://localhost:6001/apps/my-app/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-event",
    "channel": "test-channel",
    "data": "{\"message\": \"test\"}"
  }'
```

**3. Boşluk Karakterlerini Kontrol Edin:**
```bash
# Kimlik bilgilerinden boşlukları temizleyin
export SOKETI_DEFAULT_APP_KEY=$(echo "my-key" | tr -d '[:space:]')
```

## Debug Teknikleri

### Debug Loglamasını Etkinleştir

**Client Tarafı:**
```typescript
// Pusher debug loglamasını etkinleştirin
Pusher.logToConsole = true;

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  // ... diğer config
});

// Tarayıcı konsolunda detaylı loglar göreceksiniz:
// Pusher : State changed : connecting -> connected
// Pusher : Event sent : {"event":"pusher:subscribe","data":{"channel":"my-channel"}}
```

**Sunucu Tarafı:**
```bash
# Debug modunu etkinleştirin
SOKETI_DEBUG=true
SOKETI_LOG_LEVEL=debug

# Veya config.json'da
{
  "debug": true,
  "log_level": "debug"
}
```

### WebSocket Trafiğini İzle

**Tarayıcı DevTools Kullanarak:**
```
1. Chrome DevTools'u açın (F12)
2. Network sekmesine gidin
3. "WS" (WebSocket) ile filtreleyin
4. Bağlantıya tıklayarak frame'leri görün
5. Gönderilen/alınan mesajları görüntüleyin
```

**wscat Kullanarak:**
```bash
# wscat'i yükleyin
npm install -g wscat

# soketi.rs'ye bağlanın
wscat -c ws://localhost:6001/app/app-key

# Test mesajı gönderin
> {"event":"pusher:subscribe","data":{"channel":"test"}}

# Yanıtları görüntüleyin
< {"event":"pusher:connection_established","data":"..."}
```

**tcpdump Kullanarak:**
```bash
# WebSocket trafiğini yakalayın
sudo tcpdump -i any -A 'tcp port 6001'

# Analiz için dosyaya kaydedin
sudo tcpdump -i any -w websocket.pcap 'tcp port 6001'
```

### Event Akışını Test Et

**1. Sunucudan Client'a Test:**
```bash
# HTTP API üzerinden event tetikle
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-event",
    "channel": "test-channel",
    "data": "{\"message\": \"Merhaba\"}"
  }'

# Client event'i almalı
```

**2. Client'tan Sunucuya Test:**
```typescript
// Channel'a abone ol
const channel = pusher.subscribe('test-channel');

// Event'e bind et
channel.bind('test-event', (data) => {
  console.log('Alındı:', data);
});

// Aboneliğin başarılı olduğunu doğrula
channel.bind('pusher:subscription_succeeded', () => {
  console.log('✅ Başarıyla abone olundu');
});

channel.bind('pusher:subscription_error', (error) => {
  console.error('❌ Abonelik başarısız:', error);
});
```

**3. Client Event'leri Test Et:**
```typescript
// Private/presence channel'da
const channel = pusher.subscribe('private-test');

channel.bind('pusher:subscription_succeeded', () => {
  // Client event tetikle
  channel.trigger('client-test', { message: 'Merhaba' });
});

// Client event'i dinle
channel.bind('client-test', (data) => {
  console.log('Client event alındı:', data);
});
```

### Bağlantı Durumunu İncele

```typescript
// Tüm bağlantı durum değişikliklerini izle
pusher.connection.bind('state_change', (states) => {
  console.log(`Durum: ${states.previous} → ${states.current}`);
});

// Olası durumlar:
// - initialized
// - connecting
// - connected
// - unavailable
// - failed
// - disconnected

// Mevcut durumu kontrol et
console.log('Mevcut durum:', pusher.connection.state);

// Socket ID'yi al (bağlı olduğunda mevcut)
console.log('Socket ID:', pusher.connection.socket_id);
```

### Event Handler'ları İzle

```typescript
// Bir channel'daki tüm event'leri logla
const channel = pusher.subscribe('my-channel');

// Tüm event'lere bind et
channel.bind_global((eventName, data) => {
  console.log(`Event: ${eventName}`, data);
});

// Bind edilmiş tüm event'leri listele
console.log('Bind edilmiş event\'ler:', channel.callbacks._callbacks);
```

## İzleme ve Loglama

### Metrics Endpoint

soketi.rs Prometheus uyumlu bir metrics endpoint sağlar:

```bash
# Metrics'e erişin
curl http://localhost:9601/metrics

# İzlenecek önemli metrikler:
# - soketi_connections_total: Toplam aktif bağlantılar
# - soketi_messages_sent_total: Gönderilen toplam mesajlar
# - soketi_messages_received_total: Alınan toplam mesajlar
# - soketi_channels_total: Toplam aktif channel'lar
```

### Health Check Endpoint

```bash
# Sunucu sağlığını kontrol edin
curl http://localhost:6001/health

# Beklenen yanıt:
# {"status":"ok"}

# İzleme sistemlerinde kullanın
# Kubernetes liveness probe:
livenessProbe:
  httpGet:
    path: /health
    port: 6001
  initialDelaySeconds: 10
  periodSeconds: 30
```

### Yapılandırılmış Loglama

**Log Formatını Yapılandırın:**
```bash
# Daha kolay ayrıştırma için JSON loglama
SOKETI_LOG_FORMAT=json

# Dosyaya logla
SOKETI_LOG_FILE=/var/log/soketi/soketi.log
```

**Logları Ayrıştırın:**
```bash
# Hata loglarını filtrele
cat soketi.log | jq 'select(.level == "error")'

# Event'leri tipe göre say
cat soketi.log | jq -r '.event' | sort | uniq -c

# Gerçek zamanlı izle
tail -f soketi.log | jq .
```

### Harici İzleme

**İzleme Servisleri ile Entegrasyon:**

**1. Datadog:**
```yaml
# docker-compose.yml
services:
  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    labels:
      com.datadoghq.ad.logs: '[{"source": "soketi", "service": "websocket"}]'
```

**2. Prometheus:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['soketi:9601']
```

**3. Grafana Dashboard:**
```bash
# soketi.rs dashboard'unu import edin
# Prometheus'tan metrikleri kullanın
# Bağlantıları, mesajları, gecikmeyi görselleştirin
```

**4. Sentry (Hata Takibi):**
```typescript
// Next.js uygulamanızda
import * as Sentry from '@sentry/nextjs';

pusher.connection.bind('error', (err) => {
  Sentry.captureException(err);
});
```

## Yaygın Hata Mesajları

### "Connection refused"

**Anlamı:** soketi.rs sunucusuna bağlanılamıyor.

**Çözümler:**
- Sunucunun çalıştığını doğrulayın: `curl http://localhost:6001/health`
- Host ve port yapılandırmasını kontrol edin
- Firewall kurallarını doğrulayın

### "Invalid app key"

**Anlamı:** Client yanlış app key kullanıyor.

**Çözümler:**
- `NEXT_PUBLIC_PUSHER_KEY`'in `SOKETI_DEFAULT_APP_KEY` ile eşleştiğini doğrulayın
- Yazım hataları veya boşlukları kontrol edin
- Environment variable'ların yüklendiğinden emin olun

### "Subscription error: 403"

**Anlamı:** Private/presence channel için yetkilendirme başarısız.

**Çözümler:**
- Auth endpoint'in doğru yapılandırıldığını kontrol edin
- Auth endpoint'in doğru format döndürdüğünü doğrulayın
- Kullanıcının kimlik doğrulaması yapıldığından emin olun
- CORS yapılandırmasını kontrol edin

### "Max connections exceeded"

**Anlamı:** Çok fazla eşzamanlı bağlantı.

**Çözümler:**
- Config'de `max_connections`'ı artırın
- Birden fazla instance ile yatay ölçeklendirin
- Connection pooling uygulayın

### "Event payload too large"

**Anlamı:** Mesaj boyut limitini aşıyor.

**Çözümler:**
- Mesaj boyutunu azaltın
- Config'de `max_event_payload_in_kb`'yi artırın
- Büyük mesajları daha küçük parçalara bölün

### "Channel not found"

**Anlamı:** Var olmayan bir channel'da event tetiklemeye çalışılıyor.

**Çözümler:**
- Tetiklemeden önce channel'a abone olunduğundan emin olun
- Channel adı yazımını kontrol edin
- Channel'ın metrics'te var olduğunu doğrulayın

### "Rate limit exceeded"

**Anlamı:** Kısa sürede çok fazla istek.

**Çözümler:**
- Client tarafında rate limiting uygulayın
- Config'de rate limitleri artırın:
  ```json
  {
    "max_backend_events_per_second": 1000,
    "max_client_events_per_second": 100
  }
  ```
- Birden fazla event için batching kullanın

### "WebSocket upgrade failed"

**Anlamı:** Sunucu HTTP'yi WebSocket'e yükseltemiyor.

**Çözümler:**
- Reverse proxy WebSocket yapılandırmasını kontrol edin
- `Upgrade` ve `Connection` header'larını doğrulayın
- HTTP/1.1 kullanıldığından emin olun

## Yardım Alma

### Yardım İstemeden Önce

1. **Bu sorun giderme kılavuzunu kontrol edin** spesifik sorununuz için
2. **Debug loglamasını etkinleştirin** ve logları inceleyin
3. **Minimal yapılandırma ile test edin** sorunu izole etmek için
4. **Kimlik bilgilerini doğrulayın** doğru ve tutarlı olduklarından emin olun
5. **Sunucu durumunu kontrol edin** ve kaynak kullanımını inceleyin

### Eklenecek Bilgiler

Yardım isterken şunları sağlayın:

1. **soketi.rs versiyonu:**
   ```bash
   docker run quay.io/soketi/soketi:latest-16-alpine --version
   ```

2. **Yapılandırma:**
   ```bash
   # Paylaşmadan önce secret'ları temizleyin!
   cat config.json | jq 'del(.app_manager.array.apps[].secret)'
   ```

3. **Hata mesajları:**
   ```bash
   # Tam hata logları
   docker logs soketi 2>&1 | tail -50
   ```

4. **Client yapılandırması:**
   ```typescript
   // Pusher client kurulumunuzu paylaşın (secret'ları kaldırın!)
   const pusher = new Pusher('app-key', {
     wsHost: 'localhost',
     wsPort: 6001,
     // ...
   });
   ```

5. **Ortam:**
   - İşletim sistemi
   - Docker versiyonu (Docker kullanıyorsanız)
   - Node.js versiyonu (client için)
   - Tarayıcı versiyonu (tarayıcı sorunu ise)

### Topluluk Kaynakları

**GitHub Issues:**
- Mevcut issue'ları arayın: [soketi.rs Issues](https://github.com/soketi/soketi.rs/issues)
- Template ile yeni issue oluşturun
- Tüm ilgili bilgileri ekleyin

**Discord/Slack:**
- Topluluk sohbetine katılın
- Uygun kanallarda sorular sorun
- Benzer sorunları olan diğerlerine yardım edin

**Stack Overflow:**
- Soruları `soketi` ve `websocket` ile etiketleyin
- Önce mevcut soruları arayın
- Minimal tekrarlanabilir örnek sağlayın

### Profesyonel Destek

Production deployment'lar ve kurumsal destek için:
- soketi.rs ekibiyle iletişime geçin
- Yönetilen hosting çözümlerini değerlendirin
- WebSocket danışmanları kiralayın

## Hızlı Referans

### Temel Komutlar

```bash
# Sunucu sağlığını kontrol et
curl http://localhost:6001/health

# Metrics'i görüntüle
curl http://localhost:9601/metrics

# WebSocket bağlantısını test et
wscat -c ws://localhost:6001/app/app-key

# Test event tetikle
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{"name":"test","channel":"test","data":"{}"}'

# Docker loglarını görüntüle
docker logs soketi -f

# Container istatistiklerini kontrol et
docker stats soketi

# Container'ı yeniden başlat
docker restart soketi
```

### Debug Kontrol Listesi

- [ ] Sunucu çalışıyor (`curl http://localhost:6001/health`)
- [ ] Kimlik bilgileri client ve sunucu arasında eşleşiyor
- [ ] Port erişilebilir (firewall'u kontrol et)
- [ ] WebSocket upgrade header'ları yapılandırılmış (proxy kullanıyorsanız)
- [ ] SSL/TLS yapılandırması eşleşiyor (forceTLS, encrypted)
- [ ] Auth endpoint doğru yapılandırılmış (private/presence için)
- [ ] CORS tarayıcı client'ları için yapılandırılmış
- [ ] Debug loglama etkin
- [ ] Metrics beklenen değerleri gösteriyor
- [ ] Sunucu loglarında hata yok

### Performans Kontrol Listesi

- [ ] Yatay ölçeklendirme için Redis adapter kullanılıyor
- [ ] Birden fazla soketi.rs instance'ı deploy edilmiş
- [ ] Load balancer uygun timeout'larla yapılandırılmış
- [ ] Connection pooling uygulanmış
- [ ] Event payload'ları optimize edilmiş (< 100KB)
- [ ] Rate limitler uygun şekilde yapılandırılmış
- [ ] Sıkıştırma etkin
- [ ] İzleme ve uyarı kurulmuş
- [ ] Kaynak limitleri yapılandırılmış
- [ ] Coğrafi dağıtım değerlendirilmiş

## İlgili Kaynaklar

- [Başlangıç Kılavuzu](baslangic.md)
- [Yapılandırma Referansı](yapilandirma.md)
- [API Referansı](api-referans.md)
- [Deployment Kılavuzu](deployment/reverse-proxy.md)
- [Vercel Deployment](deployment/vercel.md)
- [Netlify Deployment](deployment/netlify.md)
- [Örnekler](ornekler/temel-chat.md)

---

**Daha fazla yardıma mı ihtiyacınız var?** [soketi.rs dokümantasyonunu](https://docs.soketi.app) kontrol edin veya GitHub'da [bir issue açın](https://github.com/soketi/soketi.rs/issues).
