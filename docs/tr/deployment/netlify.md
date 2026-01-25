# soketi.rs'yi Netlify'da Dağıtma

> Netlify platformu ile soketi.rs WebSocket sunucusunu dağıtmak için eksiksiz kılavuz

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Gereksinimler](#ön-gereksinimler)
- [Adım Adım Dağıtım](#adım-adım-dağıtım)
- [Ortam Değişkenleri](#ortam-değişkenleri)
- [Build Ayarları](#build-ayarları)
- [WebSocket Yapılandırması](#websocket-yapılandırması)
- [SSL/TLS Kurulumu](#ssltls-kurulumu)
- [İzleme ve Loglama](#izleme-ve-loglama)
- [Maliyet Tahmini](#maliyet-tahmini)
- [Ölçeklendirme Önerileri](#ölçeklendirme-önerileri)
- [Sorun Giderme](#sorun-giderme)

## Genel Bakış

Netlify, sürekli dağıtım, serverless fonksiyonlar ve edge computing yetenekleri ile modern bir web projesi platformudur. Vercel'e benzer şekilde, Netlify statik siteler ve serverless fonksiyonlar için optimize edilmiştir, bu nedenle soketi.rs WebSocket sunucusu ayrı olarak dağıtılmalı ve frontend'iniz Netlify'da barındırılmalıdır.

**Önerilen Mimari:**
- Frontend (Next.js chat uygulaması): Netlify'da dağıtılır
- soketi.rs WebSocket sunucusu: Bir konteyner platformunda dağıtılır (Railway, Fly.io veya Render)
- Bağlantı: Frontend, WebSocket üzerinden soketi.rs'ye bağlanır

## Ön Gereksinimler

Dağıtmadan önce, aşağıdakilere sahip olduğunuzdan emin olun:

- Bir Netlify hesabı ([buradan kaydolun](https://app.netlify.com/signup))
- Netlify CLI kurulu: `npm install -g netlify-cli`
- Projenizin bulunduğu Git deposu
- soketi.rs sunucusu için ayrı bir barındırma çözümü

## Adım Adım Dağıtım

### Adım 1: soketi.rs Sunucusunu Dağıtın

soketi.rs'yi uzun süreli WebSocket bağlantılarını destekleyen bir konteyner platformunda dağıtın:

**Seçenek A: Render Kullanarak**

```bash
# soketi.rs projenizde render.yaml oluşturun
cat > render.yaml << EOF
services:
  - type: web
    name: soketi-server
    env: docker
    dockerfilePath: ./Dockerfile
    envVars:
      - key: SOKETI_DEFAULT_APP_ID
        value: app-id
      - key: SOKETI_DEFAULT_APP_KEY
        value: app-key
      - key: SOKETI_DEFAULT_APP_SECRET
        generateValue: true
      - key: SOKETI_HOST
        value: 0.0.0.0
      - key: SOKETI_PORT
        value: 6001
EOF

# Render'a dağıtın
# GitHub'a push edin ve Render dashboard üzerinden bağlayın
```

**Seçenek B: Railway Kullanarak**

```bash
# Railway CLI'yi yükleyin
npm install -g @railway/cli

# Giriş yapın ve dağıtın
railway login
railway init
railway up
```

**Seçenek C: Fly.io Kullanarak**

```bash
# Fly CLI'yi yükleyin
curl -L https://fly.io/install.sh | sh

# Dağıtın
fly auth login
fly launch
fly deploy
```

### Adım 2: Frontend'i Netlify'da Dağıtın

**Yöntem 1: Netlify CLI Kullanarak**

```bash
# Netlify'a giriş yapın
netlify login

# Next.js projenize gidin
cd next-chat-app

# Netlify sitesini başlatın
netlify init

# Dağıtın
netlify deploy --prod
```

**Yöntem 2: Netlify Dashboard Kullanarak**

1. [Netlify Dashboard](https://app.netlify.com)'a gidin
2. "Add new site" → "Import an existing project"e tıklayın
3. Git sağlayıcınızı bağlayın (GitHub, GitLab, Bitbucket)
4. Deponuzu seçin
5. Build ayarlarını yapılandırın:
   - **Build command**: `npm run build`
   - **Publish directory**: `.next`
   - **Functions directory**: `.netlify/functions`

### Adım 3: netlify.toml Yapılandırın

Proje kök dizininizde `netlify.toml` oluşturun:

```toml
[build]
  command = "npm run build"
  publish = ".next"

[build.environment]
  NEXT_TELEMETRY_DISABLED = "1"
  NODE_VERSION = "20"

[[plugins]]
  package = "@netlify/plugin-nextjs"

[[headers]]
  for = "/*"
  [headers.values]
    X-Frame-Options = "DENY"
    X-Content-Type-Options = "nosniff"
    X-XSS-Protection = "1; mode=block"
    Referrer-Policy = "strict-origin-when-cross-origin"

[[headers]]
  for = "/api/*"
  [headers.values]
    Access-Control-Allow-Origin = "*"
    Access-Control-Allow-Methods = "GET, POST, PUT, DELETE, OPTIONS"
    Access-Control-Allow-Headers = "Content-Type, Authorization"

[[redirects]]
  from = "/api/*"
  to = "/.netlify/functions/:splat"
  status = 200

[functions]
  directory = ".netlify/functions"
  node_bundler = "esbuild"

[dev]
  command = "npm run dev"
  port = 3000
  targetPort = 3000
  autoLaunch = false
```

### Adım 4: Frontend'i soketi.rs'ye Bağlayın

Netlify'da ortam değişkenlerini yapılandırın:

```bash
# Netlify CLI kullanarak
netlify env:set NEXT_PUBLIC_PUSHER_HOST your-soketi-host.railway.app
netlify env:set NEXT_PUBLIC_PUSHER_PORT 443
netlify env:set NEXT_PUBLIC_PUSHER_KEY your-app-key
netlify env:set PUSHER_SECRET your-app-secret

# Veya Netlify Dashboard kullanın:
# Site settings → Environment variables → Add variables
```

## Ortam Değişkenleri

### Frontend Ortam Değişkenleri (Netlify)

Bunları Netlify Dashboard'da veya CLI üzerinden yapılandırın:

```bash
# Pusher Client Yapılandırması
NEXT_PUBLIC_PUSHER_KEY=your-app-key
NEXT_PUBLIC_PUSHER_HOST=your-soketi-host.render.com
NEXT_PUBLIC_PUSHER_PORT=443
NEXT_PUBLIC_PUSHER_CLUSTER=mt1
NEXT_PUBLIC_PUSHER_FORCE_TLS=true
NEXT_PUBLIC_PUSHER_ENCRYPTED=true

# Pusher Server Yapılandırması
PUSHER_APP_ID=your-app-id
PUSHER_SECRET=your-app-secret
PUSHER_HOST=your-soketi-host.render.com
PUSHER_PORT=443
PUSHER_USE_TLS=true

# Veritabanı (Better Auth kullanıyorsanız)
DATABASE_URL=postgresql://user:password@host:5432/database

# Better Auth
AUTH_SECRET=your-random-secret-key
AUTH_URL=https://your-site.netlify.app

# Netlify'ye özel
NETLIFY_SITE_ID=your-site-id
NETLIFY_AUTH_TOKEN=your-auth-token
```

### soketi.rs Sunucu Ortam Değişkenleri

Bunları konteyner platformunuzda yapılandırın:

```bash
# Uygulama Yapılandırması
SOKETI_DEFAULT_APP_ID=your-app-id
SOKETI_DEFAULT_APP_KEY=your-app-key
SOKETI_DEFAULT_APP_SECRET=your-app-secret

# Sunucu Yapılandırması
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001

# CORS Yapılandırması
SOKETI_CORS_ALLOWED_ORIGINS=https://your-site.netlify.app,https://deploy-preview-*--your-site.netlify.app

# SSL Yapılandırması (gerekirse)
SOKETI_SSL_CERT=/path/to/cert.pem
SOKETI_SSL_KEY=/path/to/key.pem
```

## Build Ayarları

### Netlify Build Yapılandırması

**netlify.toml** (detaylı yapılandırma):

```toml
[build]
  command = "npm run build"
  publish = ".next"
  functions = ".netlify/functions"

[build.environment]
  # Node.js versiyonu
  NODE_VERSION = "20"
  
  # Telemetri'yi devre dışı bırak
  NEXT_TELEMETRY_DISABLED = "1"
  
  # Build optimizasyonları
  NODE_OPTIONS = "--max-old-space-size=4096"

# Netlify için Next.js eklentisi
[[plugins]]
  package = "@netlify/plugin-nextjs"

# Güvenlik header'ları
[[headers]]
  for = "/*"
  [headers.values]
    Strict-Transport-Security = "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options = "DENY"
    X-Content-Type-Options = "nosniff"
    X-XSS-Protection = "1; mode=block"
    Referrer-Policy = "strict-origin-when-cross-origin"
    Permissions-Policy = "camera=(), microphone=(), geolocation=()"

# API route'ları için header'lar
[[headers]]
  for = "/api/*"
  [headers.values]
    Access-Control-Allow-Origin = "*"
    Access-Control-Allow-Methods = "GET, POST, PUT, DELETE, OPTIONS"
    Access-Control-Allow-Headers = "Content-Type, Authorization"
    Cache-Control = "no-cache, no-store, must-revalidate"

# Statik varlıklar için önbellekleme
[[headers]]
  for = "/_next/static/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"

[[headers]]
  for = "/static/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"

# API route'ları için yönlendirmeler
[[redirects]]
  from = "/api/*"
  to = "/.netlify/functions/:splat"
  status = 200
  force = true

# SPA fallback
[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
  conditions = {Role = ["admin"]}

# Functions yapılandırması
[functions]
  directory = ".netlify/functions"
  node_bundler = "esbuild"
  included_files = ["prisma/**/*"]

# Geliştirme ayarları
[dev]
  command = "npm run dev"
  port = 3000
  targetPort = 3000
  autoLaunch = false
  framework = "#custom"

# Context'e özel ayarlar
[context.production]
  environment = { NODE_ENV = "production" }

[context.deploy-preview]
  environment = { NODE_ENV = "preview" }

[context.branch-deploy]
  environment = { NODE_ENV = "development" }
```

### Netlify için Next.js Yapılandırması

`next.config.js` dosyasını güncelleyin:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  
  // Netlify'ye özel çıktı
  output: 'standalone',
  
  // Görsel optimizasyonu
  images: {
    domains: ['localhost'],
    formats: ['image/avif', 'image/webp'],
    // Netlify Image CDN
    loader: 'custom',
    loaderFile: './lib/netlify-image-loader.js',
  },
  
  // Sıkıştırma
  compress: true,
  
  // Netlify için trailing slash
  trailingSlash: false,
  
  // Ortam değişkenleri
  env: {
    NEXT_PUBLIC_PUSHER_KEY: process.env.NEXT_PUBLIC_PUSHER_KEY,
    NEXT_PUBLIC_PUSHER_HOST: process.env.NEXT_PUBLIC_PUSHER_HOST,
    NEXT_PUBLIC_PUSHER_PORT: process.env.NEXT_PUBLIC_PUSHER_PORT,
  },
  
  // Webpack yapılandırması
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
      };
    }
    return config;
  },
  
  // Güvenlik header'ları
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
          {
            key: 'X-DNS-Prefetch-Control',
            value: 'on',
          },
          {
            key: 'Strict-Transport-Security',
            value: 'max-age=63072000; includeSubDomains; preload',
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
          {
            key: 'X-Frame-Options',
            value: 'DENY',
          },
        ],
      },
    ];
  },
};

module.exports = nextConfig;
```

### Package.json Script'leri

Netlify'ye özel script'ler ekleyin:

```json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint",
    "netlify:dev": "netlify dev",
    "netlify:build": "next build",
    "netlify:deploy": "netlify deploy --prod"
  }
}
```

## WebSocket Yapılandırması

### İstemci Tarafı WebSocket Kurulumu

Netlify dağıtımı için Pusher client'ı yapılandırın (`lib/pusher.ts`):

```typescript
import Pusher from 'pusher-js';

// Geliştirmede loglama'yı etkinleştir
if (process.env.NODE_ENV === 'development') {
  Pusher.logToConsole = true;
}

export const pusherClient = new Pusher(process.env.NEXT_PUBLIC_PUSHER_KEY!, {
  wsHost: process.env.NEXT_PUBLIC_PUSHER_HOST,
  wsPort: parseInt(process.env.NEXT_PUBLIC_PUSHER_PORT || '443'),
  wssPort: parseInt(process.env.NEXT_PUBLIC_PUSHER_PORT || '443'),
  forceTLS: true,
  encrypted: true,
  disableStats: true,
  enabledTransports: ['ws', 'wss'],
  cluster: process.env.NEXT_PUBLIC_PUSHER_CLUSTER || 'mt1',
  authEndpoint: '/api/pusher/auth',
  
  // Bağlantı timeout ayarları
  activityTimeout: 30000,
  pongTimeout: 10000,
  unavailableTimeout: 10000,
});

// Bağlantı durumu işleyicileri
pusherClient.connection.bind('connected', () => {
  console.log('✅ soketi.rs\'ye bağlandı');
});

pusherClient.connection.bind('disconnected', () => {
  console.log('❌ soketi.rs bağlantısı kesildi');
});

pusherClient.connection.bind('error', (err: any) => {
  console.error('❌ Bağlantı hatası:', err);
});

pusherClient.connection.bind('state_change', (states: any) => {
  console.log('🔄 Bağlantı durumu:', states.current);
});

export default pusherClient;
```

### Pusher Auth için Netlify Functions

`.netlify/functions/pusher-auth.ts` oluşturun:

```typescript
import { Handler } from '@netlify/functions';
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID!,
  key: process.env.NEXT_PUBLIC_PUSHER_KEY!,
  secret: process.env.PUSHER_SECRET!,
  cluster: process.env.NEXT_PUBLIC_PUSHER_CLUSTER || 'mt1',
  host: process.env.PUSHER_HOST,
  port: process.env.PUSHER_PORT,
  useTLS: true,
});

export const handler: Handler = async (event) => {
  // Sadece POST isteklerine izin ver
  if (event.httpMethod !== 'POST') {
    return {
      statusCode: 405,
      body: JSON.stringify({ error: 'Method not allowed' }),
    };
  }

  try {
    const body = event.body || '';
    const params = new URLSearchParams(body);
    const socketId = params.get('socket_id');
    const channelName = params.get('channel_name');

    if (!socketId || !channelName) {
      return {
        statusCode: 400,
        body: JSON.stringify({ error: 'Eksik parametreler' }),
      };
    }

    // Session'dan kullanıcıyı al (kendi auth mantığınızı uygulayın)
    // const user = await getUser(event);

    // Presence channel
    if (channelName.startsWith('presence-')) {
      const presenceData = {
        user_id: 'user-123', // Gerçek kullanıcı ID'si ile değiştirin
        user_info: {
          name: 'Kullanıcı Adı',
          email: 'user@example.com',
        },
      };
      
      const authResponse = pusher.authorizeChannel(socketId, channelName, presenceData);
      
      return {
        statusCode: 200,
        body: JSON.stringify(authResponse),
      };
    }

    // Private channel
    if (channelName.startsWith('private-')) {
      const authResponse = pusher.authorizeChannel(socketId, channelName);
      
      return {
        statusCode: 200,
        body: JSON.stringify(authResponse),
      };
    }

    return {
      statusCode: 400,
      body: JSON.stringify({ error: 'Geçersiz kanal' }),
    };
  } catch (error) {
    console.error('Auth hatası:', error);
    return {
      statusCode: 500,
      body: JSON.stringify({ error: 'Sunucu hatası' }),
    };
  }
};
```

## SSL/TLS Kurulumu

### Netlify ile Otomatik SSL

Netlify, tüm siteler için otomatik SSL sertifikaları sağlar:

- **Netlify domain'leri**: Varsayılan olarak SSL etkin (*.netlify.app)
- **Özel domain'ler**: Let's Encrypt üzerinden otomatik SSL sağlama
- **Sertifika yenileme**: Otomatik, manuel müdahale gerekmez
- **HTTPS yönlendirme**: HTTP'den HTTPS'e otomatik yönlendirme

### Özel Domain SSL Kurulumu

1. **Netlify Dashboard'da özel domain ekleyin:**
   - Site settings → Domain management → Add custom domain
   - DNS yapılandırma talimatlarını takip edin

2. **SSL sertifikasını doğrulayın:**
   - Netlify otomatik olarak SSL sertifikası sağlar
   - Genellikle 1-2 dakika sürer
   - Domain management bölümünden durumu kontrol edin

3. **HTTPS'i zorla:**

`netlify.toml` dosyasına ekleyin:

```toml
[[redirects]]
  from = "http://*"
  to = "https://:splat"
  status = 301
  force = true
```

### soketi.rs SSL Yapılandırması

soketi.rs barındırma platformunuzda SSL'i yapılandırın:

**Render:**
- Tüm servisler için otomatik SSL
- Otomatik SSL ile özel domain'ler desteklenir

**Railway:**
- Railway domain'leri için otomatik SSL
- Özel domain SSL mevcut

**Fly.io:**
```bash
# SSL ile özel domain ekleyin
fly certs add your-domain.com

# Sertifika durumunu kontrol edin
fly certs show your-domain.com
```

## İzleme ve Loglama

### Netlify İzleme

**1. Gerçek Zamanlı Loglar:**

```bash
# Function loglarını görüntüleyin
netlify functions:log

# Build loglarını görüntüleyin
netlify watch

# Site loglarını görüntüleyin
netlify logs
```

**2. Netlify Dashboard:**
   - Sitenize gidin
   - **Deploys**: Dağıtım geçmişini ve logları görüntüleyin
   - **Functions**: Function çağrılarını ve hatalarını izleyin
   - **Analytics**: Sayfa görüntülemelerini ve performansı takip edin

**3. Netlify Analytics:**

Site ayarlarında etkinleştirin:
- Gerçek kullanıcı metrikleri
- Sayfa görüntülemeleri ve benzersiz ziyaretçiler
- En çok görüntülenen sayfalar ve kaynaklar
- JavaScript gerekmez (sunucu tarafı analytics)

### Function İzleme

Netlify Functions'ı izleyin:

```typescript
// Function'lara loglama ekleyin
export const handler: Handler = async (event) => {
  console.log('Function çağrıldı:', {
    path: event.path,
    method: event.httpMethod,
    timestamp: new Date().toISOString(),
  });
  
  // Function mantığınız
  
  return {
    statusCode: 200,
    body: JSON.stringify({ success: true }),
  };
};
```

### soketi.rs İzleme

**Health Check:**

```bash
# Sunucu sağlığını kontrol edin
curl https://your-soketi-host.render.com/health
```

**Loglama:**

```bash
# Debug loglama'yı etkinleştirin
SOKETI_DEBUG=true
SOKETI_LOG_LEVEL=debug
```

### Harici İzleme Araçları

Üçüncü taraf izleme entegre edin:

**Hata Takibi için Sentry:**

```typescript
// lib/sentry.ts
import * as Sentry from '@sentry/nextjs';

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.NETLIFY_CONTEXT || 'development',
  tracesSampleRate: 1.0,
});
```

**Session Replay için LogRocket:**

```typescript
// lib/logrocket.ts
import LogRocket from 'logrocket';

if (typeof window !== 'undefined' && process.env.NODE_ENV === 'production') {
  LogRocket.init(process.env.NEXT_PUBLIC_LOGROCKET_ID!);
}
```

## Maliyet Tahmini

### Netlify Fiyatlandırması

**Starter Planı (Ücretsiz):**
- Ayda 100 GB bant genişliği
- Ayda 300 build dakikası
- Otomatik SSL
- Deploy preview'lar
- **En uygun**: Kişisel projeler ve geliştirme

**Pro Planı ($19/ay):**
- Ayda 400 GB bant genişliği
- Ayda 1,000 build dakikası
- Şifre koruması
- Analytics
- **En uygun**: Profesyonel siteler

**Business Planı ($99/ay):**
- Ayda 1 TB bant genişliği
- Sınırsız build dakikası
- SSO/SAML
- Öncelikli destek
- **En uygun**: Takım projeleri

**Enterprise Planı (Özel):**
- Özel bant genişliği ve build'ler
- SLA garantileri
- Özel destek
- **En uygun**: Büyük organizasyonlar

### soketi.rs Barındırma Maliyetleri

**Render:**
- **Ücretsiz katman**: Ayda 750 saat (hareketsizlikten sonra uyur)
- **Starter**: $7/ay (her zaman açık, 512 MB RAM)
- **Standard**: $25/ay (2 GB RAM)

**Railway:**
- **Starter**: $5/ay (512 MB RAM)
- **Developer**: $10/ay (1 GB RAM)

**Fly.io:**
- **Ücretsiz katman**: 3 shared VM, 160 GB bant genişliği
- **Ücretli**: Kaynaklara bağlı olarak ~$5-20/ay

### Toplam Aylık Maliyet Tahmini

**Küçük Proje:**
- Netlify Starter: $0
- Render Free: $0
- **Toplam**: $0/ay (sınırlamalarla)

**Production Uygulaması:**
- Netlify Pro: $19
- Render Starter: $7
- Veritabanı (opsiyonel): $7
- **Toplam**: $33/ay

**Business Uygulaması:**
- Netlify Business: $99
- Render Standard: $25
- Veritabanı: $15
- İzleme: $10
- **Toplam**: $149/ay

**Enterprise:**
- Netlify Enterprise: Özel ($200+)
- Özel sunucular: $100-500+
- **Toplam**: $300-1000+/ay

## Ölçeklendirme Önerileri

### Yatay Ölçeklendirme

**soketi.rs Cluster:**

Redis adapter ile birden fazla instance dağıtın:

```bash
# soketi.rs ortam değişkenleri
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=your-redis-host.com
SOKETI_REDIS_PORT=6379
SOKETI_REDIS_PASSWORD=your-redis-password
```

**Load Balancing:**

Barındırma platformunuzun load balancer'ını kullanın veya Nginx/Caddy arkasında dağıtın.

### Dikey Ölçeklendirme

**Kaynakları Artırın:**

Render:
```bash
# Render dashboard'da planı yükseltin
# Settings → Instance Type → Daha yüksek katmanı seçin
```

Railway:
```bash
# Railway dashboard'da yükseltin
# Settings → Plan → Daha yüksek katmanı seçin
```

### Netlify Ölçeklendirme

Netlify otomatik olarak ölçeklenir:
- **Edge Network**: 100+ lokasyonlu global CDN
- **Functions**: Otomatik ölçeklenen serverless fonksiyonlar
- **Anında Önbellek Geçersizleştirme**: Hızlı içerik güncellemeleri
- **Yapılandırma gerekmez**: Trafikle otomatik ölçeklenir

### Performans Optimizasyonu

**1. soketi.rs için Redis'i etkinleştirin:**

```bash
# Redis ile yatay ölçeklendirme
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=redis-host
SOKETI_REDIS_PORT=6379
```

**2. Netlify Functions'ı optimize edin:**

```typescript
// Daha iyi performans için edge functions kullanın
export const config = {
  path: '/api/pusher/auth',
  cache: 'manual',
};
```

**3. Önbellekleme Uygulayın:**

```toml
# netlify.toml
[[headers]]
  for = "/_next/static/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
```

**4. Netlify Image CDN Kullanın:**

```javascript
// next.config.js
module.exports = {
  images: {
    loader: 'custom',
    loaderFile: './lib/netlify-image-loader.js',
  },
};
```

## Sorun Giderme

### Yaygın Sorunlar

**1. Build Hataları**

```
Error: Command failed with exit code 1
```

**Çözüm:**
```bash
# Netlify dashboard'da build loglarını kontrol edin
# Önbelleği temizleyin ve tekrar deneyin
netlify build --clear-cache

# Build'i yerel olarak test edin
npm run build
```

**2. Function Timeout**

```
Error: Function execution timed out
```

**Çözüm:**
```toml
# netlify.toml'da function timeout'u artırın
[functions]
  timeout = 30
```

**3. WebSocket Bağlantısı Başarısız**

```
Error: WebSocket connection failed
```

**Çözüm:**
- soketi.rs sunucusunun çalıştığını doğrulayın
- CORS yapılandırmasını kontrol edin
- SSL sertifikalarının geçerli olduğundan emin olun
- Ortam değişkenlerinin doğru ayarlandığını doğrulayın

**4. Ortam Değişkenleri Yüklenmiyor**

**Çözüm:**
```bash
# Ortam değişkenlerini yerel olarak çekin
netlify env:list

# Eksik değişkenleri ayarlayın
netlify env:set VARIABLE_NAME value

# Yeniden dağıtın
netlify deploy --prod
```

**5. CORS Hataları**

```
Access-Control-Allow-Origin error
```

**Çözüm:**
```bash
# soketi.rs CORS ayarlarını güncelleyin
SOKETI_CORS_ALLOWED_ORIGINS=https://your-site.netlify.app,https://deploy-preview-*--your-site.netlify.app
```

**6. Deploy Preview Sorunları**

**Çözüm:**
```toml
# netlify.toml'da deploy preview'ları yapılandırın
[context.deploy-preview]
  environment = { NODE_ENV = "preview" }
  
[context.deploy-preview.environment]
  NEXT_PUBLIC_PUSHER_HOST = "preview-soketi-host.com"
```

### Debug Modu

Debug loglama'yı etkinleştirin:

```typescript
// lib/pusher.ts
if (process.env.NODE_ENV !== 'production') {
  Pusher.logToConsole = true;
}

const pusher = new Pusher(key, {
  enabledTransports: ['ws', 'wss'],
  // ... diğer yapılandırma
});
```

### Health Check'ler

Dağıtımınızı izleyin:

```bash
# Netlify sitesini kontrol edin
curl https://your-site.netlify.app/api/health

# soketi.rs sunucusunu kontrol edin
curl https://your-soketi-host.render.com/health

# Netlify function'ı kontrol edin
curl https://your-site.netlify.app/.netlify/functions/pusher-auth
```

### Netlify CLI Debug

```bash
# Function'ları yerel olarak test edin
netlify dev

# Function loglarını görüntüleyin
netlify functions:log pusher-auth

# Site durumunu kontrol edin
netlify status

# Son dağıtımları görüntüleyin
netlify deploy:list
```

## İlgili Kaynaklar

- [Netlify Dokümantasyonu](https://docs.netlify.com)
- [Netlify Functions Kılavuzu](https://docs.netlify.com/functions/overview/)
- [Netlify'da Next.js](https://docs.netlify.com/integrations/frameworks/next-js/)
- [soketi.rs Dokümantasyonu](https://docs.soketi.app)
- [Render Dokümantasyonu](https://render.com/docs)
- [Vercel Dağıtım Kılavuzu](./vercel.md)
- [Reverse Proxy Kurulumu](./reverse-proxy.md)
- [Başlangıç Kılavuzu](../baslangic.md)
- [API Referansı](../api-referans.md)
