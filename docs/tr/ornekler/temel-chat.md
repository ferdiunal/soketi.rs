# Temel Chat Uygulaması

> soketi.rs ve Pusher SDK kullanarak basit bir gerçek zamanlı chat uygulaması

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Koşullar](#ön-koşullar)
- [Kurulum Talimatları](#kurulum-talimatları)
- [TypeScript Uygulaması](#typescript-uygulaması)
- [JavaScript Uygulaması](#javascript-uygulaması)
- [Beklenen Çıktı](#beklenen-çıktı)
- [Sonraki Adımlar](#sonraki-adımlar)

## Genel Bakış

Bu örnek, soketi.rs'yi WebSocket sunucusu ve Pusher JavaScript SDK'yı istemci tarafı iletişimi için kullanarak temel bir gerçek zamanlı chat uygulamasının nasıl oluşturulacağını gösterir. Kullanıcılar, genel bir kanal üzerinden gerçek zamanlı olarak mesaj gönderip alabilirler.

## Ön Koşullar

- Node.js 18+ kurulu
- soketi.rs sunucusu çalışıyor ([Başlangıç Kılavuzu](../baslangic.md)'na bakın)
- JavaScript/TypeScript temel bilgisi
- Bir metin editörü veya IDE

## Kurulum Talimatları

### 1. Bağımlılıkları Yükleyin

```bash
npm install pusher-js
# veya
yarn add pusher-js
```

### 2. soketi.rs Sunucusunu Yapılandırın

soketi.rs sunucunuzun aşağıdaki yapılandırma ile çalıştığından emin olun:

```bash
# soketi.rs'yi varsayılan ayarlarla başlatın
soketi start --port=6001 --app-id=app-id --key=app-key --secret=app-secret
```

### 3. HTML Dosyası Oluşturun

Bir `index.html` dosyası oluşturun:

```html
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Temel Chat - soketi.rs</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
        }
        #messages {
            border: 1px solid #ccc;
            height: 300px;
            overflow-y: auto;
            padding: 10px;
            margin-bottom: 10px;
            background: #f9f9f9;
        }
        .message {
            margin: 5px 0;
            padding: 5px;
            background: white;
            border-radius: 3px;
        }
        .message-user {
            font-weight: bold;
            color: #0066cc;
        }
        .message-time {
            font-size: 0.8em;
            color: #666;
        }
        #input-container {
            display: flex;
            gap: 10px;
        }
        #message-input {
            flex: 1;
            padding: 10px;
            border: 1px solid #ccc;
            border-radius: 3px;
        }
        #send-button {
            padding: 10px 20px;
            background: #0066cc;
            color: white;
            border: none;
            border-radius: 3px;
            cursor: pointer;
        }
        #send-button:hover {
            background: #0052a3;
        }
        #status {
            padding: 10px;
            margin-bottom: 10px;
            border-radius: 3px;
            text-align: center;
        }
        .connected {
            background: #d4edda;
            color: #155724;
        }
        .disconnected {
            background: #f8d7da;
            color: #721c24;
        }
    </style>
</head>
<body>
    <h1>Temel Chat Uygulaması</h1>
    <div id="status" class="disconnected">Bağlantı Kesildi</div>
    <div id="messages"></div>
    <div id="input-container">
        <input type="text" id="message-input" placeholder="Mesajınızı yazın..." />
        <button id="send-button">Gönder</button>
    </div>
    <script src="https://js.pusher.com/8.2.0/pusher.min.js"></script>
    <script src="chat.js"></script>
</body>
</html>
```

## TypeScript Uygulaması

Bir `chat.ts` dosyası oluşturun:

```typescript
import Pusher from 'pusher-js';

// Yapılandırma
interface ChatConfig {
  appKey: string;
  wsHost: string;
  wsPort: number;
  forceTLS: boolean;
  enabledTransports: string[];
}

interface Message {
  user: string;
  text: string;
  timestamp: string;
}

// Pusher istemcisini başlat
const config: ChatConfig = {
  appKey: 'app-key',
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
};

const pusher = new Pusher(config.appKey, {
  wsHost: config.wsHost,
  wsPort: config.wsPort,
  forceTLS: config.forceTLS,
  enabledTransports: config.enabledTransports,
  cluster: 'mt1',
  disableStats: true,
});

// Genel bir kanala abone ol
const channel = pusher.subscribe('chat-room');

// DOM elementleri
const messagesDiv = document.getElementById('messages') as HTMLDivElement;
const messageInput = document.getElementById('message-input') as HTMLInputElement;
const sendButton = document.getElementById('send-button') as HTMLButtonElement;
const statusDiv = document.getElementById('status') as HTMLDivElement;

// Rastgele bir kullanıcı adı oluştur
const username = `Kullanici${Math.floor(Math.random() * 1000)}`;

// Bağlantı durumu işleyicileri
pusher.connection.bind('connected', () => {
  console.log('soketi.rs\'ye bağlandı');
  statusDiv.textContent = 'Bağlandı';
  statusDiv.className = 'connected';
});

pusher.connection.bind('disconnected', () => {
  console.log('soketi.rs bağlantısı kesildi');
  statusDiv.textContent = 'Bağlantı Kesildi';
  statusDiv.className = 'disconnected';
});

pusher.connection.bind('error', (err: any) => {
  console.error('Bağlantı hatası:', err);
  statusDiv.textContent = 'Bağlantı Hatası';
  statusDiv.className = 'disconnected';
});

// Mesajları dinle
channel.bind('message', (data: Message) => {
  displayMessage(data);
});

// Mesajı UI'da göster
function displayMessage(data: Message): void {
  const messageEl = document.createElement('div');
  messageEl.className = 'message';
  
  const time = new Date(data.timestamp).toLocaleTimeString('tr-TR');
  
  messageEl.innerHTML = `
    <span class="message-user">${data.user}:</span>
    <span class="message-text">${escapeHtml(data.text)}</span>
    <span class="message-time">${time}</span>
  `;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// XSS'i önlemek için HTML'i kaçır
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Mesaj gönder
function sendMessage(): void {
  const text = messageInput.value.trim();
  
  if (!text) {
    return;
  }
  
  const message: Message = {
    user: username,
    text: text,
    timestamp: new Date().toISOString(),
  };
  
  // İstemci olayını tetikle (istemci olaylarının etkinleştirilmesi gerekir)
  channel.trigger('client-message', message);
  
  // Girdiyi temizle
  messageInput.value = '';
}

// Olay dinleyicileri
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Hoş geldin mesajını göster
setTimeout(() => {
  displayMessage({
    user: 'Sistem',
    text: `Hoş geldin ${username}! Sohbete başla...`,
    timestamp: new Date().toISOString(),
  });
}, 500);
```

### TypeScript'i Derleyin

```bash
# TypeScript henüz kurulu değilse kurun
npm install -g typescript

# Derle
tsc chat.ts --target ES2015 --module ES2015 --lib ES2015,DOM
```

## JavaScript Uygulaması

Bir `chat.js` dosyası oluşturun:

```javascript
// Pusher istemcisini başlat
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1',
  disableStats: true,
});

// Genel bir kanala abone ol
const channel = pusher.subscribe('chat-room');

// DOM elementleri
const messagesDiv = document.getElementById('messages');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const statusDiv = document.getElementById('status');

// Rastgele bir kullanıcı adı oluştur
const username = `Kullanici${Math.floor(Math.random() * 1000)}`;

// Bağlantı durumu işleyicileri
pusher.connection.bind('connected', () => {
  console.log('soketi.rs\'ye bağlandı');
  statusDiv.textContent = 'Bağlandı';
  statusDiv.className = 'connected';
});

pusher.connection.bind('disconnected', () => {
  console.log('soketi.rs bağlantısı kesildi');
  statusDiv.textContent = 'Bağlantı Kesildi';
  statusDiv.className = 'disconnected';
});

pusher.connection.bind('error', (err) => {
  console.error('Bağlantı hatası:', err);
  statusDiv.textContent = 'Bağlantı Hatası';
  statusDiv.className = 'disconnected';
});

// Mesajları dinle
channel.bind('message', (data) => {
  displayMessage(data);
});

// Mesajı UI'da göster
function displayMessage(data) {
  const messageEl = document.createElement('div');
  messageEl.className = 'message';
  
  const time = new Date(data.timestamp).toLocaleTimeString('tr-TR');
  
  messageEl.innerHTML = `
    <span class="message-user">${data.user}:</span>
    <span class="message-text">${escapeHtml(data.text)}</span>
    <span class="message-time">${time}</span>
  `;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// XSS'i önlemek için HTML'i kaçır
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Mesaj gönder
function sendMessage() {
  const text = messageInput.value.trim();
  
  if (!text) {
    return;
  }
  
  const message = {
    user: username,
    text: text,
    timestamp: new Date().toISOString(),
  };
  
  // İstemci olayını tetikle (istemci olaylarının etkinleştirilmesi gerekir)
  channel.trigger('client-message', message);
  
  // Girdiyi temizle
  messageInput.value = '';
}

// Olay dinleyicileri
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Hoş geldin mesajını göster
setTimeout(() => {
  displayMessage({
    user: 'Sistem',
    text: `Hoş geldin ${username}! Sohbete başla...`,
    timestamp: new Date().toISOString(),
  });
}, 500);
```

## Beklenen Çıktı

### Konsol Çıktısı

Uygulamayı tarayıcınızda açtığınızda şunu görmelisiniz:

```
soketi.rs'ye bağlandı
```

### Tarayıcı Görünümü

1. **Durum Çubuğu**: soketi.rs'ye başarıyla bağlandığında yeşil renkte "Bağlandı" gösterir
2. **Mesajlar Alanı**: Tüm chat mesajlarını kullanıcı adı, metin ve zaman damgası ile gösterir
3. **Hoş Geldin Mesajı**: "Sistem: Hoş geldin Kullanici123! Sohbete başla..."
4. **Giriş Alanı**: Mesaj yazmak için metin girişi
5. **Gönder Butonu**: Mesaj göndermek için buton

### Mesaj Akışı

1. Kullanıcı bir mesaj yazar ve "Gönder"e tıklar veya Enter'a basar
2. Mesaj, `chat-room` kanalı üzerinden soketi.rs sunucusuna gönderilir
3. Bağlı tüm istemciler mesajı gerçek zamanlı olarak alır
4. Mesaj, kullanıcı adı ve zaman damgası ile mesajlar alanında görünür

### Örnek Mesajlar

```
Sistem: Hoş geldin Kullanici123! Sohbete başla... 10:30:15
Kullanici123: Herkese merhaba! 10:30:20
Kullanici456: Selam! 10:30:25
Kullanici123: Nasılsınız? 10:30:30
```

## Sonraki Adımlar

- **Özel Kanallar Ekleyin**: [Özel Kanallar Örneği](./ozel-kanallar.md)'nde kimlik doğrulama ile özel kanalların nasıl uygulanacağını öğrenin
- **Presence Ekleyin**: [Presence Kanalları Örneği](./presence.md) ile kimlerin çevrimiçi olduğunu görün
- **Kimlik Doğrulama Ekleyin**: [Kimlik Doğrulama Örneği](./kimlik-dogrulama.md) ile chat'inizi güvenli hale getirin
- **Sunucu Tarafı Olaylar**: Backend'inizden olayları nasıl tetikleyeceğinizi öğrenin
- **Mesaj Geçmişi**: Bir veritabanı ile mesaj kalıcılığını uygulayın
- **Kullanıcı Profilleri**: Avatar ve kullanıcı profilleri ekleyin
- **Yazma Göstergeleri**: Kullanıcılar yazarken gösterin
- **Dosya Paylaşımı**: Kullanıcıların resim ve dosya paylaşmasına izin verin

## İlgili Dokümantasyon

- [Başlangıç Kılavuzu](../baslangic.md)
- [API Referansı](../api-referans.md)
- [Yapılandırma Kılavuzu](../yapilandirma.md)
- [Sorun Giderme](../sorun-giderme.md)
