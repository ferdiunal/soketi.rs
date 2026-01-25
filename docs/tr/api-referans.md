# API Referansı

> Soketi WebSocket ve HTTP API'leri için eksiksiz API dokümantasyonu.

## İçindekiler

- [WebSocket API](#websocket-api)
- [HTTP API](#http-api)
- [Kimlik Doğrulama](#kimlik-doğrulama)
- [Kanal Tipleri](#kanal-tipleri)
- [Olaylar](#olaylar)
- [Hata Kodları](#hata-kodları)

## WebSocket API

Soketi, Pusher WebSocket protokolünü uygular. İstemciler WebSocket üzerinden bağlanır ve JSON mesajları kullanarak iletişim kurar.

### Bağlantı

Pusher istemci kütüphanesini kullanarak Soketi'ye bağlanın:

```typescript
import Pusher from 'pusher-js';

const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  encrypted: false,
  enabledTransports: ['ws', 'wss'],
});
```

### Bağlantı Olayları

| Olay | Açıklama |
|------|----------|
| `pusher:connection_established` | Bağlantı kurulduğunda tetiklenir |
| `pusher:error` | Bir hata oluştuğunda tetiklenir |

Örnek:

```typescript
pusher.connection.bind('connected', () => {
  console.log('Soketi\'ye bağlandı');
});

pusher.connection.bind('error', (err: any) => {
  console.error('Bağlantı hatası:', err);
});
```

### Kanallara Abone Olma

#### Genel Kanallar

```typescript
const channel = pusher.subscribe('my-channel');

channel.bind('my-event', (data: any) => {
  console.log('Alındı:', data);
});
```

#### Özel Kanallar

Özel kanallar kimlik doğrulama gerektirir:

```typescript
const privateChannel = pusher.subscribe('private-my-channel');

privateChannel.bind('my-event', (data: any) => {
  console.log('Alındı:', data);
});
```

#### Presence Kanalları

Presence kanalları çevrimiçi kullanıcıları takip eder:

```typescript
const presenceChannel = pusher.subscribe('presence-my-channel');

presenceChannel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Çevrimiçi kullanıcılar:', members.count);
  members.each((member: any) => {
    console.log('Kullanıcı:', member.id, member.info);
  });
});

presenceChannel.bind('pusher:member_added', (member: any) => {
  console.log('Kullanıcı katıldı:', member.id);
});

presenceChannel.bind('pusher:member_removed', (member: any) => {
  console.log('Kullanıcı ayrıldı:', member.id);
});
```

### İstemci Olayları

İstemciden diğer istemcilere olay gönderin (uygulama yapılandırmasında etkinleştirilmiş olmalıdır):

```typescript
channel.trigger('client-my-event', {
  message: 'İstemciden merhaba!'
});
```

**Gereksinimler**:
- Kanal özel veya presence olmalıdır
- Uygulama yapılandırmasında `enable_client_messages` true olmalıdır
- Olay adı `client-` ile başlamalıdır

### Abonelikten Çıkma

```typescript
pusher.unsubscribe('my-channel');
```

### Bağlantıyı Kesme

```typescript
pusher.disconnect();
```

## HTTP API

HTTP API, backend'inizden olayları tetiklemenize, kanal bilgilerini sorgulamanıza ve bağlantıları yönetmenize olanak tanır.

### Temel URL

```
http://localhost:6001/apps/{app_id}
```

### Kimlik Doğrulama

Tüm HTTP API istekleri HMAC SHA256 imzası kullanılarak doğrulanmalıdır.

#### İstek İmzalama

1. Auth string oluştur: `POST\n/apps/{app_id}/events\nauth_key={key}&auth_timestamp={timestamp}&auth_version=1.0&body_md5={md5}`
2. Uygulama secret'ı kullanarak HMAC SHA256 ile imzala
3. İmzayı sorgu parametrelerine ekle

Node.js kullanarak örnek:

```typescript
import crypto from 'crypto';

function signRequest(
  method: string,
  path: string,
  params: Record<string, string>,
  body: string,
  secret: string
): string {
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `${method}\n${path}\n${sortedParams}&body_md5=${bodyMd5}`;
  
  return crypto
    .createHmac('sha256', secret)
    .update(authString)
    .digest('hex');
}
```

### Olay Tetikleme

Backend'inizden kanallara olay gönderin.

**Endpoint**: `POST /apps/{app_id}/events`

**İstek Parametreleri**:

| Parametre | Tip | Açıklama | Gerekli |
|-----------|-----|----------|---------|
| `name` | string | Olay adı | Evet |
| `data` | string veya object | Olay verisi (JSON string veya object) | Evet |
| `channel` | string | Tek kanal adı | Hayır* |
| `channels` | array | Kanal adları dizisi | Hayır* |
| `socket_id` | string | Olayı almaktan hariç tutulacak Socket ID | Hayır |

*`channel` veya `channels`'dan biri sağlanmalıdır

**İstek Gövdesi Örnekleri**:

**Tek Kanal**:

```json
{
  "name": "my-event",
  "channel": "my-channel",
  "data": "{\"message\": \"Merhaba Dünya!\"}"
}
```

**Çoklu Kanallar**:

```json
{
  "name": "my-event",
  "channels": ["channel-1", "channel-2", "channel-3"],
  "data": "{\"message\": \"Merhaba Dünya!\"}"
}
```

**Socket ID Hariç Tutma ile**:

```json
{
  "name": "message-sent",
  "channel": "chat-room",
  "data": "{\"user\": \"Ahmet\", \"message\": \"Herkese merhaba!\"}",
  "socket_id": "123.456"
}
```

**Yanıt**:

```json
{}
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Olay başarıyla tetiklendi |
| `400` | Geçersiz istek (doğrulama hatası) |
| `401` | Kimlik doğrulama başarısız |
| `403` | Yasak (hız limiti aşıldı) |
| `413` | Yük çok büyük |

**Pusher Server SDK Kullanarak Örnek (Önerilen)**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Tek kanala tetikleme
await pusher.trigger('my-channel', 'my-event', {
  message: 'Merhaba Dünya!'
});

// Çoklu kanallara tetikleme
await pusher.trigger(['channel-1', 'channel-2'], 'my-event', {
  message: 'Merhaba Dünya!'
});

// Socket ID hariç tutma ile tetikleme
await pusher.trigger('chat-room', 'message-sent', {
  user: 'Ahmet',
  message: 'Herkese merhaba!'
}, {
  socket_id: '123.456'
});
```

**cURL Kullanarak Örnek**:

```bash
# Kimlik doğrulama parametrelerini oluştur
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
TIMESTAMP=$(date +%s)
BODY='{"name":"my-event","channel":"my-channel","data":"{\"message\":\"Merhaba!\"}"}'
BODY_MD5=$(echo -n "$BODY" | md5sum | cut -d' ' -f1)

# Auth string ve imza oluştur
AUTH_STRING="POST
/apps/$APP_ID/events
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&body_md5=$BODY_MD5"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# İstek yap
curl -X POST "http://localhost:6001/apps/$APP_ID/events?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE&body_md5=$BODY_MD5" \
  -H "Content-Type: application/json" \
  -d "$BODY"
```

**Axios Kullanarak Örnek (Node.js)**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function triggerEvent(
  appId: string,
  appKey: string,
  appSecret: string,
  channel: string,
  event: string,
  data: any
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const body = JSON.stringify({
    name: event,
    channel: channel,
    data: JSON.stringify(data),
  });
  
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
    body_md5: bodyMd5,
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `POST\n/apps/${appId}/events\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001/apps/${appId}/events?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.post(url, body, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  return response.data;
}

// Kullanım
await triggerEvent(
  'app-id',
  'app-key',
  'app-secret',
  'my-channel',
  'my-event',
  { message: 'Merhaba Dünya!' }
);
```

**Doğrulama Limitleri**:

| Limit | Varsayılan | Açıklama |
|-------|------------|----------|
| Olay adı uzunluğu | 200 karakter | Olay adının maksimum uzunluğu |
| Kanal adı uzunluğu | 200 karakter | Kanal adının maksimum uzunluğu |
| Yük boyutu | 10 KB | Olay verisinin maksimum boyutu |
| İstek başına kanal | 100 | Bir istekteki maksimum kanal sayısı |

Bu limitler uygulama yapılandırmasında uygulama başına yapılandırılabilir.


### Toplu Olaylar

Daha iyi performans için tek bir istekte birden fazla olay tetikleyin.

**Endpoint**: `POST /apps/{app_id}/batch_events`

**İstek Gövdesi**:

```json
{
  "batch": [
    {
      "name": "event-1",
      "channel": "channel-1",
      "data": "{\"message\": \"Olay 1\"}"
    },
    {
      "name": "event-2",
      "channels": ["channel-2", "channel-3"],
      "data": "{\"message\": \"Olay 2\"}"
    },
    {
      "name": "event-3",
      "channel": "channel-4",
      "data": "{\"user\": \"Ahmet\", \"action\": \"katıldı\"}",
      "socket_id": "123.456"
    }
  ]
}
```

**Yanıt**:

```json
{}
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Tüm olaylar başarıyla tetiklendi |
| `400` | Geçersiz istek (doğrulama hatası) |
| `401` | Kimlik doğrulama başarısız |
| `403` | Yasak (hız limiti aşıldı) |

**Pusher Server SDK Kullanarak Örnek**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Toplu olayları tetikle
await pusher.triggerBatch([
  {
    channel: 'channel-1',
    name: 'event-1',
    data: { message: 'Olay 1' },
  },
  {
    channel: 'channel-2',
    name: 'event-2',
    data: { message: 'Olay 2' },
  },
  {
    channel: 'channel-3',
    name: 'event-3',
    data: { message: 'Olay 3' },
  },
]);
```

**Axios Kullanarak Örnek**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function triggerBatchEvents(
  appId: string,
  appKey: string,
  appSecret: string,
  events: Array<{
    channel: string;
    name: string;
    data: any;
    socket_id?: string;
  }>
) {
  const timestamp = Math.floor(Date.now() / 1000);
  
  const body = JSON.stringify({
    batch: events.map(event => ({
      name: event.name,
      channel: event.channel,
      data: JSON.stringify(event.data),
      socket_id: event.socket_id,
    })),
  });
  
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
    body_md5: bodyMd5,
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `POST\n/apps/${appId}/batch_events\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001/apps/${appId}/batch_events?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.post(url, body, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  return response.data;
}

// Kullanım
await triggerBatchEvents(
  'app-id',
  'app-key',
  'app-secret',
  [
    { channel: 'channel-1', name: 'event-1', data: { message: 'Olay 1' } },
    { channel: 'channel-2', name: 'event-2', data: { message: 'Olay 2' } },
    { channel: 'channel-3', name: 'event-3', data: { message: 'Olay 3' } },
  ]
);
```

**Toplu İşlem Boyut Limitleri**:

| Limit | Varsayılan | Açıklama |
|-------|------------|----------|
| Toplu işlem boyutu | 10 olay | Toplu istek başına maksimum olay sayısı |

**Toplu Olayların Avantajları**:

- **Azaltılmış Ağ Yükü**: Birden fazla yerine tek HTTP isteği
- **Daha İyi Performans**: Birden fazla olay için daha düşük gecikme
- **Atomik İşlemler**: Tüm olaylar birlikte işlenir
- **Hız Limiti Verimliliği**: Hız sınırlaması için bir istek olarak sayılır

### Kanalları Getir

Abonelik bilgileriyle birlikte tüm aktif kanalları listele.

**Endpoint**: `GET /apps/{app_id}/channels`

**Sorgu Parametreleri**:

| Parametre | Tip | Açıklama | Gerekli |
|-----------|-----|----------|---------|
| `filter_by_prefix` | string | Kanalları ön eke göre filtrele | Hayır |
| `info` | string | Dahil edilecek özelliklerin virgülle ayrılmış listesi | Hayır |

**Mevcut Info Özellikleri**:

- `user_count` - Benzersiz kullanıcı sayısı (sadece presence kanalları)
- `subscription_count` - Aktif abonelik sayısı

**Yanıt**:

```json
{
  "channels": {
    "channel-1": {
      "subscription_count": 3,
      "occupied": true
    },
    "channel-2": {
      "subscription_count": 1,
      "occupied": true
    },
    "presence-room": {
      "subscription_count": 5,
      "occupied": true,
      "user_count": 5
    }
  }
}
```

**Örnek - Tüm Kanalları Listele**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Tüm kanalları getir
const result = await pusher.get({ path: '/channels' });
console.log(result.channels);
```

**Örnek - Ön Eke Göre Filtrele**:

```typescript
// Sadece presence kanallarını getir
const result = await pusher.get({
  path: '/channels',
  params: { filter_by_prefix: 'presence-' }
});
console.log(result.channels);
```

**Örnek - Kullanıcı Sayısını Dahil Et**:

```typescript
// Kullanıcı sayısı bilgisiyle kanalları getir
const result = await pusher.get({
  path: '/channels',
  params: { info: 'user_count' }
});
console.log(result.channels);
```

**Axios Kullanarak Örnek**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function getChannels(
  appId: string,
  appKey: string,
  appSecret: string,
  filterPrefix?: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/channels`;
  
  const params: Record<string, string> = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  if (filterPrefix) {
    params.filter_by_prefix = filterPrefix;
  }
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `GET\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.get(url);
  return response.data;
}

// Kullanım
const channels = await getChannels('app-id', 'app-key', 'app-secret');
console.log(channels);

// Filtre ile
const presenceChannels = await getChannels(
  'app-id',
  'app-key',
  'app-secret',
  'presence-'
);
console.log(presenceChannels);
```

**cURL Kullanarak Örnek**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
TIMESTAMP=$(date +%s)

# Auth string oluştur
AUTH_STRING="GET
/apps/$APP_ID/channels
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# İstek yap
curl "http://localhost:6001/apps/$APP_ID/channels?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE"

# Filtre ile
curl "http://localhost:6001/apps/$APP_ID/channels?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE&filter_by_prefix=presence-"
```

**Notlar**:

- Sadece en az bir aktif aboneliği olan kanallar döndürülür
- Boş kanallar otomatik olarak yanıttan hariç tutulur
- Döndürülen kanallar için `occupied` alanı her zaman `true`'dur

### Kanal Bilgisi Getir

Belirli bir kanal hakkında detaylı bilgi alın.

**Endpoint**: `GET /apps/{app_id}/channels/{channel_name}`

**Sorgu Parametreleri**:

| Parametre | Tip | Açıklama | Gerekli |
|-----------|-----|----------|---------|
| `info` | string | Dahil edilecek özelliklerin virgülle ayrılmış listesi | Hayır |

**Mevcut Info Özellikleri**:

- `user_count` - Benzersiz kullanıcı sayısı (sadece presence kanalları)
- `subscription_count` - Aktif abonelik sayısı

**Normal Kanal için Yanıt**:

```json
{
  "occupied": true,
  "subscription_count": 5
}
```

**Presence Kanalı için Yanıt**:

```json
{
  "occupied": true,
  "subscription_count": 5,
  "user_count": 5
}
```

**Pusher SDK Kullanarak Örnek**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Kanal bilgisi getir
const result = await pusher.get({
  path: '/channels/my-channel',
  params: { info: 'subscription_count' }
});
console.log(result);

// Kullanıcı sayısıyla presence kanal bilgisi getir
const presenceResult = await pusher.get({
  path: '/channels/presence-room',
  params: { info: 'user_count,subscription_count' }
});
console.log(presenceResult);
```

**Axios Kullanarak Örnek**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function getChannelInfo(
  appId: string,
  appKey: string,
  appSecret: string,
  channelName: string,
  info?: string[]
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/channels/${encodeURIComponent(channelName)}`;
  
  const params: Record<string, string> = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  if (info && info.length > 0) {
    params.info = info.join(',');
  }
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `GET\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.get(url);
  return response.data;
}

// Kullanım
const channelInfo = await getChannelInfo(
  'app-id',
  'app-key',
  'app-secret',
  'my-channel',
  ['subscription_count']
);
console.log(channelInfo);

// Presence kanalı
const presenceInfo = await getChannelInfo(
  'app-id',
  'app-key',
  'app-secret',
  'presence-room',
  ['user_count', 'subscription_count']
);
console.log(presenceInfo);
```

**cURL Kullanarak Örnek**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
CHANNEL_NAME="my-channel"
TIMESTAMP=$(date +%s)

# Kanal adını URL encode et
ENCODED_CHANNEL=$(echo -n "$CHANNEL_NAME" | jq -sRr @uri)

# Auth string oluştur
AUTH_STRING="GET
/apps/$APP_ID/channels/$ENCODED_CHANNEL
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# İstek yap
curl "http://localhost:6001/apps/$APP_ID/channels/$ENCODED_CHANNEL?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE"
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Kanal bilgisi başarıyla alındı |
| `401` | Kimlik doğrulama başarısız |
| `404` | Kanal bulunamadı veya abonesi yok |

### Kanal Kullanıcılarını Getir

Kullanıcı bilgileriyle birlikte presence kanalındaki kullanıcıların listesini alın.

**Endpoint**: `GET /apps/{app_id}/channels/{channel_name}/users`

**Gereksinimler**:

- Kanal bir presence kanalı olmalıdır (adı `presence-` ile başlamalıdır)
- Kanalın en az bir aktif abonesi olmalıdır

**Yanıt**:

```json
{
  "users": [
    {
      "id": "user-1"
    },
    {
      "id": "user-2",
      "user_info": {
        "name": "Ahmet Yılmaz",
        "email": "ahmet@example.com"
      }
    },
    {
      "id": "user-3",
      "user_info": {
        "name": "Ayşe Demir",
        "email": "ayse@example.com",
        "avatar": "https://example.com/avatar.jpg"
      }
    }
  ]
}
```

**Pusher SDK Kullanarak Örnek**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Presence kanalındaki kullanıcıları getir
const result = await pusher.get({
  path: '/channels/presence-room/users'
});

console.log('Kullanıcılar:', result.users);
result.users.forEach((user: any) => {
  console.log(`Kullanıcı ${user.id}:`, user.user_info);
});
```

**Axios Kullanarak Örnek**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function getChannelUsers(
  appId: string,
  appKey: string,
  appSecret: string,
  channelName: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/channels/${encodeURIComponent(channelName)}/users`;
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `GET\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.get(url);
  return response.data;
}

// Kullanım
const users = await getChannelUsers(
  'app-id',
  'app-key',
  'app-secret',
  'presence-room'
);

console.log('Çevrimiçi kullanıcılar:', users.users.length);
users.users.forEach((user: any) => {
  console.log(`- ${user.id}: ${user.user_info?.name || 'Anonim'}`);
});
```

**cURL Kullanarak Örnek**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
CHANNEL_NAME="presence-room"
TIMESTAMP=$(date +%s)

# Kanal adını URL encode et
ENCODED_CHANNEL=$(echo -n "$CHANNEL_NAME" | jq -sRr @uri)

# Auth string oluştur
AUTH_STRING="GET
/apps/$APP_ID/channels/$ENCODED_CHANNEL/users
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# İstek yap
curl "http://localhost:6001/apps/$APP_ID/channels/$ENCODED_CHANNEL/users?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE"
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Kullanıcılar başarıyla alındı |
| `400` | Kanal bir presence kanalı değil |
| `401` | Kimlik doğrulama başarısız |
| `404` | Kanal bulunamadı veya abonesi yok |

**Notlar**:

- Sadece presence kanalları için çalışır (`presence-` ile başlayan kanallar)
- Şu anda kanala abone olan tüm benzersiz kullanıcıları döndürür
- `user_info` alanı kimlik doğrulama sırasında sağlanan verileri içerir
- Kimlik doğrulama sırasında `user_info` sağlanmadıysa, sadece `id` alanı döndürülür


### Kullanıcı Bağlantılarını Sonlandır

Belirli bir kullanıcının tüm kanallardaki tüm WebSocket bağlantılarını sonlandırın.

**Endpoint**: `POST /apps/{app_id}/users/{user_id}/terminate_connections`

**Kullanım Senaryoları**:

- Bir kullanıcıyı tüm cihazlardan zorla çıkış yaptır
- Kullanıcı izinleri değiştiğinde erişimi iptal et
- Güvenlik: Tehlikeye girmiş kullanıcı oturumlarını sonlandır
- İdari işlemler

**Yanıt**:

```json
{}
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Bağlantılar başarıyla sonlandırıldı |
| `401` | Kimlik doğrulama başarısız |
| `500` | Sunucu hatası |

**Davranış**:

1. Sunucu, kullanıcı ID'siyle ilişkili tüm bağlantıları tanımlar
2. Her bağlantıya `4009` koduyla `pusher:error` olayı gönderir
3. O kullanıcının tüm WebSocket bağlantılarını kapatır
4. Kullanıcı tüm presence kanallarından çıkarılır

**Pusher SDK Kullanarak Örnek**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Bir kullanıcının tüm bağlantılarını sonlandır
await pusher.post({
  path: '/users/user-123/terminate_connections'
});

console.log('Kullanıcı bağlantıları sonlandırıldı');
```

**Axios Kullanarak Örnek**:

```typescript
import axios from 'axios';
import crypto from 'crypto';

async function terminateUserConnections(
  appId: string,
  appKey: string,
  appSecret: string,
  userId: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  const path = `/apps/${appId}/users/${encodeURIComponent(userId)}/terminate_connections`;
  const body = '';
  
  const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
  
  const params = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
    body_md5: bodyMd5,
  };
  
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  const authString = `POST\n${path}\n${sortedParams}`;
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  const url = `http://localhost:6001${path}?${sortedParams}&auth_signature=${signature}`;
  
  const response = await axios.post(url, body, {
    headers: { 'Content-Type': 'application/json' },
  });
  
  return response.data;
}

// Kullanım
await terminateUserConnections('app-id', 'app-key', 'app-secret', 'user-123');
console.log('Kullanıcı bağlantıları sonlandırıldı');
```

**cURL Kullanarak Örnek**:

```bash
APP_ID="app-id"
APP_KEY="app-key"
APP_SECRET="app-secret"
USER_ID="user-123"
TIMESTAMP=$(date +%s)
BODY=""
BODY_MD5=$(echo -n "$BODY" | md5sum | cut -d' ' -f1)

# Kullanıcı ID'sini URL encode et
ENCODED_USER=$(echo -n "$USER_ID" | jq -sRr @uri)

# Auth string oluştur
AUTH_STRING="POST
/apps/$APP_ID/users/$ENCODED_USER/terminate_connections
auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&body_md5=$BODY_MD5"

SIGNATURE=$(echo -n "$AUTH_STRING" | openssl dgst -sha256 -hmac "$APP_SECRET" | cut -d' ' -f2)

# İstek yap
curl -X POST "http://localhost:6001/apps/$APP_ID/users/$ENCODED_USER/terminate_connections?auth_key=$APP_KEY&auth_timestamp=$TIMESTAMP&auth_version=1.0&auth_signature=$SIGNATURE&body_md5=$BODY_MD5" \
  -H "Content-Type: application/json" \
  -d "$BODY"
```

**İstemci Tarafı İşleme**:

Bir kullanıcının bağlantıları sonlandırıldığında, bir hata olayı alacaklardır:

```typescript
pusher.connection.bind('error', (err: any) => {
  if (err.error && err.error.code === 4009) {
    console.log('Bağlantı sunucu tarafından sonlandırıldı');
    // Zorla çıkışı işle
    // Giriş sayfasına yönlendir veya mesaj göster
    window.location.href = '/login?reason=session_terminated';
  }
});
```

**Tam Örnek - Zorla Çıkış Sistemi**:

```typescript
// Backend: Kullanıcı bağlantılarını sonlandır
import Pusher from 'pusher';
import express from 'express';

const app = express();
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Bir kullanıcıyı zorla çıkış yaptırmak için admin endpoint
app.post('/admin/force-logout/:userId', async (req, res) => {
  const userId = req.params.userId;
  
  try {
    // Kullanıcının tüm Pusher bağlantılarını sonlandır
    await pusher.post({
      path: `/users/${userId}/terminate_connections`
    });
    
    // Ayrıca veritabanınızdaki kullanıcı oturumlarını geçersiz kıl
    await invalidateUserSessions(userId);
    
    res.json({ success: true, message: 'Kullanıcı tüm cihazlardan çıkış yaptırıldı' });
  } catch (error) {
    console.error('Bağlantıları sonlandırma başarısız:', error);
    res.status(500).json({ error: 'Kullanıcı çıkış yaptırılamadı' });
  }
});

// Frontend: Zorla çıkışı işle
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
});

pusher.connection.bind('error', (err: any) => {
  if (err.error && err.error.code === 4009) {
    // Kullanıcı zorla çıkış yaptırıldı
    localStorage.clear();
    sessionStorage.clear();
    window.location.href = '/login?reason=forced_logout';
  }
});
```

## Kimlik Doğrulama

Soketi, kimlik doğrulama için HMAC SHA256 imzaları kullanır. Üç tür kimlik doğrulama vardır:

1. **HTTP API Kimlik Doğrulama** - Backend API istekleri için
2. **Özel Kanal Kimlik Doğrulama** - Özel kanal abonelikleri için
3. **Presence Kanal Kimlik Doğrulama** - Kullanıcı bilgisiyle presence kanal abonelikleri için

### HTTP API Kimlik Doğrulama

Tüm HTTP API istekleri HMAC SHA256 imzası kullanılarak doğrulanmalıdır.

#### Kimlik Doğrulama Parametreleri

API isteklerinize şu sorgu parametrelerini ekleyin:

| Parametre | Açıklama | Gerekli |
|-----------|----------|---------|
| `auth_key` | Uygulama anahtarınız | Evet |
| `auth_timestamp` | Saniye cinsinden Unix timestamp | Evet |
| `auth_version` | Kimlik doğrulama sürümü ("1.0" kullanın) | Evet |
| `auth_signature` | HMAC SHA256 imzası | Evet |
| `body_md5` | İstek gövdesinin MD5 hash'i | Evet (POST için) |

#### İmza Oluşturma

İmza HMAC SHA256 kullanılarak oluşturulur:

1. Auth string oluştur: `METHOD\nPATH\nQUERY_STRING`
2. Uygulama secret'ınızı kullanarak HMAC SHA256 ile imzala
3. İmzayı sorgu parametrelerine ekle

**Örnek - Manuel İmza Oluşturma (Node.js)**:

```typescript
import crypto from 'crypto';

function generateAuthSignature(
  method: string,
  path: string,
  appKey: string,
  appSecret: string,
  body: string = ''
): string {
  // Timestamp oluştur
  const timestamp = Math.floor(Date.now() / 1000);
  
  // Body MD5 oluştur (POST istekleri için)
  const bodyMd5 = body ? crypto.createHash('md5').update(body).digest('hex') : '';
  
  // Sorgu parametrelerini oluştur (alfabetik sırayla)
  const params: Record<string, string> = {
    auth_key: appKey,
    auth_timestamp: timestamp.toString(),
    auth_version: '1.0',
  };
  
  if (bodyMd5) {
    params.body_md5 = bodyMd5;
  }
  
  // Parametreleri sırala ve sorgu string'i oluştur
  const sortedParams = Object.keys(params)
    .sort()
    .map(key => `${key}=${params[key]}`)
    .join('&');
  
  // Auth string oluştur
  const authString = `${method}\n${path}\n${sortedParams}`;
  
  // İmza oluştur
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(authString)
    .digest('hex');
  
  return signature;
}

// Kullanım örneği
const method = 'POST';
const path = '/apps/app-id/events';
const appKey = 'your-app-key';
const appSecret = 'your-app-secret';
const body = JSON.stringify({
  name: 'my-event',
  channel: 'my-channel',
  data: JSON.stringify({ message: 'Merhaba!' })
});

const signature = generateAuthSignature(method, path, appKey, appSecret, body);
console.log('İmza:', signature);
```

**Örnek - Pusher SDK Kullanarak (Önerilen)**:

```typescript
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// SDK kimlik doğrulamayı otomatik olarak işler
await pusher.trigger('my-channel', 'my-event', {
  message: 'Merhaba Dünya!'
});
```

#### Timestamp Doğrulama

**Önemli**: Kimlik doğrulama timestamp'leri sunucu zamanının **600 saniye (10 dakika)** içinde olmalıdır. Bu, tekrar saldırılarını önler.

Timestamp'iniz bu pencere dışındaysa, `401 Unauthorized` hatası alırsınız.

**Örnek - Timestamp Doğrulama**:

```typescript
// Geçerli timestamp (geçerli)
const validTimestamp = Math.floor(Date.now() / 1000);

// 5 dakika önceki timestamp (geçerli)
const stillValid = validTimestamp - 300;

// 15 dakika önceki timestamp (geçersiz)
const expired = validTimestamp - 900; // Reddedilecek
```

### Özel Kanal Kimlik Doğrulama

Bir istemci özel bir kanala abone olduğunda, backend'iniz üzerinden kimlik doğrulaması yapmalıdır.

#### Kimlik Doğrulama Akışı

1. İstemci özel bir kanala abone olmaya çalışır
2. Pusher istemcisi auth endpoint'inize POST isteği yapar
3. Backend'iniz kullanıcıyı doğrular ve bir auth imzası oluşturur
4. İstemci imzayı alır ve aboneliği tamamlar

**İstemci İsteği**:

Pusher istemcisi auth endpoint'inize (varsayılan: `/pusher/auth`) POST isteği yapacaktır:

```
POST /pusher/auth
Content-Type: application/x-www-form-urlencoded

socket_id=123.456&channel_name=private-my-channel
```

**Sunucu Yanıtı**:

Backend'iniz imzalanmış bir kimlik doğrulama token'ı döndürmelidir:

```typescript
import Pusher from 'pusher';
import express from 'express';

const app = express();
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channel = req.body.channel_name;
  
  // Kullanıcı kimlik doğrulamasını doğrula (kendi mantığınız)
  if (!req.session || !req.session.user) {
    return res.status(403).json({ error: 'Yetkisiz' });
  }
  
  // Kanal erişimini doğrula (kendi mantığınız)
  if (!canAccessChannel(req.session.user, channel)) {
    return res.status(403).json({ error: 'Yasak' });
  }
  
  // Auth imzası oluştur
  const authResponse = pusher.authorizeChannel(socketId, channel);
  res.json(authResponse);
});
```

**Yanıt Formatı**:

```json
{
  "auth": "app-key:signature"
}
```

**Manuel İmza Oluşturma**:

Pusher SDK kullanmıyorsanız, imzayı manuel olarak oluşturabilirsiniz:

```typescript
import crypto from 'crypto';

function generateChannelAuth(
  appKey: string,
  appSecret: string,
  socketId: string,
  channelName: string
): string {
  // İmzalanacak string oluştur: socket_id:channel_name
  const toSign = `${socketId}:${channelName}`;
  
  // HMAC SHA256 imzası oluştur
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(toSign)
    .digest('hex');
  
  // Format: app_key:signature
  return `${appKey}:${signature}`;
}

// Kullanım
const auth = generateChannelAuth(
  'app-key',
  'app-secret',
  '123.456',
  'private-my-channel'
);

// Yanıt
res.json({ auth });
```

### Presence Kanal Kimlik Doğrulama

Presence kanalları, çevrimiçi kullanıcıları takip etmek için ek kullanıcı bilgisi gerektirir.

**Sunucu Yanıtı**:

```typescript
import Pusher from 'pusher';

app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channel = req.body.channel_name;
  
  // Kullanıcı kimlik doğrulamasını doğrula
  if (!req.session || !req.session.user) {
    return res.status(403).json({ error: 'Yetkisiz' });
  }
  
  // Presence kanalları için kullanıcı verisi sağla
  if (channel.startsWith('presence-')) {
    const presenceData = {
      user_id: req.session.user.id,
      user_info: {
        name: req.session.user.name,
        email: req.session.user.email,
        avatar: req.session.user.avatar,
      },
    };
    
    const authResponse = pusher.authorizeChannel(socketId, channel, presenceData);
    return res.json(authResponse);
  }
  
  // Özel kanallar için kullanıcı verisi gerekmez
  const authResponse = pusher.authorizeChannel(socketId, channel);
  res.json(authResponse);
});
```

**Yanıt Formatı**:

```json
{
  "auth": "app-key:signature",
  "channel_data": "{\"user_id\":\"123\",\"user_info\":{\"name\":\"Ahmet\",\"email\":\"ahmet@example.com\"}}"
}
```

**Presence Kanalları için Manuel İmza Oluşturma**:

```typescript
import crypto from 'crypto';

function generatePresenceAuth(
  appKey: string,
  appSecret: string,
  socketId: string,
  channelName: string,
  userId: string,
  userInfo: Record<string, any>
): { auth: string; channel_data: string } {
  // Kanal verisi oluştur
  const channelData = JSON.stringify({
    user_id: userId,
    user_info: userInfo,
  });
  
  // İmzalanacak string oluştur: socket_id:channel_name:channel_data
  const toSign = `${socketId}:${channelName}:${channelData}`;
  
  // HMAC SHA256 imzası oluştur
  const signature = crypto
    .createHmac('sha256', appSecret)
    .update(toSign)
    .digest('hex');
  
  return {
    auth: `${appKey}:${signature}`,
    channel_data: channelData,
  };
}

// Kullanım
const authResponse = generatePresenceAuth(
  'app-key',
  'app-secret',
  '123.456',
  'presence-room',
  'user-123',
  { name: 'Ahmet Yılmaz', email: 'ahmet@example.com' }
);

res.json(authResponse);
```

### Kullanıcı Kimlik Doğrulama (pusher:signin)

Kullanıcıya özel özellikler için, `pusher:signin` olayıyla kullanıcıları doğrulayabilirsiniz.

**İstemci Tarafı**:

```typescript
// Bağlantı kurulduktan sonra
pusher.signin();
```

**Sunucu Tarafı**:

```typescript
app.post('/pusher/user-auth', (req, res) => {
  const socketId = req.body.socket_id;
  
  // Kullanıcı oturumunu doğrula
  if (!req.session || !req.session.user) {
    return res.status(403).json({ error: 'Yetkisiz' });
  }
  
  const userData = JSON.stringify({
    id: req.session.user.id,
    name: req.session.user.name,
    email: req.session.user.email,
  });
  
  // Kullanıcı auth imzası oluştur
  const toSign = `${socketId}::user::${userData}`;
  const signature = crypto
    .createHmac('sha256', 'app-secret')
    .update(toSign)
    .digest('hex');
  
  res.json({
    auth: `app-key:${signature}`,
    user_data: userData,
  });
});
```

### Kimlik Doğrulama Hata İşleme

**Yaygın Kimlik Doğrulama Hataları**:

| Hata | Açıklama | Çözüm |
|------|----------|--------|
| `401 Unauthorized` | Geçersiz imza veya süresi dolmuş timestamp | App secret ve timestamp'inizi doğrulayın |
| `403 Forbidden` | Kullanıcı kanal için yetkili değil | Yetkilendirme mantığınızı kontrol edin |
| `4008` (WebSocket) | Bağlantı yetkisiz | App key ve kimlik bilgilerini doğrulayın |

**Örnek - Hata İşleme**:

```typescript
// İstemci tarafı
const channel = pusher.subscribe('private-my-channel');

channel.bind('pusher:subscription_error', (status: any) => {
  if (status === 403) {
    console.error('Kanala erişim reddedildi');
  } else if (status === 401) {
    console.error('Kimlik doğrulama başarısız');
  }
});

// Sunucu tarafı
app.post('/pusher/auth', (req, res) => {
  try {
    const socketId = req.body.socket_id;
    const channel = req.body.channel_name;
    
    // Doğrulama mantığınız
    if (!isValidUser(req)) {
      return res.status(403).json({ 
        error: 'Yasak',
        message: 'Kullanıcı bu kanal için yetkili değil'
      });
    }
    
    const authResponse = pusher.authorizeChannel(socketId, channel);
    res.json(authResponse);
  } catch (error) {
    console.error('Auth hatası:', error);
    res.status(500).json({ 
      error: 'Sunucu Hatası',
      message: 'Auth imzası oluşturulamadı'
    });
  }
});
```

## Kanal Tipleri

### Genel Kanallar

- Kimlik doğrulama gerektirmez
- Herkes abone olabilir
- Kanal adı `private-` veya `presence-` ile başlamaz

### Özel Kanallar

- Kimlik doğrulama gerektirir
- Kanal adı `private-` ile başlar
- Kullanıcıya özel veya hassas veriler için kullanılır

### Presence Kanalları

- Kullanıcı bilgisiyle kimlik doğrulama gerektirir
- Kanal adı `presence-` ile başlar
- Çevrimiçi kullanıcıları takip eder
- Üye eklendi/çıkarıldı olayları sağlar

## Olaylar

### Sistem Olayları

| Olay | Açıklama |
|------|----------|
| `pusher:connection_established` | Bağlantı başarılı |
| `pusher:error` | Hata oluştu |
| `pusher:subscription_succeeded` | Kanal aboneliği başarılı |
| `pusher:subscription_error` | Kanal aboneliği başarısız |
| `pusher:member_added` | Kullanıcı presence kanalına katıldı |
| `pusher:member_removed` | Kullanıcı presence kanalından ayrıldı |

### Özel Olaylar

`pusher:` veya `pusher_internal:` ile başlayanlar hariç herhangi bir adla özel olaylar tetikleyebilirsiniz.

## Hata Kodları

| Kod | Açıklama |
|-----|----------|
| 4000 | Uygulama mevcut değil |
| 4001 | Uygulama devre dışı |
| 4003 | Uygulama bağlantı kotası aşıldı |
| 4004 | Yol bulunamadı |
| 4005 | Geçersiz sürüm string formatı |
| 4006 | Desteklenmeyen protokol sürümü |
| 4007 | Protokol sürümü sağlanmadı |
| 4008 | Bağlantı yetkisiz |
| 4009 | Bağlantı limiti aşıldı |
| 4100 | Kapasite aşıldı |
| 4200 | Genel hata |
| 4201 | Pong yanıtı zamanında alınamadı |
| 4202 | Hareketsizlik nedeniyle kapatıldı |

## Hız Limitleri

Hız limitleri uygulama başına yapılandırılır:

- `max_connections`: Maksimum eşzamanlı bağlantı
- `max_backend_events_per_second`: Backend olay hız limiti
- `max_client_events_per_second`: İstemci olay hız limiti
- `max_read_requests_per_second`: Okuma isteği hız limiti

Hız limitleri aşıldığında, istekler 429 durum kodu alacaktır.

## Sağlık Kontrolü ve İzleme Endpoint'leri

Soketi, sağlık kontrolleri, izleme ve gözlemlenebilirlik için çeşitli endpoint'ler sağlar.

### Sağlık Kontrolü

Sunucunun çalıştığını doğrulamak için temel sağlık kontrolü endpoint'i.

**Endpoint**: `GET /`

**Kimlik Doğrulama**: Gerekli değil

**Yanıt**:

```json
{
  "ok": true
}
```

**Örnek**:

```bash
curl http://localhost:6001/
```

**Kullanım Senaryosu**: Temel çalışma süresi izleme

### Hazırlık Kontrolü

Sunucunun bağlantıları kabul etmeye hazır olup olmadığını kontrol edin.

**Endpoint**: `GET /ready`

**Kimlik Doğrulama**: Gerekli değil

**Yanıt (Hazır)**:

```json
{
  "ready": true
}
```

**Yanıt (Hazır Değil)**:

```json
{
  "ready": false,
  "reason": "sunucu kapanıyor"
}
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Sunucu hazır |
| `503` | Sunucu hazır değil (kapanıyor) |

**Örnek**:

```bash
curl http://localhost:6001/ready
```

**Kullanım Senaryosu**: Kubernetes hazırlık probe'ları, yük dengeleyici sağlık kontrolleri

### Trafik Kabul Kontrolü

Sunucunun bellek kullanımına göre trafik kabul edip edemeyeceğini kontrol edin.

**Endpoint**: `GET /accept-traffic`

**Kimlik Doğrulama**: Gerekli değil

**Yanıt (Kabul Edebilir)**:

```json
{
  "accept_traffic": true,
  "used_memory_mb": 512,
  "threshold_mb": 1024
}
```

**Yanıt (Kabul Edemez)**:

```json
{
  "accept_traffic": false,
  "reason": "bellek eşiği aşıldı",
  "used_memory_mb": 1100,
  "threshold_mb": 1024
}
```

**Durum Kodları**:

| Kod | Açıklama |
|-----|----------|
| `200` | Sunucu trafik kabul edebilir |
| `503` | Sunucu trafik kabul edemez (bellek eşiği aşıldı veya kapanıyor) |

**Örnek**:

```bash
curl http://localhost:6001/accept-traffic
```

**Yapılandırma**:

Bellek eşiği sunucu yapılandırmanızda yapılandırılabilir:

```json
{
  "http_api": {
    "accept_traffic_memory_threshold_mb": 1024
  }
}
```

**Kullanım Senaryosu**: Bellek farkındalığı ile yük dengeleyici sağlık kontrolleri

### Bellek Kullanımı

Detaylı bellek kullanım bilgisi alın.

**Endpoint**: `GET /usage`

**Kimlik Doğrulama**: Gerekli değil

**Yanıt**:

```json
{
  "memory": {
    "total_mb": 2048,
    "used_mb": 512,
    "free_mb": 1536,
    "available_mb": 1600,
    "usage_percent": 25.0
  }
}
```

**Örnek**:

```bash
curl http://localhost:6001/usage
```

**Kullanım Senaryosu**: İzleme panoları, kapasite planlaması

### Metrikler (Prometheus)

İzleme için Prometheus uyumlu metrikler alın.

**Endpoint**: `GET /metrics`

**Kimlik Doğrulama**: Gerekli değil

**Sorgu Parametreleri**:

| Parametre | Açıklama | Varsayılan |
|-----------|----------|------------|
| `format` | Yanıt formatı: `text` veya `json` | `text` |

**Yanıt (Düz Metin - Prometheus formatı)**:

```
# HELP soketi_connections_total Toplam WebSocket bağlantı sayısı
# TYPE soketi_connections_total gauge
soketi_connections_total 42

# HELP soketi_messages_sent_total Gönderilen toplam mesaj sayısı
# TYPE soketi_messages_sent_total counter
soketi_messages_sent_total 1234

# HELP soketi_messages_received_total Alınan toplam mesaj sayısı
# TYPE soketi_messages_received_total counter
soketi_messages_received_total 5678
```

**Yanıt (JSON formatı)**:

```json
{
  "connections_total": 42,
  "messages_sent_total": 1234,
  "messages_received_total": 5678,
  "channels_total": 15,
  "http_requests_total": 890
}
```

**Örnek - Düz Metin (Prometheus)**:

```bash
curl http://localhost:6001/metrics
```

**Örnek - JSON**:

```bash
curl "http://localhost:6001/metrics?format=json"
```

**Prometheus Yapılandırması**:

Bunu `prometheus.yml` dosyanıza ekleyin:

```yaml
scrape_configs:
  - job_name: 'soketi'
    static_configs:
      - targets: ['localhost:6001']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

**Kullanım Senaryosu**: Prometheus izleme, Grafana panoları

**Not**: Metrikler endpoint'i sadece sunucu yapılandırmasında metrikler etkinleştirilmişse kullanılabilir.

### Sağlık Kontrolü Örnekleri

**Docker Sağlık Kontrolü**:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:6001/ready || exit 1
```

**Kubernetes Probe'ları**:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: soketi
spec:
  containers:
  - name: soketi
    image: soketi:latest
    ports:
    - containerPort: 6001
    livenessProbe:
      httpGet:
        path: /
        port: 6001
      initialDelaySeconds: 5
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: 6001
      initialDelaySeconds: 5
      periodSeconds: 5
```

**Yük Dengeleyici Sağlık Kontrolü (Nginx)**:

```nginx
upstream soketi_backend {
    server soketi:6001 max_fails=3 fail_timeout=30s;
    
    # Sağlık kontrolü
    check interval=5000 rise=2 fall=3 timeout=3000 type=http;
    check_http_send "GET /accept-traffic HTTP/1.0\r\n\r\n";
    check_http_expect_alive http_2xx;
}
```

## Sonraki Adımlar

- **[Başlangıç](baslangic.md)** - Hızlı başlangıç kılavuzu
- **[Örnekler](ornekler/temel-chat.md)** - Kod örnekleri
- **[Yapılandırma](yapilandirma.md)** - Yapılandırma referansı

## İlgili Kaynaklar

- [Pusher Protokol Dokümantasyonu](https://pusher.com/docs/channels/library_auth_reference/pusher-websockets-protocol)
- [Pusher JavaScript İstemcisi](https://github.com/pusher/pusher-js)
- [Pusher Server SDK'ları](https://pusher.com/docs/channels/channels_libraries/libraries)
