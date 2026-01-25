# Özel Kanallar

> soketi.rs ve Pusher SDK ile güvenli kanal kimlik doğrulama

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Koşullar](#ön-koşullar)
- [Kurulum Talimatları](#kurulum-talimatları)
- [Sunucu Tarafı Kimlik Doğrulama](#sunucu-tarafı-kimlik-doğrulama)
- [TypeScript Uygulaması](#typescript-uygulaması)
- [JavaScript Uygulaması](#javascript-uygulaması)
- [Beklenen Çıktı](#beklenen-çıktı)
- [Güvenlik En İyi Uygulamaları](#güvenlik-en-iyi-uygulamaları)

## Genel Bakış

Özel kanallar, güvenli ve kimliği doğrulanmış iletişim kanalları sağlar. Genel kanalların aksine, özel kanallar bir istemcinin abone olabilmesi için sunucu tarafı kimlik doğrulama gerektirir. Bu şunlar için gereklidir:

- Kullanıcıya özel bildirimler
- Kullanıcılar arası özel mesajlaşma
- Güvenli veri iletimi
- Erişim kontrollü özellikler
- Çok kiracılı uygulamalar

Özel kanallar `private-` öneki ile başlar ve abonelik isteklerini doğrulayan bir kimlik doğrulama endpoint'i gerektirir.

## Ön Koşullar

- Node.js 18+ kurulu
- soketi.rs sunucusu çalışıyor ([Başlangıç Kılavuzu](../baslangic.md)'na bakın)
- Express.js veya benzer backend framework
- JavaScript/TypeScript temel bilgisi
- [Temel Chat Örneği](./temel-chat.md)'ni anlama

## Kurulum Talimatları

### 1. Bağımlılıkları Yükleyin

**İstemci tarafı:**
```bash
npm install pusher-js
```

**Sunucu tarafı:**
```bash
npm install express pusher cors body-parser jsonwebtoken
```

### 2. soketi.rs Sunucusunu Yapılandırın

soketi.rs'yi başlatın:

```bash
soketi start \
  --port=6001 \
  --app-id=app-id \
  --key=app-key \
  --secret=app-secret
```

### 3. HTML Dosyası Oluşturun

Bir `private-channel.html` dosyası oluşturun:

```html
<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Özel Kanal - soketi.rs</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
        }
        .auth-section {
            margin-bottom: 30px;
            padding: 20px;
            background: #f5f5f5;
            border-radius: 5px;
        }
        .auth-section input {
            width: 100%;
            padding: 10px;
            margin: 10px 0;
            border: 1px solid #ccc;
            border-radius: 3px;
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
        .error-message {
            color: #dc3545;
            font-weight: bold;
        }
        button {
            padding: 10px 20px;
            background: #0066cc;
            color: white;
            border: none;
            border-radius: 3px;
            cursor: pointer;
            width: 100%;
        }
        button:hover {
            background: #0052a3;
        }
        button:disabled {
            background: #ccc;
            cursor: not-allowed;
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
        .hidden {
            display: none;
        }
    </style>
</head>
<body>
    <h1>Özel Kanal Demo</h1>
    
    <div id="auth-section" class="auth-section">
        <h2>Giriş</h2>
        <input type="text" id="username" placeholder="Kullanıcı Adı" value="ayse" />
        <input type="password" id="password" placeholder="Şifre" value="password123" />
        <button id="login-button">Giriş Yap</button>
    </div>
    
    <div id="chat-section" class="hidden">
        <div id="status" class="disconnected">Bağlantı Kesildi</div>
        <h2>Özel Mesajlar</h2>
        <div id="messages"></div>
        <div style="display: flex; gap: 10px;">
            <input type="text" id="message-input" placeholder="Mesajınızı yazın..." style="flex: 1;" />
            <button id="send-button" style="width: auto;">Gönder</button>
        </div>
    </div>
    
    <script src="https://js.pusher.com/8.2.0/pusher.min.js"></script>
    <script src="private-channel.js"></script>
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
import jwt from 'jsonwebtoken';

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

// Yapılandırma
const JWT_SECRET = 'your-secret-key-change-in-production';

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

// Mock kullanıcı veritabanı
interface User {
  id: string;
  username: string;
  password: string;
  name: string;
}

const users: User[] = [
  { id: 'user-1', username: 'ayse', password: 'password123', name: 'Ayşe Yılmaz' },
  { id: 'user-2', username: 'mehmet', password: 'password123', name: 'Mehmet Demir' },
  { id: 'user-3', username: 'fatma', password: 'password123', name: 'Fatma Kaya' },
];

// Giriş endpoint'i
app.post('/login', (req: Request, res: Response) => {
  const { username, password } = req.body;
  
  const user = users.find(u => u.username === username && u.password === password);
  
  if (!user) {
    return res.status(401).json({ error: 'Geçersiz kimlik bilgileri' });
  }
  
  // JWT token oluştur
  const token = jwt.sign(
    { userId: user.id, username: user.username },
    JWT_SECRET,
    { expiresIn: '24h' }
  );
  
  res.json({
    token,
    user: {
      id: user.id,
      username: user.username,
      name: user.name,
    },
  });
});

// JWT token'ı doğrulama middleware'i
function authenticateToken(req: Request, res: Response, next: Function) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'Token sağlanmadı' });
  }
  
  jwt.verify(token, JWT_SECRET, (err: any, decoded: any) => {
    if (err) {
      return res.status(403).json({ error: 'Geçersiz token' });
    }
    
    req.user = decoded;
    next();
  });
}

// Pusher kimlik doğrulama endpoint'i
app.post('/pusher/auth', authenticateToken, (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Eksik parametreler' });
  }
  
  // Kanal erişimini doğrula
  if (!channelName.startsWith('private-')) {
    return res.status(403).json({ error: 'Geçersiz kanal tipi' });
  }
  
  // Kullanıcının bu kanala erişimi olup olmadığını kontrol et
  // Örnek: private-user-{userId}
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Bu kanala erişim reddedildi' });
  }
  
  try {
    const authResponse = pusher.authorizeChannel(socketId, channelName);
    res.json(authResponse);
  } catch (error) {
    console.error('Kimlik doğrulama hatası:', error);
    res.status(500).json({ error: 'Kimlik doğrulama başarısız' });
  }
});

// Olay tetikleme endpoint'i (sunucu tarafı olaylar için)
app.post('/pusher/trigger', authenticateToken, async (req: Request, res: Response) => {
  const { channel, event, data } = req.body;
  
  try {
    await pusher.trigger(channel, event, data);
    res.json({ success: true });
  } catch (error) {
    console.error('Tetikleme hatası:', error);
    res.status(500).json({ error: 'Olay tetiklenemedi' });
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
const jwt = require('jsonwebtoken');

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

// Yapılandırma
const JWT_SECRET = 'your-secret-key-change-in-production';

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

// Mock kullanıcı veritabanı
const users = [
  { id: 'user-1', username: 'ayse', password: 'password123', name: 'Ayşe Yılmaz' },
  { id: 'user-2', username: 'mehmet', password: 'password123', name: 'Mehmet Demir' },
  { id: 'user-3', username: 'fatma', password: 'password123', name: 'Fatma Kaya' },
];

// Giriş endpoint'i
app.post('/login', (req, res) => {
  const { username, password } = req.body;
  
  const user = users.find(u => u.username === username && u.password === password);
  
  if (!user) {
    return res.status(401).json({ error: 'Geçersiz kimlik bilgileri' });
  }
  
  // JWT token oluştur
  const token = jwt.sign(
    { userId: user.id, username: user.username },
    JWT_SECRET,
    { expiresIn: '24h' }
  );
  
  res.json({
    token,
    user: {
      id: user.id,
      username: user.username,
      name: user.name,
    },
  });
});

// JWT token'ı doğrulama middleware'i
function authenticateToken(req, res, next) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'Token sağlanmadı' });
  }
  
  jwt.verify(token, JWT_SECRET, (err, decoded) => {
    if (err) {
      return res.status(403).json({ error: 'Geçersiz token' });
    }
    
    req.user = decoded;
    next();
  });
}

// Pusher kimlik doğrulama endpoint'i
app.post('/pusher/auth', authenticateToken, (req, res) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Eksik parametreler' });
  }
  
  // Kanal erişimini doğrula
  if (!channelName.startsWith('private-')) {
    return res.status(403).json({ error: 'Geçersiz kanal tipi' });
  }
  
  // Kullanıcının bu kanala erişimi olup olmadığını kontrol et
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Bu kanala erişim reddedildi' });
  }
  
  try {
    const authResponse = pusher.authorizeChannel(socketId, channelName);
    res.json(authResponse);
  } catch (error) {
    console.error('Kimlik doğrulama hatası:', error);
    res.status(500).json({ error: 'Kimlik doğrulama başarısız' });
  }
});

// Olay tetikleme endpoint'i
app.post('/pusher/trigger', authenticateToken, async (req, res) => {
  const { channel, event, data } = req.body;
  
  try {
    await pusher.trigger(channel, event, data);
    res.json({ success: true });
  } catch (error) {
    console.error('Tetikleme hatası:', error);
    res.status(500).json({ error: 'Olay tetiklenemedi' });
  }
});

// Sunucuyu başlat
const PORT = 3000;
app.listen(PORT, () => {
  console.log(`Kimlik doğrulama sunucusu http://localhost:${PORT} adresinde çalışıyor`);
});
```

## TypeScript Uygulaması

Bir `private-channel.ts` dosyası oluşturun:

```typescript
import Pusher from 'pusher-js';

// Yapılandırma
interface Message {
  user: string;
  text: string;
  timestamp: string;
}

interface AuthResponse {
  token: string;
  user: {
    id: string;
    username: string;
    name: string;
  };
}

// Durum
let authToken: string | null = null;
let currentUser: AuthResponse['user'] | null = null;
let pusher: Pusher | null = null;
let channel: any = null;

// DOM elementleri
const authSection = document.getElementById('auth-section') as HTMLDivElement;
const chatSection = document.getElementById('chat-section') as HTMLDivElement;
const usernameInput = document.getElementById('username') as HTMLInputElement;
const passwordInput = document.getElementById('password') as HTMLInputElement;
const loginButton = document.getElementById('login-button') as HTMLButtonElement;
const messagesDiv = document.getElementById('messages') as HTMLDivElement;
const messageInput = document.getElementById('message-input') as HTMLInputElement;
const sendButton = document.getElementById('send-button') as HTMLButtonElement;
const statusDiv = document.getElementById('status') as HTMLDivElement;

// Giriş fonksiyonu
async function login(): Promise<void> {
  const username = usernameInput.value.trim();
  const password = passwordInput.value.trim();
  
  if (!username || !password) {
    displayErrorMessage('Lütfen kullanıcı adı ve şifre girin');
    return;
  }
  
  try {
    loginButton.disabled = true;
    loginButton.textContent = 'Giriş yapılıyor...';
    
    const response = await fetch('http://localhost:3000/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });
    
    if (!response.ok) {
      throw new Error('Giriş başarısız');
    }
    
    const data: AuthResponse = await response.json();
    authToken = data.token;
    currentUser = data.user;
    
    // Kimlik doğrulama bölümünü gizle, chat'i göster
    authSection.classList.add('hidden');
    chatSection.classList.remove('hidden');
    
    // Pusher'ı başlat
    initializePusher();
    
  } catch (error) {
    console.error('Giriş hatası:', error);
    displayErrorMessage('Giriş başarısız. Lütfen kimlik bilgilerinizi kontrol edin.');
    loginButton.disabled = false;
    loginButton.textContent = 'Giriş Yap';
  }
}

// Pusher'ı kimlik doğrulama ile başlat
function initializePusher(): void {
  pusher = new Pusher('app-key', {
    wsHost: 'localhost',
    wsPort: 6001,
    forceTLS: false,
    enabledTransports: ['ws', 'wss'],
    cluster: 'mt1',
    disableStats: true,
    authEndpoint: 'http://localhost:3000/pusher/auth',
    auth: {
      headers: {
        'Authorization': `Bearer ${authToken}`,
      },
    },
  });
  
  // Bağlantı durumu işleyicileri
  pusher.connection.bind('connected', () => {
    console.log('soketi.rs\'ye bağlandı');
    statusDiv.textContent = 'Bağlandı';
    statusDiv.className = 'connected';
    
    // Özel kanala abone ol
    subscribeToPrivateChannel();
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
}

// Özel kanala abone ol
function subscribeToPrivateChannel(): void {
  if (!pusher || !currentUser) return;
  
  // Kullanıcıya özel özel kanala abone ol
  const channelName = `private-user-${currentUser.id}`;
  channel = pusher.subscribe(channelName);
  
  channel.bind('pusher:subscription_succeeded', () => {
    console.log(`${channelName} kanalına başarıyla abone olundu`);
    displaySystemMessage(`Özel kanalınıza bağlandınız`);
  });
  
  channel.bind('pusher:subscription_error', (status: any) => {
    console.error('Abonelik hatası:', status);
    displayErrorMessage('Özel kanala abone olunamadı');
  });
  
  // Mesajları dinle
  channel.bind('message', (data: Message) => {
    displayMessage(data);
  });
  
  // Bildirimleri dinle
  channel.bind('notification', (data: any) => {
    displaySystemMessage(`Bildirim: ${data.text}`);
  });
}

// Mesajı göster
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

// Hata mesajını göster
function displayErrorMessage(text: string): void {
  const messageEl = document.createElement('div');
  messageEl.className = 'message error-message';
  messageEl.textContent = `Hata: ${text}`;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// HTML'i kaçır
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Sunucu üzerinden mesaj gönder
async function sendMessage(): Promise<void> {
  const text = messageInput.value.trim();
  
  if (!text || !currentUser || !authToken) return;
  
  try {
    const response = await fetch('http://localhost:3000/pusher/trigger', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authToken}`,
      },
      body: JSON.stringify({
        channel: `private-user-${currentUser.id}`,
        event: 'message',
        data: {
          user: currentUser.name,
          text: text,
          timestamp: new Date().toISOString(),
        },
      }),
    });
    
    if (!response.ok) {
      throw new Error('Mesaj gönderilemedi');
    }
    
    messageInput.value = '';
    
  } catch (error) {
    console.error('Gönderme hatası:', error);
    displayErrorMessage('Mesaj gönderilemedi');
  }
}

// Olay dinleyicileri
loginButton.addEventListener('click', login);

passwordInput.addEventListener('keypress', (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    login();
  }
});

sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});
```

## JavaScript Uygulaması

Bir `private-channel.js` dosyası oluşturun:

```javascript
// Durum
let authToken = null;
let currentUser = null;
let pusher = null;
let channel = null;

// DOM elementleri
const authSection = document.getElementById('auth-section');
const chatSection = document.getElementById('chat-section');
const usernameInput = document.getElementById('username');
const passwordInput = document.getElementById('password');
const loginButton = document.getElementById('login-button');
const messagesDiv = document.getElementById('messages');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const statusDiv = document.getElementById('status');

// Giriş fonksiyonu
async function login() {
  const username = usernameInput.value.trim();
  const password = passwordInput.value.trim();
  
  if (!username || !password) {
    displayErrorMessage('Lütfen kullanıcı adı ve şifre girin');
    return;
  }
  
  try {
    loginButton.disabled = true;
    loginButton.textContent = 'Giriş yapılıyor...';
    
    const response = await fetch('http://localhost:3000/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });
    
    if (!response.ok) {
      throw new Error('Giriş başarısız');
    }
    
    const data = await response.json();
    authToken = data.token;
    currentUser = data.user;
    
    // Kimlik doğrulama bölümünü gizle, chat'i göster
    authSection.classList.add('hidden');
    chatSection.classList.remove('hidden');
    
    // Pusher'ı başlat
    initializePusher();
    
  } catch (error) {
    console.error('Giriş hatası:', error);
    displayErrorMessage('Giriş başarısız. Lütfen kimlik bilgilerinizi kontrol edin.');
    loginButton.disabled = false;
    loginButton.textContent = 'Giriş Yap';
  }
}

// Pusher'ı kimlik doğrulama ile başlat
function initializePusher() {
  pusher = new Pusher('app-key', {
    wsHost: 'localhost',
    wsPort: 6001,
    forceTLS: false,
    enabledTransports: ['ws', 'wss'],
    cluster: 'mt1',
    disableStats: true,
    authEndpoint: 'http://localhost:3000/pusher/auth',
    auth: {
      headers: {
        'Authorization': `Bearer ${authToken}`,
      },
    },
  });
  
  // Bağlantı durumu işleyicileri
  pusher.connection.bind('connected', () => {
    console.log('soketi.rs\'ye bağlandı');
    statusDiv.textContent = 'Bağlandı';
    statusDiv.className = 'connected';
    
    // Özel kanala abone ol
    subscribeToPrivateChannel();
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
}

// Özel kanala abone ol
function subscribeToPrivateChannel() {
  if (!pusher || !currentUser) return;
  
  // Kullanıcıya özel özel kanala abone ol
  const channelName = `private-user-${currentUser.id}`;
  channel = pusher.subscribe(channelName);
  
  channel.bind('pusher:subscription_succeeded', () => {
    console.log(`${channelName} kanalına başarıyla abone olundu`);
    displaySystemMessage(`Özel kanalınıza bağlandınız`);
  });
  
  channel.bind('pusher:subscription_error', (status) => {
    console.error('Abonelik hatası:', status);
    displayErrorMessage('Özel kanala abone olunamadı');
  });
  
  // Mesajları dinle
  channel.bind('message', (data) => {
    displayMessage(data);
  });
  
  // Bildirimleri dinle
  channel.bind('notification', (data) => {
    displaySystemMessage(`Bildirim: ${data.text}`);
  });
}

// Mesajı göster
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

// Hata mesajını göster
function displayErrorMessage(text) {
  const messageEl = document.createElement('div');
  messageEl.className = 'message error-message';
  messageEl.textContent = `Hata: ${text}`;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// HTML'i kaçır
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Sunucu üzerinden mesaj gönder
async function sendMessage() {
  const text = messageInput.value.trim();
  
  if (!text || !currentUser || !authToken) return;
  
  try {
    const response = await fetch('http://localhost:3000/pusher/trigger', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authToken}`,
      },
      body: JSON.stringify({
        channel: `private-user-${currentUser.id}`,
        event: 'message',
        data: {
          user: currentUser.name,
          text: text,
          timestamp: new Date().toISOString(),
        },
      }),
    });
    
    if (!response.ok) {
      throw new Error('Mesaj gönderilemedi');
    }
    
    messageInput.value = '';
    
  } catch (error) {
    console.error('Gönderme hatası:', error);
    displayErrorMessage('Mesaj gönderilemedi');
  }
}

// Olay dinleyicileri
loginButton.addEventListener('click', login);

passwordInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    login();
  }
});

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

### Tarayıcı Konsolu (Giriş Sonrası)

```
soketi.rs'ye bağlandı
private-user-1 kanalına başarıyla abone olundu
```

### Tarayıcı Görünümü

**Giriş Ekranı:**
```
Giriş
Kullanıcı Adı: [ayse]
Şifre: [••••••••••]
[Giriş Yap Butonu]
```

**Giriş Sonrası:**
```
Durum: Bağlandı

Özel Mesajlar
┌─────────────────────────────────────┐
│ Özel kanalınıza bağlandınız         │
│ Ayşe Yılmaz: Merhaba!               │
│ Bildirim: Yeni bir mesajınız var    │
└─────────────────────────────────────┘
[Mesajınızı yazın...] [Gönder]
```

## Güvenlik En İyi Uygulamaları

### 1. Production'da HTTPS Kullanın

```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'your-domain.com',
  forceTLS: true,  // Production'da her zaman TLS kullanın
  encrypted: true,
});
```

### 2. Uygun Token Yönetimi Uygulayın

```typescript
// Token'ı güvenli bir şekilde saklayın
localStorage.setItem('authToken', token);

// Çıkışta token'ı temizleyin
function logout() {
  localStorage.removeItem('authToken');
  if (pusher) {
    pusher.disconnect();
  }
}
```

### 3. Kanal Erişimini Doğrulayın

```typescript
// Sunucu tarafı: Kullanıcı izinlerini kontrol edin
app.post('/pusher/auth', authenticateToken, (req, res) => {
  const { channel_name } = req.body;
  const userId = req.user.userId;
  
  // Kullanıcının bu kanala erişimi olduğunu doğrulayın
  if (!hasChannelAccess(userId, channel_name)) {
    return res.status(403).json({ error: 'Erişim reddedildi' });
  }
  
  // Yetkilendir
  const authResponse = pusher.authorizeChannel(socketId, channel_name);
  res.json(authResponse);
});
```

### 4. Hız Sınırlama

```typescript
// Kötüye kullanımı önlemek için hız sınırlama ekleyin
const rateLimit = require('express-rate-limit');

const authLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 dakika
  max: 100, // Her IP için windowMs başına 100 istek
});

app.post('/pusher/auth', authLimiter, authenticateToken, (req, res) => {
  // ... kimlik doğrulama mantığı
});
```

### 5. Girdi Doğrulama

```typescript
// Tüm girdileri doğrulayın ve temizleyin
function validateChannelName(channelName: string): boolean {
  // Sadece alfanumerik, tire ve alt çizgiye izin verin
  return /^private-[a-zA-Z0-9_-]+$/.test(channelName);
}
```

## İlgili Dokümantasyon

- [Temel Chat Örneği](./temel-chat.md)
- [Presence Kanalları Örneği](./presence.md)
- [Kimlik Doğrulama Örneği](./kimlik-dogrulama.md)
- [API Referansı](../api-referans.md)
- [Güvenlik En İyi Uygulamaları](../baslangic.md#güvenlik)
