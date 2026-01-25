# Presence Kanalları

> soketi.rs ve Pusher SDK ile gerçek zamanlı kullanıcı varlık takibi

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Koşullar](#ön-koşullar)
- [Kurulum Talimatları](#kurulum-talimatları)
- [Sunucu Tarafı Kimlik Doğrulama](#sunucu-tarafı-kimlik-doğrulama)
- [TypeScript Uygulaması](#typescript-uygulaması)
- [JavaScript Uygulaması](#javascript-uygulaması)
- [Beklenen Çıktı](#beklenen-çıktı)
- [Gelişmiş Özellikler](#gelişmiş-özellikler)

## Genel Bakış

Presence kanalları, bir kanala şu anda hangi kullanıcıların abone olduğunu takip etmenizi sağlar. Bu, şu özellikler için mükemmeldir:

- Bir chat odasında kimlerin çevrimiçi olduğunu gösterme
- İşbirlikçi bir belgede aktif kullanıcıları görüntüleme
- Çok oyunculu oyun lobilerini oluşturma
- "Kim görüntülüyor" göstergeleri oluşturma

Genel kanalların aksine, presence kanalları kimlik doğrulama gerektirir ve tüm abonelere üye bilgilerini sağlar.

## Ön Koşullar

- Node.js 18+ kurulu
- soketi.rs sunucusu çalışıyor ([Başlangıç Kılavuzu](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/baslangic.md)'na bakın)
- Kimlik doğrulama için Express.js veya benzer backend framework
- JavaScript/TypeScript temel bilgisi
- [Temel Chat Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/temel-chat.md)'ni anlama

## Kurulum Talimatları

### 1. Bağımlılıkları Yükleyin

**İstemci tarafı:**
```bash
npm install pusher-js
```

**Sunucu tarafı:**
```bash
npm install express pusher cors body-parser
```

### 2. soketi.rs Sunucusunu Yapılandırın

soketi.rs'yi istemci olayları etkinleştirilmiş olarak başlatın:

```bash
soketi start \
  --port=6001 \
  --app-id=app-id \
  --key=app-key \
  --secret=app-secret \
  --enable-client-messages
```

### 3. HTML Dosyası Oluşturun

Bir `presence.html` dosyası oluşturun:

```html
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Presence Kanalı - soketi.rs</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        .container {
            display: flex;
            gap: 20px;
        }
        .chat-section {
            flex: 2;
        }
        .users-section {
            flex: 1;
            border-left: 2px solid #ccc;
            padding-left: 20px;
        }
        #messages {
            border: 1px solid #ccc;
            height: 400px;
            overflow-y: auto;
            padding: 10px;
            margin-bottom: 10px;
            background: #f9f9f9;
        }
        .message {
            margin: 5px 0;
            padding: 8px;
            background: white;
            border-radius: 3px;
        }
        .message-user {
            font-weight: bold;
            color: #0066cc;
        }
        .system-message {
            color: #666;
            font-style: italic;
        }
        #users-list {
            list-style: none;
            padding: 0;
        }
        .user-item {
            padding: 10px;
            margin: 5px 0;
            background: #f0f0f0;
            border-radius: 3px;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .user-status {
            width: 10px;
            height: 10px;
            background: #28a745;
            border-radius: 50%;
        }
        .user-info {
            flex: 1;
        }
        .user-name {
            font-weight: bold;
        }
        .user-email {
            font-size: 0.85em;
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
        button {
            padding: 10px 20px;
            background: #0066cc;
            color: white;
            border: none;
            border-radius: 3px;
            cursor: pointer;
        }
        button:hover {
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
        .member-count {
            font-size: 0.9em;
            color: #666;
            margin-bottom: 10px;
        }
    </style>
</head>
<body>
    <h1>Presence Kanalı Demo</h1>
    <div id="status" class="disconnected">Bağlantı Kesildi</div>
    
    <div class="container">
        <div class="chat-section">
            <h2>Sohbet</h2>
            <div id="messages"></div>
            <div id="input-container">
                <input type="text" id="message-input" placeholder="Mesajınızı yazın..." />
                <button id="send-button">Gönder</button>
            </div>
        </div>
        
        <div class="users-section">
            <h2>Çevrimiçi Kullanıcılar</h2>
            <div class="member-count">
                <span id="member-count">0</span> kullanıcı çevrimiçi
            </div>
            <ul id="users-list"></ul>
        </div>
    </div>
    
    <script src="https://js.pusher.com/8.2.0/pusher.min.js"></script>
    <script src="presence.js"></script>
</body>
</html>
```

## Sunucu Tarafı Kimlik Doğrulama

### TypeScript Sunucu Uygulaması

Bir `auth-server.ts` dosyası oluşturun:

```typescript
import express, { Request, Response } from 'express';
import Pusher from 'pusher';
import cors from 'cors';
import bodyParser from 'body-parser';

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

// Pusher sunucu SDK'sını başlat
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  cluster: 'mt1',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Mock kullanıcı veritabanı (gerçek kimlik doğrulama ile değiştirin)
interface User {
  id: string;
  name: string;
  email: string;
}

const users: Record<string, User> = {
  'user-1': { id: 'user-1', name: 'Ayşe Yılmaz', email: 'ayse@example.com' },
  'user-2': { id: 'user-2', name: 'Mehmet Demir', email: 'mehmet@example.com' },
  'user-3': { id: 'user-3', name: 'Fatma Kaya', email: 'fatma@example.com' },
};

// Presence kanalları için kimlik doğrulama endpoint'i
app.post('/pusher/auth', (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  
  // Oturum/token'dan kullanıcıyı al (demo için basitleştirilmiş)
  const userId = req.query.user_id as string || 'user-1';
  const user = users[userId];
  
  if (!user) {
    return res.status(403).json({ error: 'Kullanıcı bulunamadı' });
  }
  
  // Sadece presence kanallarını doğrula
  if (!channelName.startsWith('presence-')) {
    return res.status(403).json({ error: 'Geçersiz kanal' });
  }
  
  // Diğer kullanıcılarla paylaşılacak presence verisi
  const presenceData = {
    user_id: user.id,
    user_info: {
      name: user.name,
      email: user.email,
    },
  };
  
  try {
    const authResponse = pusher.authorizeChannel(socketId, channelName, presenceData);
    res.json(authResponse);
  } catch (error) {
    console.error('Kimlik doğrulama hatası:', error);
    res.status(500).json({ error: 'Kimlik doğrulama başarısız' });
  }
});

// Sunucuyu başlat
const PORT = 3000;
app.listen(PORT, () => {
  console.log(`Kimlik doğrulama sunucusu http://localhost:${PORT} adresinde çalışıyor`);
});
```

### JavaScript Sunucu Uygulaması

Bir `auth-server.js` dosyası oluşturun:

```javascript
const express = require('express');
const Pusher = require('pusher');
const cors = require('cors');
const bodyParser = require('body-parser');

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

// Pusher sunucu SDK'sını başlat
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  cluster: 'mt1',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Mock kullanıcı veritabanı (gerçek kimlik doğrulama ile değiştirin)
const users = {
  'user-1': { id: 'user-1', name: 'Ayşe Yılmaz', email: 'ayse@example.com' },
  'user-2': { id: 'user-2', name: 'Mehmet Demir', email: 'mehmet@example.com' },
  'user-3': { id: 'user-3', name: 'Fatma Kaya', email: 'fatma@example.com' },
};

// Presence kanalları için kimlik doğrulama endpoint'i
app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  
  // Oturum/token'dan kullanıcıyı al (demo için basitleştirilmiş)
  const userId = req.query.user_id || 'user-1';
  const user = users[userId];
  
  if (!user) {
    return res.status(403).json({ error: 'Kullanıcı bulunamadı' });
  }
  
  // Sadece presence kanallarını doğrula
  if (!channelName.startsWith('presence-')) {
    return res.status(403).json({ error: 'Geçersiz kanal' });
  }
  
  // Diğer kullanıcılarla paylaşılacak presence verisi
  const presenceData = {
    user_id: user.id,
    user_info: {
      name: user.name,
      email: user.email,
    },
  };
  
  try {
    const authResponse = pusher.authorizeChannel(socketId, channelName, presenceData);
    res.json(authResponse);
  } catch (error) {
    console.error('Kimlik doğrulama hatası:', error);
    res.status(500).json({ error: 'Kimlik doğrulama başarısız' });
  }
});

// Sunucuyu başlat
const PORT = 3000;
app.listen(PORT, () => {
  console.log(`Kimlik doğrulama sunucusu http://localhost:${PORT} adresinde çalışıyor`);
});
```

## TypeScript Uygulaması

Bir `presence.ts` dosyası oluşturun:

```typescript
import Pusher, { PresenceChannel } from 'pusher-js';

// Yapılandırma
interface PresenceMember {
  id: string;
  info: {
    name: string;
    email: string;
  };
}

interface Message {
  user: string;
  text: string;
  timestamp: string;
}

// Kullanıcı ID'sini simüle et (production'da kimlik doğrulamadan al)
const currentUserId = `user-${Math.floor(Math.random() * 3) + 1}`;

// Pusher istemcisini başlat
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1',
  disableStats: true,
  authEndpoint: `http://localhost:3000/pusher/auth?user_id=${currentUserId}`,
});

// Presence kanalına abone ol
const channel = pusher.subscribe('presence-chat-room') as PresenceChannel;

// DOM elementleri
const messagesDiv = document.getElementById('messages') as HTMLDivElement;
const messageInput = document.getElementById('message-input') as HTMLInputElement;
const sendButton = document.getElementById('send-button') as HTMLButtonElement;
const statusDiv = document.getElementById('status') as HTMLDivElement;
const usersListDiv = document.getElementById('users-list') as HTMLUListElement;
const memberCountSpan = document.getElementById('member-count') as HTMLSpanElement;

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

// Presence kanalı olayları
channel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Presence kanalına başarıyla abone olundu');
  updateUsersList(members);
  
  displaySystemMessage(`${members.me.info.name} olarak odaya katıldınız`);
});

channel.bind('pusher:member_added', (member: PresenceMember) => {
  console.log('Üye eklendi:', member);
  addUserToList(member);
  displaySystemMessage(`${member.info.name} odaya katıldı`);
});

channel.bind('pusher:member_removed', (member: PresenceMember) => {
  console.log('Üye çıkarıldı:', member);
  removeUserFromList(member.id);
  displaySystemMessage(`${member.info.name} odadan ayrıldı`);
});

// Chat mesajı olayı
channel.bind('client-message', (data: Message) => {
  displayMessage(data);
});

// Kullanıcı listesini güncelle
function updateUsersList(members: any): void {
  usersListDiv.innerHTML = '';
  memberCountSpan.textContent = members.count.toString();
  
  members.each((member: PresenceMember) => {
    addUserToList(member);
  });
}

// Kullanıcıyı listeye ekle
function addUserToList(member: PresenceMember): void {
  const existingUser = document.getElementById(`user-${member.id}`);
  if (existingUser) return;
  
  const userEl = document.createElement('li');
  userEl.id = `user-${member.id}`;
  userEl.className = 'user-item';
  userEl.innerHTML = `
    <div class="user-status"></div>
    <div class="user-info">
      <div class="user-name">${member.info.name}</div>
      <div class="user-email">${member.info.email}</div>
    </div>
  `;
  
  usersListDiv.appendChild(userEl);
  updateMemberCount();
}

// Kullanıcıyı listeden çıkar
function removeUserFromList(userId: string): void {
  const userEl = document.getElementById(`user-${userId}`);
  if (userEl) {
    userEl.remove();
    updateMemberCount();
  }
}

// Üye sayısını güncelle
function updateMemberCount(): void {
  const count = usersListDiv.children.length;
  memberCountSpan.textContent = count.toString();
}

// Chat mesajını göster
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

// Sistem mesajını göster
function displaySystemMessage(text: string): void {
  const messageEl = document.createElement('div');
  messageEl.className = 'message system-message';
  messageEl.textContent = text;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// HTML'i kaçır
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Mesaj gönder
function sendMessage(): void {
  const text = messageInput.value.trim();
  
  if (!text) return;
  
  const member = channel.members.me;
  const message: Message = {
    user: member.info.name,
    text: text,
    timestamp: new Date().toISOString(),
  };
  
  channel.trigger('client-message', message);
  messageInput.value = '';
}

// Olay dinleyicileri
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});
```

## JavaScript Uygulaması

Bir `presence.js` dosyası oluşturun:

```javascript
// Kullanıcı ID'sini simüle et (production'da kimlik doğrulamadan al)
const currentUserId = `user-${Math.floor(Math.random() * 3) + 1}`;

// Pusher istemcisini başlat
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1',
  disableStats: true,
  authEndpoint: `http://localhost:3000/pusher/auth?user_id=${currentUserId}`,
});

// Presence kanalına abone ol
const channel = pusher.subscribe('presence-chat-room');

// DOM elementleri
const messagesDiv = document.getElementById('messages');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const statusDiv = document.getElementById('status');
const usersListDiv = document.getElementById('users-list');
const memberCountSpan = document.getElementById('member-count');

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

// Presence kanalı olayları
channel.bind('pusher:subscription_succeeded', (members) => {
  console.log('Presence kanalına başarıyla abone olundu');
  updateUsersList(members);
  
  displaySystemMessage(`${members.me.info.name} olarak odaya katıldınız`);
});

channel.bind('pusher:member_added', (member) => {
  console.log('Üye eklendi:', member);
  addUserToList(member);
  displaySystemMessage(`${member.info.name} odaya katıldı`);
});

channel.bind('pusher:member_removed', (member) => {
  console.log('Üye çıkarıldı:', member);
  removeUserFromList(member.id);
  displaySystemMessage(`${member.info.name} odadan ayrıldı`);
});

// Chat mesajı olayı
channel.bind('client-message', (data) => {
  displayMessage(data);
});

// Kullanıcı listesini güncelle
function updateUsersList(members) {
  usersListDiv.innerHTML = '';
  memberCountSpan.textContent = members.count;
  
  members.each((member) => {
    addUserToList(member);
  });
}

// Kullanıcıyı listeye ekle
function addUserToList(member) {
  const existingUser = document.getElementById(`user-${member.id}`);
  if (existingUser) return;
  
  const userEl = document.createElement('li');
  userEl.id = `user-${member.id}`;
  userEl.className = 'user-item';
  userEl.innerHTML = `
    <div class="user-status"></div>
    <div class="user-info">
      <div class="user-name">${member.info.name}</div>
      <div class="user-email">${member.info.email}</div>
    </div>
  `;
  
  usersListDiv.appendChild(userEl);
  updateMemberCount();
}

// Kullanıcıyı listeden çıkar
function removeUserFromList(userId) {
  const userEl = document.getElementById(`user-${userId}`);
  if (userEl) {
    userEl.remove();
    updateMemberCount();
  }
}

// Üye sayısını güncelle
function updateMemberCount() {
  const count = usersListDiv.children.length;
  memberCountSpan.textContent = count;
}

// Chat mesajını göster
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

// Sistem mesajını göster
function displaySystemMessage(text) {
  const messageEl = document.createElement('div');
  messageEl.className = 'message system-message';
  messageEl.textContent = text;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// HTML'i kaçır
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Mesaj gönder
function sendMessage() {
  const text = messageInput.value.trim();
  
  if (!text) return;
  
  const member = channel.members.me;
  const message = {
    user: member.info.name,
    text: text,
    timestamp: new Date().toISOString(),
  };
  
  channel.trigger('client-message', message);
  messageInput.value = '';
}

// Olay dinleyicileri
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});
```

## Beklenen Çıktı

### Sunucu Konsolu

```
Kimlik doğrulama sunucusu http://localhost:3000 adresinde çalışıyor
```

### Tarayıcı Konsolu

```
soketi.rs'ye bağlandı
Presence kanalına başarıyla abone olundu
Üye eklendi: { id: 'user-2', info: { name: 'Mehmet Demir', email: 'mehmet@example.com' } }
```

### Tarayıcı Görünümü

**Durum Çubuğu:**
```
Bağlandı
```

**Çevrimiçi Kullanıcılar Paneli:**
```
Çevrimiçi Kullanıcılar
2 kullanıcı çevrimiçi

● Ayşe Yılmaz
  ayse@example.com

● Mehmet Demir
  mehmet@example.com
```

**Chat Mesajları:**
```
Ayşe Yılmaz olarak odaya katıldınız
Mehmet Demir odaya katıldı
Ayşe Yılmaz: Herkese merhaba!
Mehmet Demir: Selam Ayşe!
Fatma Kaya odaya katıldı
Fatma Kaya: Merhaba arkadaşlar!
Mehmet Demir odadan ayrıldı
```

## Gelişmiş Özellikler

### 1. Yazma Göstergeleri

```typescript
let typingTimeout: NodeJS.Timeout;

messageInput.addEventListener('input', () => {
  channel.trigger('client-typing', {
    user: channel.members.me.info.name,
  });
  
  clearTimeout(typingTimeout);
  typingTimeout = setTimeout(() => {
    channel.trigger('client-stopped-typing', {
      user: channel.members.me.info.name,
    });
  }, 1000);
});

channel.bind('client-typing', (data: { user: string }) => {
  // Yazma göstergesini göster
  console.log(`${data.user} yazıyor...`);
});
```

### 2. Kullanıcı Durumu

```typescript
// Kullanıcı durumunu güncelle
function updateUserStatus(status: 'online' | 'away' | 'busy') {
  channel.trigger('client-status-change', {
    userId: channel.members.me.id,
    status: status,
  });
}
```

### 3. Doğrudan Mesajlar

```typescript
// Belirli bir kullanıcıya doğrudan mesaj gönder
function sendDirectMessage(recipientId: string, message: string) {
  channel.trigger('client-direct-message', {
    from: channel.members.me.id,
    to: recipientId,
    message: message,
  });
}
```

## İlgili Dokümantasyon

- [Temel Chat Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/temel-chat.md)
- [Özel Kanallar Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/ozel-kanallar.md)
- [Kimlik Doğrulama Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/kimlik-dogrulama.md)
- [API Referansı](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/api-referans.md)
- [Başlangıç Kılavuzu](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/baslangic.md)
