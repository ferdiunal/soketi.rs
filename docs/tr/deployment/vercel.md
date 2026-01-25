# soketi.rs'yi Vercel'de Dağıtma

> Vercel platformunda soketi.rs WebSocket sunucusunu dağıtmak için eksiksiz kılavuz

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

Vercel, frontend framework'leri ve serverless fonksiyonlar için optimize edilmiş bir bulut platformudur. Vercel öncelikle statik siteler ve serverless API'ler için tasarlanmış olsa da, soketi.rs'yi Vercel'in Docker desteğini kullanarak konteynerleştirilmiş bir uygulama olarak veya Vercel'de barındırılan frontend'inizin bağlandığı ayrı bir servis olarak dağıtabilirsiniz.

**Önerilen Mimari:**
- Frontend (Next.js chat uygulaması): Vercel'de dağıtılır
- soketi.rs WebSocket sunucusu: Bir konteyner platformunda dağıtılır (Railway, Fly.io veya DigitalOcean)
- Bağlantı: Frontend, WebSocket üzerinden soketi.rs'ye bağlanır

## Ön Gereksinimler

Dağıtmadan önce, aşağıdakilere sahip olduğunuzdan emin olun:

- Bir Vercel hesabı ([buradan kaydolun](https://vercel.com/signup))
- Vercel CLI kurulu: `npm install -g vercel`
- soketi.rs projenizin bulunduğu Git deposu
- soketi.rs sunucusu için ayrı bir barındırma çözümü (Railway, Fly.io, vb.)

## Adım Adım Dağıtım

### Adım 1: soketi.rs Sunucusunu Dağıtın

Vercel uzun süreli WebSocket sunucularını doğal olarak desteklemediğinden, soketi.rs'yi bir konteyner platformunda dağıtın:

**Seçenek A: Railway Kullanarak**

```bash
# Railway CLI'yi yükleyin
npm install -g @railway/cli

# Railway'e giriş yapın
railway login

# Projeyi başlatın
railway init

# soketi.rs'yi dağıtın
railway up
```

**Seçenek B: Fly.io Kullanarak**

```bash
# Fly CLI'yi yükleyin
curl -L https://fly.io/install.sh | sh

# Fly'a giriş yapın
fly auth login

# soketi.rs'yi başlatın
fly launch

# Dağıtın
fly deploy
```

### Adım 2: Frontend'i Vercel'de Dağıtın

1. **Deponuzu bağlayın:**

```bash
# Vercel'e giriş yapın
vercel login

# Next.js projenize gidin
cd next-chat-app

# Dağıtın
vercel
```

2. **Vercel Dashboard üzerinden yapılandırın:**

- [Vercel Dashboard](https://vercel.com/dashboard)'a gidin
- "Import Project"e tıklayın
- Git deponuzu seçin
- Proje ayarlarını yapılandırın

### Adım 3: Frontend'i soketi.rs'ye Bağlayın

Frontend ortam değişkenlerinizi soketi.rs dağıtımınıza işaret edecek şekilde güncelleyin:

```bash
# Ortam değişkenlerini ayarlayın
vercel env add NEXT_PUBLIC_PUSHER_HOST
# soketi.rs host'unuzu girin (örn., your-app.railway.app)

vercel env add NEXT_PUBLIC_PUSHER_PORT
# Girin: 443

vercel env add NEXT_PUBLIC_PUSHER_KEY
# Pusher app key'inizi girin

vercel env add PUSHER_SECRET
# Pusher app secret'ınızı girin
```

## Ortam Değişkenleri

### Frontend Ortam Değişkenleri (Vercel)

Bunları Vercel proje ayarlarınızda veya `.env.production` dosyasında yapılandırın:

```bash
# Pusher Client Yapılandırması
NEXT_PUBLIC_PUSHER_KEY=your-app-key
NEXT_PUBLIC_PUSHER_HOST=your-soketi-host.railway.app
NEXT_PUBLIC_PUSHER_PORT=443
NEXT_PUBLIC_PUSHER_CLUSTER=mt1
NEXT_PUBLIC_PUSHER_FORCE_TLS=true
NEXT_PUBLIC_PUSHER_ENCRYPTED=true

# Pusher Server Yapılandırması
PUSHER_APP_ID=your-app-id
PUSHER_SECRET=your-app-secret
PUSHER_HOST=your-soketi-host.railway.app
PUSHER_PORT=443
PUSHER_USE_TLS=true

# Veritabanı (Better Auth kullanıyorsanız)
DATABASE_URL=postgresql://user:password@host:5432/database

# Better Auth
AUTH_SECRET=your-random-secret-key
AUTH_URL=https://your-app.vercel.app
```

### soketi.rs Sunucu Ortam Değişkenleri

Bunları konteyner platformunuzda yapılandırın (Railway, Fly.io, vb.):

```bash
# Uygulama Yapılandırması
SOKETI_DEFAULT_APP_ID=your-app-id
SOKETI_DEFAULT_APP_KEY=your-app-key
SOKETI_DEFAULT_APP_SECRET=your-app-secret

# Sunucu Yapılandırması
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001

# SSL Yapılandırması (özel domain kullanıyorsanız)
SOKETI_SSL_CERT=/path/to/cert.pem
SOKETI_SSL_KEY=/path/to/key.pem

# CORS Yapılandırması
SOKETI_CORS_ALLOWED_ORIGINS=https://your-app.vercel.app
```

## Build Ayarları

### Vercel Build Yapılandırması

Next.js projenizde `vercel.json` dosyasını oluşturun veya güncelleyin:

```json
{
  "version": 2,
  "buildCommand": "npm run build",
  "devCommand": "npm run dev",
  "installCommand": "npm install",
  "framework": "nextjs",
  "regions": ["iad1"],
  "env": {
    "NEXT_PUBLIC_PUSHER_KEY": "@pusher-key",
    "NEXT_PUBLIC_PUSHER_HOST": "@pusher-host",
    "NEXT_PUBLIC_PUSHER_PORT": "@pusher-port",
    "PUSHER_APP_ID": "@pusher-app-id",
    "PUSHER_SECRET": "@pusher-secret"
  },
  "build": {
    "env": {
      "NEXT_TELEMETRY_DISABLED": "1"
    }
  },
  "headers": [
    {
      "source": "/api/(.*)",
      "headers": [
        {
          "key": "Access-Control-Allow-Origin",
          "value": "*"
        },
        {
          "key": "Access-Control-Allow-Methods",
          "value": "GET, POST, PUT, DELETE, OPTIONS"
        },
        {
          "key": "Access-Control-Allow-Headers",
          "value": "Content-Type, Authorization"
        }
      ]
    }
  ]
}
```

### Next.js Yapılandırması

`next.config.js` dosyanızı optimize edin:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  
  // Çıktı yapılandırması
  output: 'standalone',
  
  // Görsel optimizasyonu
  images: {
    domains: ['localhost'],
    formats: ['image/avif', 'image/webp'],
  },
  
  // Sıkıştırma
  compress: true,
  
  // Güvenlik için header'lar
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
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
          {
            key: 'X-XSS-Protection',
            value: '1; mode=block',
          },
        ],
      },
    ];
  },
};

module.exports = nextConfig;
```

## WebSocket Yapılandırması

### İstemci Tarafı WebSocket Kurulumu

Next.js uygulamanızda Pusher client'ı yapılandırın (`lib/pusher.ts`):

```typescript
import Pusher from 'pusher-js';

export const pusherClient = new Pusher(process.env.NEXT_PUBLIC_PUSHER_KEY!, {
  wsHost: process.env.NEXT_PUBLIC_PUSHER_HOST,
  wsPort: parseInt(process.env.NEXT_PUBLIC_PUSHER_PORT || '443'),
  wssPort: parseInt(process.env.NEXT_PUBLIC_PUSHER_PORT || '443'),
  forceTLS: process.env.NEXT_PUBLIC_PUSHER_FORCE_TLS === 'true',
  encrypted: true,
  disableStats: true,
  enabledTransports: ['ws', 'wss'],
  cluster: process.env.NEXT_PUBLIC_PUSHER_CLUSTER || 'mt1',
  authEndpoint: '/api/pusher/auth',
});

// Bağlantı durumu izleme
pusherClient.connection.bind('connected', () => {
  console.log('✅ soketi.rs\'ye bağlandı');
});

pusherClient.connection.bind('disconnected', () => {
  console.log('❌ soketi.rs bağlantısı kesildi');
});

pusherClient.connection.bind('error', (err: any) => {
  console.error('❌ Bağlantı hatası:', err);
});

export default pusherClient;
```

### Sunucu Tarafı Yapılandırma

API route'larınızda Pusher server'ı yapılandırın (`app/api/pusher/auth/route.ts`):

```typescript
import { NextRequest, NextResponse } from 'next/server';
import Pusher from 'pusher';

const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID!,
  key: process.env.NEXT_PUBLIC_PUSHER_KEY!,
  secret: process.env.PUSHER_SECRET!,
  cluster: process.env.NEXT_PUBLIC_PUSHER_CLUSTER || 'mt1',
  host: process.env.PUSHER_HOST,
  port: process.env.PUSHER_PORT,
  useTLS: process.env.PUSHER_USE_TLS === 'true',
});

export async function POST(req: NextRequest) {
  // Kimlik doğrulama mantığı burada
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  if (!socketId || !channelName) {
    return NextResponse.json({ error: 'Eksik parametreler' }, { status: 400 });
  }

  // Kanalı yetkilendir
  const authResponse = pusher.authorizeChannel(socketId, channelName);
  return NextResponse.json(authResponse);
}
```

## SSL/TLS Kurulumu

### Vercel ile Otomatik SSL

Vercel, tüm dağıtımlar için otomatik olarak SSL sertifikaları sağlar:

- **Özel domain'ler**: Otomatik SSL sağlama
- **Vercel domain'leri**: Varsayılan olarak SSL etkin
- **Sertifika yenileme**: Let's Encrypt üzerinden otomatik

### soketi.rs SSL Yapılandırması

soketi.rs sunucunuz için, barındırma platformunuzda SSL'i yapılandırın:

**Railway:**
- Özel domain'ler için otomatik SSL
- SSL ile Railway'in sağladığı domain'i kullanın

**Fly.io:**
```bash
# Özel domain ekleyin
fly certs add your-domain.com

# Sertifika durumunu kontrol edin
fly certs show your-domain.com
```

**Manuel SSL Yapılandırması:**

Bir VPS kullanıyorsanız, soketi.rs'de SSL'i yapılandırın:

```bash
# Ortam değişkenleri
SOKETI_SSL_CERT=/etc/ssl/certs/cert.pem
SOKETI_SSL_KEY=/etc/ssl/private/key.pem
SOKETI_SSL_PASSPHRASE=your-passphrase
```

## İzleme ve Loglama

### Vercel İzleme

1. **Gerçek Zamanlı Loglar:**

```bash
# Dağıtım loglarını görüntüleyin
vercel logs

# Logları gerçek zamanlı takip edin
vercel logs --follow
```

2. **Vercel Dashboard:**
   - Projenize gidin
   - "Deployments" → Dağıtımı seçin
   - Logları, metrikleri ve hataları görüntüleyin

3. **Analytics:**
   - Proje ayarlarında Vercel Analytics'i etkinleştirin
   - Sayfa görüntülemelerini, performansı ve Web Vitals'ı izleyin

### soketi.rs İzleme

**Health Check Endpoint:**

```bash
# Sunucu sağlığını kontrol edin
curl https://your-soketi-host.railway.app/health
```

**Loglama Yapılandırması:**

```bash
# Debug loglama'yı etkinleştirin
SOKETI_DEBUG=true
SOKETI_LOG_LEVEL=debug
```

**İzleme Araçları:**

- **Railway**: Yerleşik metrikler ve loglar
- **Fly.io**: `fly logs` komutu ve dashboard
- **Harici**: Datadog, New Relic veya Sentry gibi servisleri kullanın

### Uygulama İzleme

Next.js uygulamanıza hata takibi entegre edin:

```typescript
// lib/monitoring.ts
import * as Sentry from '@sentry/nextjs';

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.NODE_ENV,
  tracesSampleRate: 1.0,
});
```

## Maliyet Tahmini

### Vercel Fiyatlandırması

**Hobby Planı (Ücretsiz):**
- Ayda 100 GB bant genişliği
- Sınırsız dağıtım
- Otomatik SSL
- **En uygun**: Geliştirme ve küçük projeler

**Pro Planı ($20/ay):**
- Ayda 1 TB bant genişliği
- Gelişmiş analytics
- Şifre koruması
- **En uygun**: Production uygulamaları

**Enterprise Planı (Özel):**
- Özel bant genişliği
- SLA garantileri
- Özel destek
- **En uygun**: Büyük ölçekli uygulamalar

### soketi.rs Barındırma Maliyetleri

**Railway:**
- **Starter**: $5/ay (512 MB RAM, 1 GB depolama)
- **Developer**: $10/ay (1 GB RAM, 10 GB depolama)
- **Team**: $20/ay (2 GB RAM, 20 GB depolama)

**Fly.io:**
- **Ücretsiz katman**: 3 shared-cpu-1x VM, 160 GB bant genişliği
- **Ücretli**: Kaynaklara bağlı olarak ~$5-20/ay

**DigitalOcean:**
- **Basic Droplet**: $6/ay (1 GB RAM, 1 vCPU)
- **App Platform**: $5/ay (512 MB RAM)

### Toplam Aylık Maliyet Tahmini

**Küçük Proje:**
- Vercel Hobby: $0
- Railway Starter: $5
- **Toplam**: $5/ay

**Production Uygulaması:**
- Vercel Pro: $20
- Railway Developer: $10
- Veritabanı (opsiyonel): $7
- **Toplam**: $37/ay

**Enterprise:**
- Vercel Enterprise: Özel
- Özel sunucular: $50-200+
- **Toplam**: $100-500+/ay

## Ölçeklendirme Önerileri

### Yatay Ölçeklendirme

**soketi.rs Cluster:**

Bir load balancer arkasında birden fazla soketi.rs instance'ı dağıtın:

```yaml
# docker-compose.yml
version: '3.8'
services:
  soketi-1:
    image: quay.io/soketi/soketi:latest-16-alpine
    environment:
      - SOKETI_DEFAULT_APP_ID=app-id
      - SOKETI_DEFAULT_APP_KEY=app-key
      - SOKETI_DEFAULT_APP_SECRET=app-secret
    
  soketi-2:
    image: quay.io/soketi/soketi:latest-16-alpine
    environment:
      - SOKETI_DEFAULT_APP_ID=app-id
      - SOKETI_DEFAULT_APP_KEY=app-key
      - SOKETI_DEFAULT_APP_SECRET=app-secret
  
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - soketi-1
      - soketi-2
```

### Dikey Ölçeklendirme

**Kaynakları Artırın:**

Railway:
```bash
# Railway dashboard'da planı yükseltin
# Settings → Plan → Daha yüksek katmanı seçin
```

Fly.io:
```bash
# VM boyutunu ölçeklendirin
fly scale vm shared-cpu-2x

# VM sayısını ölçeklendirin
fly scale count 3
```

### Vercel Ölçeklendirme

Vercel, frontend'inizi otomatik olarak ölçeklendirir:
- **Edge Network**: Global CDN dağıtımı
- **Serverless Functions**: Trafiğe göre otomatik ölçeklendirme
- **Yapılandırma gerekmez**: Otomatik olarak ölçeklenir

### Performans Optimizasyonu

1. **soketi.rs için Redis'i etkinleştirin:**

```bash
# Yatay ölçeklendirme için Redis adapter kullanın
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=your-redis-host
SOKETI_REDIS_PORT=6379
```

2. **Connection Pooling:**

```typescript
// Pusher client bağlantılarını optimize edin
const pusher = new Pusher(key, {
  cluster: 'mt1',
  enabledTransports: ['ws', 'wss'],
  activityTimeout: 30000,
  pongTimeout: 10000,
});
```

3. **Önbellekleme Stratejisi:**

```javascript
// next.config.js
module.exports = {
  async headers() {
    return [
      {
        source: '/static/:path*',
        headers: [
          {
            key: 'Cache-Control',
            value: 'public, max-age=31536000, immutable',
          },
        ],
      },
    ];
  },
};
```

## Sorun Giderme

### Yaygın Sorunlar

**1. WebSocket Bağlantısı Başarısız**

```
Error: WebSocket connection to 'wss://...' failed
```

**Çözüm:**
- `NEXT_PUBLIC_PUSHER_HOST`'un doğru olduğunu doğrulayın
- soketi.rs sunucusunun çalıştığından emin olun
- Firewall kurallarının WebSocket bağlantılarına izin verdiğini kontrol edin
- SSL sertifikasının geçerli olduğunu doğrulayın

**2. CORS Hataları**

```
Access to XMLHttpRequest blocked by CORS policy
```

**Çözüm:**
```bash
# Vercel domain'ini soketi.rs CORS'a ekleyin
SOKETI_CORS_ALLOWED_ORIGINS=https://your-app.vercel.app,https://your-app-*.vercel.app
```

**3. Ortam Değişkenleri Yüklenmiyor**

**Çözüm:**
```bash
# Ortam değişkenlerini ayarladıktan sonra yeniden dağıtın
vercel env pull
vercel --prod
```

**4. Build Hataları**

```
Error: Module not found
```

**Çözüm:**
```bash
# Önbelleği temizleyin ve yeniden build edin
vercel --force
```

**5. Yüksek Gecikme**

**Çözüm:**
- soketi.rs'yi Vercel dağıtımıyla aynı bölgede dağıtın
- Vercel'in edge network'ünü kullanın
- Next.js config'de sıkıştırmayı etkinleştirin

### Debug Modu

Debug loglama'yı etkinleştirin:

```typescript
// lib/pusher.ts
Pusher.logToConsole = true;

const pusher = new Pusher(key, {
  enabledTransports: ['ws', 'wss'],
  // ... diğer yapılandırma
});
```

### Health Check'ler

Dağıtımınızı izleyin:

```bash
# Vercel dağıtımını kontrol edin
curl https://your-app.vercel.app/api/health

# soketi.rs sunucusunu kontrol edin
curl https://your-soketi-host.railway.app/health
```

## İlgili Kaynaklar

- [Vercel Dokümantasyonu](https://vercel.com/docs)
- [Next.js Dağıtım Kılavuzu](https://nextjs.org/docs/deployment)
- [soketi.rs Dokümantasyonu](https://docs.soketi.app)
- [Railway Dokümantasyonu](https://docs.railway.app)
- [Fly.io Dokümantasyonu](https://fly.io/docs)
- [Netlify Dağıtım Kılavuzu](./netlify.md)
- [Reverse Proxy Kurulumu](./reverse-proxy.md)
- [Başlangıç Kılavuzu](../baslangic.md)
- [API Referansı](../api-referans.md)
