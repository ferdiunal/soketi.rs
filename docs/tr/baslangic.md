# Soketi ile Başlangıç

> Rust ile yazılmış yüksek performanslı Pusher protokol uyumlu WebSocket sunucusu Soketi'yi hızlıca çalıştırmak için kılavuz.

## İçindekiler

- [Giriş](#giriş)
- [Hızlı Başlangıç](#hızlı-başlangıç)
- [Kurulum](#kurulum)
- [Temel Yapılandırma](#temel-yapılandırma)
- [İlk Bağlantınız](#ilk-bağlantınız)
- [WebSocket Bağlantı Örnekleri](#websocket-bağlantı-örnekleri)
- [Sonraki Adımlar](#sonraki-adımlar)

## Giriş

Soketi, Pusher protokolünü uygulayan hızlı ve ölçeklenebilir bir WebSocket sunucusudur. Genel kanallar, özel kanallar ve presence kanallarını destekleyerek gerçek zamanlı uygulamalar oluşturmanıza olanak tanır.

## Hızlı Başlangıç

Soketi ile başlamanın en hızlı yolu Docker kullanmaktır:

```bash
docker run -p 6001:6001 -p 9601:9601 \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  quay.io/soketi/soketi:latest-16-alpine
```

Soketi sunucunuz artık `ws://localhost:6001` adresinde çalışıyor!

## Kurulum

### Docker Kullanarak (Önerilen)

Docker, Soketi'yi çalıştırmanın en kolay yoludur:

```bash
docker pull quay.io/soketi/soketi:latest-16-alpine
```

### Cargo Kullanarak

Rust yüklüyse, kaynak koddan derleyebilirsiniz:

```bash
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
./target/release/soketi
```

### Önceden Derlenmiş Binary Kullanarak

Önceden derlenmiş binary dosyalarını [sürümler sayfasından](https://github.com/soketi/soketi.rs/releases) indirebilirsiniz.

## Temel Yapılandırma

Bir `config.json` dosyası oluşturun:

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

Soketi'yi yapılandırmanızla başlatın:

```bash
soketi --config config.json
```

## İlk Bağlantınız

### JavaScript/TypeScript Kullanarak

Pusher JavaScript kütüphanesini yükleyin:

```bash
npm install pusher-js
```

Soketi sunucunuza bağlanın:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
});

const channel = pusher.subscribe('my-channel');

channel.bind('my-event', (data: any) => {
  console.log('Alındı:', data);
});

console.log('Soketi\'ye bağlandı!');
```

### Bağlantınızı Test Etme

HTTP API kullanarak olayları tetikleyebilirsiniz:

```bash
curl -X POST http://localhost:6001/apps/app-id/events \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-event",
    "channel": "my-channel",
    "data": "{\"message\": \"Soketi\'den merhaba!\"}"
  }'
```

## WebSocket Bağlantı Örnekleri

### Temel Genel Kanal

Genel kanallar tüm istemcilere açıktır ve kimlik doğrulama gerektirmez:

```typescript
import Pusher from 'pusher-js';

// Pusher istemcisini başlat
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1', // İsteğe bağlı, uyumluluk için
});

// Genel bir kanala abone ol
const channel = pusher.subscribe('public-channel');

// Olaylara bağlan
channel.bind('message', (data: any) => {
  console.log('Yeni mesaj:', data);
});

// Bağlantı durumu olayları
pusher.connection.bind('connected', () => {
  console.log('Soketi\'ye bağlandı');
});

pusher.connection.bind('disconnected', () => {
  console.log('Soketi\'den bağlantı kesildi');
});

pusher.connection.bind('error', (err: any) => {
  console.error('Bağlantı hatası:', err);
});
```

### Kimlik Doğrulamalı Özel Kanal

Özel kanallar, istemcilerin abone olabilmesi için kimlik doğrulama gerektirir:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/pusher/auth', // Kimlik doğrulama endpoint'iniz
  auth: {
    headers: {
      'Authorization': 'Bearer your-token-here'
    }
  }
});

// Özel bir kanala abone ol ('private-' ile başlamalı)
const privateChannel = pusher.subscribe('private-user-123');

privateChannel.bind('pusher:subscription_succeeded', () => {
  console.log('Özel kanala başarıyla abone olundu');
});

privateChannel.bind('notification', (data: any) => {
  console.log('Özel bildirim:', data);
});

privateChannel.bind('pusher:subscription_error', (status: any) => {
  console.error('Abonelik başarısız:', status);
});
```

### Kullanıcı Bilgili Presence Kanalı

Presence kanalları hangi kullanıcıların abone olduğunu takip eder ve üye bilgisi sağlar:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/pusher/auth',
  auth: {
    headers: {
      'Authorization': 'Bearer your-token-here'
    }
  }
});

// Presence kanalına abone ol ('presence-' ile başlamalı)
const presenceChannel = pusher.subscribe('presence-chat-room');

// Abonelik başarılı olduğunda, tüm üyeleri al
presenceChannel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Mevcut üyeler:', members.count);
  members.each((member: any) => {
    console.log('Üye:', member.id, member.info);
  });
});

// Yeni bir üye katıldığında
presenceChannel.bind('pusher:member_added', (member: any) => {
  console.log('Üye katıldı:', member.id, member.info);
});

// Bir üye ayrıldığında
presenceChannel.bind('pusher:member_removed', (member: any) => {
  console.log('Üye ayrıldı:', member.id);
});

// Presence kanalındaki normal olaylar
presenceChannel.bind('chat-message', (data: any) => {
  console.log('Sohbet mesajı:', data);
});
```

### İstemci Olayları (Eşler Arası)

İstemci olayları, istemcilerin aynı kanaldaki diğer istemcilere doğrudan mesaj göndermesine olanak tanır:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/pusher/auth',
});

// İstemci olayları yalnızca özel ve presence kanallarında çalışır
const channel = pusher.subscribe('private-chat');

channel.bind('pusher:subscription_succeeded', () => {
  // Bir istemci olayı tetikle ('client-' ile başlamalı)
  channel.trigger('client-typing', {
    user: 'Ahmet',
    isTyping: true
  });
});

// Diğer kullanıcılardan gelen istemci olaylarını dinle
channel.bind('client-typing', (data: any) => {
  console.log(`${data.user} yazıyor:`, data.isTyping);
});
```

### Yeniden Bağlanma İşleme

Soketi otomatik olarak yeniden bağlanmayı yönetir, ancak davranışı özelleştirebilirsiniz:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  // Yeniden bağlanma ayarları
  activityTimeout: 30000, // 30 saniye
  pongTimeout: 10000, // 10 saniye
});

const channel = pusher.subscribe('my-channel');

// Bağlantı durumunu takip et
pusher.connection.bind('state_change', (states: any) => {
  console.log(`Bağlantı durumu ${states.previous} durumundan ${states.current} durumuna değişti`);
});

// Yeniden bağlanmayı yönet
pusher.connection.bind('connected', () => {
  console.log('Bağlandı! Socket ID:', pusher.connection.socket_id);
});

pusher.connection.bind('connecting', () => {
  console.log('Soketi\'ye bağlanıyor...');
});

pusher.connection.bind('unavailable', () => {
  console.log('Bağlantı kullanılamıyor, yeniden denenecek...');
});

pusher.connection.bind('failed', () => {
  console.error('Bağlantı kalıcı olarak başarısız oldu');
});
```

### Tam Örnek: Gerçek Zamanlı Sohbet

İşte birden fazla konsepti birleştiren tam bir örnek:

```typescript
import Pusher from 'pusher-js';

// Pusher'ı başlat
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
  authEndpoint: '/api/pusher/auth',
  auth: {
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    }
  }
});

// Presence kanalına abone ol
const chatChannel = pusher.subscribe('presence-chat-room');

// Abonelik başarısını yönet
chatChannel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log(`Bağlandı! ${members.count} kullanıcı çevrimiçi`);
  updateUserList(members);
});

// Yeni üyeleri yönet
chatChannel.bind('pusher:member_added', (member: any) => {
  console.log(`${member.info.name} katıldı`);
  addUserToList(member);
});

// Ayrılan üyeleri yönet
chatChannel.bind('pusher:member_removed', (member: any) => {
  console.log(`${member.info.name} ayrıldı`);
  removeUserFromList(member);
});

// Sohbet mesajlarını yönet
chatChannel.bind('chat-message', (data: any) => {
  displayMessage(data);
});

// Yazma göstergelerini yönet
chatChannel.bind('client-typing', (data: any) => {
  showTypingIndicator(data.userId, data.isTyping);
});

// Mesaj gönder
function sendMessage(message: string) {
  fetch('/api/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${localStorage.getItem('token')}`
    },
    body: JSON.stringify({
      channel: 'presence-chat-room',
      event: 'chat-message',
      data: {
        message,
        userId: getCurrentUserId(),
        userName: getCurrentUserName(),
        timestamp: new Date().toISOString()
      }
    })
  });
}

// Yazma göstergesi gönder
function sendTypingIndicator(isTyping: boolean) {
  chatChannel.trigger('client-typing', {
    userId: getCurrentUserId(),
    isTyping
  });
}

// Bağlantı izleme
pusher.connection.bind('state_change', (states: any) => {
  updateConnectionStatus(states.current);
});

// Yardımcı fonksiyonlar (UI'nıza göre uygulayın)
function updateUserList(members: any) { /* ... */ }
function addUserToList(member: any) { /* ... */ }
function removeUserFromList(member: any) { /* ... */ }
function displayMessage(data: any) { /* ... */ }
function showTypingIndicator(userId: string, isTyping: boolean) { /* ... */ }
function updateConnectionStatus(status: string) { /* ... */ }
function getCurrentUserId(): string { return 'user-123'; }
function getCurrentUserName(): string { return 'Ahmet Yılmaz'; }
```

## Sonraki Adımlar

Artık Soketi çalışıyor, şu konuları keşfedin:

- **[Kurulum Kılavuzu](kurulum.md)** - Detaylı kurulum seçenekleri
- **[Yapılandırma Referansı](yapilandirma.md)** - Tüm yapılandırma seçenekleri
- **[API Referansı](api-referans.md)** - WebSocket ve HTTP API dokümantasyonu
- **[Deployment Kılavuzu](deployment/reverse-proxy.md)** - Reverse proxy ile production deployment
- **[Örnekler](ornekler/temel-chat.md)** - Kod örnekleri ve eğitimler

## İlgili Kaynaklar

- [Resmi Pusher Dokümantasyonu](https://pusher.com/docs)
- [Soketi GitHub Deposu](https://github.com/soketi/soketi.rs)
- [Sorun Giderme Kılavuzu](sorun-giderme.md)
