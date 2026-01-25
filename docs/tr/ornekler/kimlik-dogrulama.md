# Kimlik Doğrulama Örnekleri

> soketi.rs uygulamaları için eksiksiz kimlik doğrulama desenleri

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Koşullar](#ön-koşullar)
- [Kimlik Doğrulama Desenleri](#kimlik-doğrulama-desenleri)
- [JWT Tabanlı Kimlik Doğrulama](#jwt-tabanlı-kimlik-doğrulama)
- [Oturum Tabanlı Kimlik Doğrulama](#oturum-tabanlı-kimlik-doğrulama)
- [OAuth Entegrasyonu](#oauth-entegrasyonu)
- [En İyi Uygulamalar](#en-iyi-uygulamalar)

## Genel Bakış

Bu kılavuz, soketi.rs uygulamalarınızı güvenli hale getirmek için çeşitli kimlik doğrulama desenlerini kapsar. Şunları keşfedeceğiz:

- JWT (JSON Web Token) kimlik doğrulama
- Oturum tabanlı kimlik doğrulama
- OAuth 2.0 entegrasyonu
- Çok faktörlü kimlik doğrulama (MFA)
- Token yenileme stratejileri
- Güvenlik en iyi uygulamaları

## Ön Koşullar

- Node.js 18+ kurulu
- soketi.rs sunucusu çalışıyor
- Express.js veya benzer backend framework
- [Özel Kanallar](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/ozel-kanallar.md)'ı anlama
- Kimlik doğrulama kavramları hakkında temel bilgi

## Kimlik Doğrulama Desenleri

### Desen 1: JWT Kimlik Doğrulama

En uygun:
- Durumsuz kimlik doğrulama
- Mikroservis mimarisi
- Mobil uygulamalar
- API-first uygulamalar

### Desen 2: Oturum Kimlik Doğrulama

En uygun:
- Geleneksel web uygulamaları
- Sunucu tarafı render edilen uygulamalar
- Sunucu tarafı durum gerektiren uygulamalar

### Desen 3: OAuth 2.0

En uygun:
- Üçüncü taraf entegrasyonlar
- Sosyal giriş
- Kurumsal SSO
- Çok kiracılı uygulamalar

## JWT Tabanlı Kimlik Doğrulama

### TypeScript Uygulaması

Bir `auth-jwt.ts` dosyası oluşturun:

```typescript
import express, { Request, Response, NextFunction } from 'express';
import Pusher from 'pusher';
import jwt from 'jsonwebtoken';
import bcrypt from 'bcrypt';
import cors from 'cors';
import bodyParser from 'body-parser';

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());

// Yapılandırma
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key-change-in-production';
const JWT_REFRESH_SECRET = process.env.JWT_REFRESH_SECRET || 'your-refresh-secret';
const ACCESS_TOKEN_EXPIRY = '15m';
const REFRESH_TOKEN_EXPIRY = '7d';

// Pusher'ı başlat
const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID || 'app-id',
  key: process.env.PUSHER_KEY || 'app-key',
  secret: process.env.PUSHER_SECRET || 'app-secret',
  cluster: process.env.PUSHER_CLUSTER || 'mt1',
  host: process.env.PUSHER_HOST || 'localhost',
  port: parseInt(process.env.PUSHER_PORT || '6001'),
  useTLS: process.env.PUSHER_USE_TLS === 'true',
});

// Tipler
interface User {
  id: string;
  email: string;
  password: string;
  name: string;
  role: string;
}

interface JWTPayload {
  userId: string;
  email: string;
  role: string;
}

// Mock kullanıcı veritabanı (gerçek veritabanı ile değiştirin)
const users: User[] = [
  {
    id: 'user-1',
    email: 'ayse@example.com',
    password: '$2b$10$rBV2kHf7Yw3KxXqX5xXxXeX5xXxXxXxXxXxXxXxXxXxXxXxXxXxXx', // 'password123'
    name: 'Ayşe Yılmaz',
    role: 'admin',
  },
];

// Refresh token depolama (production'da Redis kullanın)
const refreshTokens = new Set<string>();

// Yardımcı: Token oluştur
function generateTokens(user: User): { accessToken: string; refreshToken: string } {
  const payload: JWTPayload = {
    userId: user.id,
    email: user.email,
    role: user.role,
  };
  
  const accessToken = jwt.sign(payload, JWT_SECRET, {
    expiresIn: ACCESS_TOKEN_EXPIRY,
  });
  
  const refreshToken = jwt.sign(payload, JWT_REFRESH_SECRET, {
    expiresIn: REFRESH_TOKEN_EXPIRY,
  });
  
  refreshTokens.add(refreshToken);
  
  return { accessToken, refreshToken };
}
```

// Middleware: Access token'ı doğrula
function authenticateToken(req: Request, res: Response, next: NextFunction) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'Token sağlanmadı' });
  }
  
  jwt.verify(token, JWT_SECRET, (err: any, decoded: any) => {
    if (err) {
      if (err.name === 'TokenExpiredError') {
        return res.status(401).json({ error: 'Token süresi doldu' });
      }
      return res.status(403).json({ error: 'Geçersiz token' });
    }
    
    req.user = decoded as JWTPayload;
    next();
  });
}

// Route: Kayıt
app.post('/auth/register', async (req: Request, res: Response) => {
  try {
    const { email, password, name } = req.body;
    
    // Girdiyi doğrula
    if (!email || !password || !name) {
      return res.status(400).json({ error: 'Gerekli alanlar eksik' });
    }
    
    // Kullanıcının var olup olmadığını kontrol et
    if (users.find(u => u.email === email)) {
      return res.status(409).json({ error: 'Kullanıcı zaten mevcut' });
    }
    
    // Şifreyi hashle
    const hashedPassword = await bcrypt.hash(password, 10);
    
    // Kullanıcı oluştur
    const user: User = {
      id: `user-${Date.now()}`,
      email,
      password: hashedPassword,
      name,
      role: 'user',
    };
    
    users.push(user);
    
    // Token oluştur
    const tokens = generateTokens(user);
    
    res.status(201).json({
      user: {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
      },
      ...tokens,
    });
  } catch (error) {
    console.error('Kayıt hatası:', error);
    res.status(500).json({ error: 'Kayıt başarısız' });
  }
});

// Route: Giriş
app.post('/auth/login', async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body;
    
    // Kullanıcıyı bul
    const user = users.find(u => u.email === email);
    if (!user) {
      return res.status(401).json({ error: 'Geçersiz kimlik bilgileri' });
    }
    
    // Şifreyi doğrula
    const validPassword = await bcrypt.compare(password, user.password);
    if (!validPassword) {
      return res.status(401).json({ error: 'Geçersiz kimlik bilgileri' });
    }
    
    // Token oluştur
    const tokens = generateTokens(user);
    
    res.json({
      user: {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
      },
      ...tokens,
    });
  } catch (error) {
    console.error('Giriş hatası:', error);
    res.status(500).json({ error: 'Giriş başarısız' });
  }
});
```

// Route: Token yenile
app.post('/auth/refresh', (req: Request, res: Response) => {
  const { refreshToken } = req.body;
  
  if (!refreshToken) {
    return res.status(401).json({ error: 'Refresh token sağlanmadı' });
  }
  
  if (!refreshTokens.has(refreshToken)) {
    return res.status(403).json({ error: 'Geçersiz refresh token' });
  }
  
  jwt.verify(refreshToken, JWT_REFRESH_SECRET, (err: any, decoded: any) => {
    if (err) {
      refreshTokens.delete(refreshToken);
      return res.status(403).json({ error: 'Geçersiz refresh token' });
    }
    
    const user = users.find(u => u.id === decoded.userId);
    if (!user) {
      return res.status(403).json({ error: 'Kullanıcı bulunamadı' });
    }
    
    // Yeni token oluştur
    refreshTokens.delete(refreshToken);
    const tokens = generateTokens(user);
    
    res.json(tokens);
  });
});

// Route: Çıkış
app.post('/auth/logout', authenticateToken, (req: Request, res: Response) => {
  const { refreshToken } = req.body;
  
  if (refreshToken) {
    refreshTokens.delete(refreshToken);
  }
  
  res.json({ message: 'Başarıyla çıkış yapıldı' });
});

// Route: Mevcut kullanıcıyı al
app.get('/auth/me', authenticateToken, (req: Request, res: Response) => {
  const user = users.find(u => u.id === req.user.userId);
  
  if (!user) {
    return res.status(404).json({ error: 'Kullanıcı bulunamadı' });
  }
  
  res.json({
    id: user.id,
    email: user.email,
    name: user.name,
    role: user.role,
  });
});

// Route: Pusher kimlik doğrulama
app.post('/pusher/auth', authenticateToken, (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Eksik parametreler' });
  }
  
  // Kullanıcı rolüne göre kanal erişimini doğrula
  if (channelName.startsWith('private-admin-') && user.role !== 'admin') {
    return res.status(403).json({ error: 'Admin erişimi gerekli' });
  }
  
  // Kullanıcıya özel kanalları doğrula
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Erişim reddedildi' });
  }
  
  try {
    let authResponse;
    
    // Presence kanalı
    if (channelName.startsWith('presence-')) {
      const presenceData = {
        user_id: user.userId,
        user_info: {
          email: user.email,
          role: user.role,
        },
      };
      authResponse = pusher.authorizeChannel(socketId, channelName, presenceData);
    } else {
      // Özel kanal
      authResponse = pusher.authorizeChannel(socketId, channelName);
    }
    
    res.json(authResponse);
  } catch (error) {
    console.error('Pusher kimlik doğrulama hatası:', error);
    res.status(500).json({ error: 'Kimlik doğrulama başarısız' });
  }
});

// Sunucuyu başlat
const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Kimlik doğrulama sunucusu http://localhost:${PORT} adresinde çalışıyor`);
});
```

## En İyi Uygulamalar

### 1. Güvenli Token Depolama

**İstemci tarafı (TypeScript):**
```typescript
class TokenManager {
  private static readonly ACCESS_TOKEN_KEY = 'access_token';
  private static readonly REFRESH_TOKEN_KEY = 'refresh_token';
  
  static setTokens(accessToken: string, refreshToken: string): void {
    // Production'da httpOnly cookie kullanın
    localStorage.setItem(this.ACCESS_TOKEN_KEY, accessToken);
    localStorage.setItem(this.REFRESH_TOKEN_KEY, refreshToken);
  }
  
  static getAccessToken(): string | null {
    return localStorage.getItem(this.ACCESS_TOKEN_KEY);
  }
  
  static getRefreshToken(): string | null {
    return localStorage.getItem(this.REFRESH_TOKEN_KEY);
  }
  
  static clearTokens(): void {
    localStorage.removeItem(this.ACCESS_TOKEN_KEY);
    localStorage.removeItem(this.REFRESH_TOKEN_KEY);
  }
}
```

### 2. Otomatik Token Yenileme

```typescript
class AuthService {
  private refreshTimeout?: NodeJS.Timeout;
  
  async login(email: string, password: string): Promise<void> {
    const response = await fetch('http://localhost:3000/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
    
    const data = await response.json();
    TokenManager.setTokens(data.accessToken, data.refreshToken);
    
    this.scheduleTokenRefresh();
  }
  
  private scheduleTokenRefresh(): void {
    // Süre dolmadan 1 dakika önce yenile (15 dakikalık token için 14 dakika)
    const refreshTime = 14 * 60 * 1000;
    
    this.refreshTimeout = setTimeout(async () => {
      await this.refreshToken();
    }, refreshTime);
  }
  
  private async refreshToken(): Promise<void> {
    try {
      const refreshToken = TokenManager.getRefreshToken();
      
      const response = await fetch('http://localhost:3000/auth/refresh', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refreshToken }),
      });
      
      const data = await response.json();
      TokenManager.setTokens(data.accessToken, data.refreshToken);
      
      this.scheduleTokenRefresh();
    } catch (error) {
      console.error('Token yenileme başarısız:', error);
      this.logout();
    }
  }
  
  logout(): void {
    if (this.refreshTimeout) {
      clearTimeout(this.refreshTimeout);
    }
    TokenManager.clearTokens();
  }
}
```

### 3. Güvenli Pusher İstemci Yapılandırması

```typescript
function createAuthenticatedPusherClient(): Pusher {
  return new Pusher('app-key', {
    wsHost: 'localhost',
    wsPort: 6001,
    forceTLS: false,
    enabledTransports: ['ws', 'wss'],
    cluster: 'mt1',
    authEndpoint: 'http://localhost:3000/pusher/auth',
    auth: {
      headers: {
        'Authorization': `Bearer ${TokenManager.getAccessToken()}`,
      },
    },
  });
}
```

### 4. Rol Tabanlı Erişim Kontrolü

```typescript
// Sunucu tarafı
enum Role {
  USER = 'user',
  ADMIN = 'admin',
  MODERATOR = 'moderator',
}

interface ChannelPermissions {
  [key: string]: Role[];
}

const channelPermissions: ChannelPermissions = {
  'private-admin-': [Role.ADMIN],
  'private-moderator-': [Role.ADMIN, Role.MODERATOR],
  'private-user-': [Role.USER, Role.ADMIN, Role.MODERATOR],
};

function hasChannelAccess(channelName: string, userRole: Role): boolean {
  for (const [prefix, allowedRoles] of Object.entries(channelPermissions)) {
    if (channelName.startsWith(prefix)) {
      return allowedRoles.includes(userRole);
    }
  }
  return false;
}

app.post('/pusher/auth', authenticateToken, (req, res) => {
  const { channel_name } = req.body;
  const userRole = req.user.role as Role;
  
  if (!hasChannelAccess(channel_name, userRole)) {
    return res.status(403).json({ error: 'Yetersiz izinler' });
  }
  
  // Kanalı yetkilendir...
});
```

### 5. Hız Sınırlama

```typescript
import rateLimit from 'express-rate-limit';

// Giriş hız sınırlayıcı
const loginLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 dakika
  max: 5, // 5 deneme
  message: 'Çok fazla giriş denemesi, lütfen daha sonra tekrar deneyin',
});

app.post('/auth/login', loginLimiter, async (req, res) => {
  // Giriş mantığı...
});

// Pusher kimlik doğrulama hız sınırlayıcı
const pusherAuthLimiter = rateLimit({
  windowMs: 1 * 60 * 1000, // 1 dakika
  max: 30, // 30 istek
});

app.post('/pusher/auth', pusherAuthLimiter, authenticateToken, (req, res) => {
  // Kimlik doğrulama mantığı...
});
```

### 6. Güvenlik Başlıkları

```typescript
import helmet from 'helmet';

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      connectSrc: ["'self'", 'ws://localhost:6001', 'wss://localhost:6001'],
    },
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true,
  },
}));
```

## Beklenen Çıktı

### JWT Kimlik Doğrulama Akışı

```
1. Kullanıcı kayıt olur/giriş yapar
   POST /auth/login
   Yanıt: { accessToken, refreshToken, user }

2. İstemci token'ları saklar
   localStorage.setItem('access_token', accessToken)

3. İstemci kimlik doğrulama ile Pusher'a bağlanır
   Authorization: Bearer <accessToken>

4. Token süresi dolar, istemci yeniler
   POST /auth/refresh
   Yanıt: { accessToken, refreshToken }

5. Kullanıcı çıkış yapar
   POST /auth/logout
   Token'lar temizlenir
```

### Konsol Çıktısı

```
Kimlik doğrulama sunucusu http://localhost:3000 adresinde çalışıyor
Kullanıcı giriş yaptı: ayse@example.com
Pusher kanalı yetkilendirildi: private-user-1
Kullanıcı için token yenilendi: ayse@example.com
Kullanıcı çıkış yaptı: ayse@example.com
```

## İlgili Dokümantasyon

- [Temel Chat Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/temel-chat.md)
- [Özel Kanallar Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/ozel-kanallar.md)
- [Presence Kanalları Örneği](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/ornekler/presence.md)
- [API Referansı](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/api-referans.md)
- [Güvenlik En İyi Uygulamaları](../baslangic.md#güvenlik)
- [Yapılandırma Kılavuzu](https://github.com/ferdiunal/soketi.rs/blob/main/docs/tr/yapilandirma.md)
