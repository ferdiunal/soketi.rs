# Deploying soketi.rs on Vercel

> Complete guide for deploying soketi.rs WebSocket server on Vercel platform

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Step-by-Step Deployment](#step-by-step-deployment)
- [Environment Variables](#environment-variables)
- [Build Settings](#build-settings)
- [WebSocket Configuration](#websocket-configuration)
- [SSL/TLS Setup](#ssltls-setup)
- [Monitoring and Logging](#monitoring-and-logging)
- [Cost Estimation](#cost-estimation)
- [Scaling Recommendations](#scaling-recommendations)
- [Troubleshooting](#troubleshooting)

## Overview

Vercel is a cloud platform optimized for frontend frameworks and serverless functions. While Vercel is primarily designed for static sites and serverless APIs, you can deploy soketi.rs as a containerized application using Vercel's Docker support or as a separate service that your Vercel-hosted frontend connects to.

**Recommended Architecture:**
- Frontend (Next.js chat app): Deployed on Vercel
- soketi.rs WebSocket server: Deployed on a container platform (Railway, Fly.io, or DigitalOcean)
- Connection: Frontend connects to soketi.rs via WebSocket

## Prerequisites

Before deploying, ensure you have:

- A Vercel account ([sign up here](https://vercel.com/signup))
- Vercel CLI installed: `npm install -g vercel`
- Git repository with your soketi.rs project
- A separate hosting solution for the soketi.rs server (Railway, Fly.io, etc.)

## Step-by-Step Deployment

### Step 1: Deploy soketi.rs Server

Since Vercel doesn't natively support long-running WebSocket servers, deploy soketi.rs on a container platform:

**Option A: Using Railway**

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login to Railway
railway login

# Initialize project
railway init

# Deploy soketi.rs
railway up
```

**Option B: Using Fly.io**

```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Login to Fly
fly auth login

# Launch soketi.rs
fly launch

# Deploy
fly deploy
```

### Step 2: Deploy Frontend on Vercel

1. **Connect your repository:**

```bash
# Login to Vercel
vercel login

# Navigate to your Next.js project
cd next-chat-app

# Deploy
vercel
```

2. **Configure through Vercel Dashboard:**

- Go to [Vercel Dashboard](https://vercel.com/dashboard)
- Click "Import Project"
- Select your Git repository
- Configure project settings

### Step 3: Link Frontend to soketi.rs

Update your frontend environment variables to point to your soketi.rs deployment:

```bash
# Set environment variables
vercel env add NEXT_PUBLIC_PUSHER_HOST
# Enter your soketi.rs host (e.g., your-app.railway.app)

vercel env add NEXT_PUBLIC_PUSHER_PORT
# Enter: 443

vercel env add NEXT_PUBLIC_PUSHER_KEY
# Enter your Pusher app key

vercel env add PUSHER_SECRET
# Enter your Pusher app secret
```

## Environment Variables

### Frontend Environment Variables (Vercel)

Configure these in your Vercel project settings or `.env.production`:

```bash
# Pusher Client Configuration
NEXT_PUBLIC_PUSHER_KEY=your-app-key
NEXT_PUBLIC_PUSHER_HOST=your-soketi-host.railway.app
NEXT_PUBLIC_PUSHER_PORT=443
NEXT_PUBLIC_PUSHER_CLUSTER=mt1
NEXT_PUBLIC_PUSHER_FORCE_TLS=true
NEXT_PUBLIC_PUSHER_ENCRYPTED=true

# Pusher Server Configuration
PUSHER_APP_ID=your-app-id
PUSHER_SECRET=your-app-secret
PUSHER_HOST=your-soketi-host.railway.app
PUSHER_PORT=443
PUSHER_USE_TLS=true

# Database (if using Better Auth)
DATABASE_URL=postgresql://user:password@host:5432/database

# Better Auth
AUTH_SECRET=your-random-secret-key
AUTH_URL=https://your-app.vercel.app
```

### soketi.rs Server Environment Variables

Configure these on your container platform (Railway, Fly.io, etc.):

```bash
# App Configuration
SOKETI_DEFAULT_APP_ID=your-app-id
SOKETI_DEFAULT_APP_KEY=your-app-key
SOKETI_DEFAULT_APP_SECRET=your-app-secret

# Server Configuration
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001

# SSL Configuration (if using custom domain)
SOKETI_SSL_CERT=/path/to/cert.pem
SOKETI_SSL_KEY=/path/to/key.pem

# CORS Configuration
SOKETI_CORS_ALLOWED_ORIGINS=https://your-app.vercel.app
```

## Build Settings

### Vercel Build Configuration

Create or update `vercel.json` in your Next.js project:

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

### Next.js Configuration

Optimize your `next.config.js`:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  
  // Output configuration
  output: 'standalone',
  
  // Image optimization
  images: {
    domains: ['localhost'],
    formats: ['image/avif', 'image/webp'],
  },
  
  // Compression
  compress: true,
  
  // Headers for security
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

## WebSocket Configuration

### Client-Side WebSocket Setup

Configure Pusher client in your Next.js app (`lib/pusher.ts`):

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

// Connection state monitoring
pusherClient.connection.bind('connected', () => {
  console.log('✅ Connected to soketi.rs');
});

pusherClient.connection.bind('disconnected', () => {
  console.log('❌ Disconnected from soketi.rs');
});

pusherClient.connection.bind('error', (err: any) => {
  console.error('❌ Connection error:', err);
});

export default pusherClient;
```

### Server-Side Configuration

Configure Pusher server in your API routes (`app/api/pusher/auth/route.ts`):

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
  // Authentication logic here
  const body = await req.text();
  const params = new URLSearchParams(body);
  const socketId = params.get('socket_id');
  const channelName = params.get('channel_name');

  if (!socketId || !channelName) {
    return NextResponse.json({ error: 'Missing parameters' }, { status: 400 });
  }

  // Authorize channel
  const authResponse = pusher.authorizeChannel(socketId, channelName);
  return NextResponse.json(authResponse);
}
```

## SSL/TLS Setup

### Automatic SSL with Vercel

Vercel automatically provides SSL certificates for all deployments:

- **Custom domains**: Automatic SSL provisioning
- **Vercel domains**: SSL enabled by default
- **Certificate renewal**: Automatic via Let's Encrypt

### soketi.rs SSL Configuration

For your soketi.rs server, configure SSL on your hosting platform:

**Railway:**
- Automatic SSL for custom domains
- Use Railway's provided domain with SSL

**Fly.io:**
```bash
# Add custom domain
fly certs add your-domain.com

# Check certificate status
fly certs show your-domain.com
```

**Manual SSL Configuration:**

If using a VPS, configure SSL in soketi.rs:

```bash
# Environment variables
SOKETI_SSL_CERT=/etc/ssl/certs/cert.pem
SOKETI_SSL_KEY=/etc/ssl/private/key.pem
SOKETI_SSL_PASSPHRASE=your-passphrase
```

## Monitoring and Logging

### Vercel Monitoring

1. **Real-time Logs:**

```bash
# View deployment logs
vercel logs

# Follow logs in real-time
vercel logs --follow
```

2. **Vercel Dashboard:**
   - Navigate to your project
   - Click "Deployments" → Select deployment
   - View logs, metrics, and errors

3. **Analytics:**
   - Enable Vercel Analytics in project settings
   - Monitor page views, performance, and Web Vitals

### soketi.rs Monitoring

**Health Check Endpoint:**

```bash
# Check server health
curl https://your-soketi-host.railway.app/health
```

**Logging Configuration:**

```bash
# Enable debug logging
SOKETI_DEBUG=true
SOKETI_LOG_LEVEL=debug
```

**Monitoring Tools:**

- **Railway**: Built-in metrics and logs
- **Fly.io**: `fly logs` command and dashboard
- **External**: Use services like Datadog, New Relic, or Sentry

### Application Monitoring

Integrate error tracking in your Next.js app:

```typescript
// lib/monitoring.ts
import * as Sentry from '@sentry/nextjs';

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.NODE_ENV,
  tracesSampleRate: 1.0,
});
```

## Cost Estimation

### Vercel Pricing

**Hobby Plan (Free):**
- 100 GB bandwidth/month
- Unlimited deployments
- Automatic SSL
- **Best for**: Development and small projects

**Pro Plan ($20/month):**
- 1 TB bandwidth/month
- Advanced analytics
- Password protection
- **Best for**: Production applications

**Enterprise Plan (Custom):**
- Custom bandwidth
- SLA guarantees
- Dedicated support
- **Best for**: Large-scale applications

### soketi.rs Hosting Costs

**Railway:**
- **Starter**: $5/month (512 MB RAM, 1 GB storage)
- **Developer**: $10/month (1 GB RAM, 10 GB storage)
- **Team**: $20/month (2 GB RAM, 20 GB storage)

**Fly.io:**
- **Free tier**: 3 shared-cpu-1x VMs, 160 GB bandwidth
- **Paid**: ~$5-20/month depending on resources

**DigitalOcean:**
- **Basic Droplet**: $6/month (1 GB RAM, 1 vCPU)
- **App Platform**: $5/month (512 MB RAM)

### Total Monthly Cost Estimate

**Small Project:**
- Vercel Hobby: $0
- Railway Starter: $5
- **Total**: $5/month

**Production Application:**
- Vercel Pro: $20
- Railway Developer: $10
- Database (optional): $7
- **Total**: $37/month

**Enterprise:**
- Vercel Enterprise: Custom
- Dedicated servers: $50-200+
- **Total**: $100-500+/month

## Scaling Recommendations

### Horizontal Scaling

**soketi.rs Cluster:**

Deploy multiple soketi.rs instances behind a load balancer:

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

### Vertical Scaling

**Increase Resources:**

Railway:
```bash
# Upgrade plan in Railway dashboard
# Settings → Plan → Select higher tier
```

Fly.io:
```bash
# Scale VM size
fly scale vm shared-cpu-2x

# Scale VM count
fly scale count 3
```

### Vercel Scaling

Vercel automatically scales your frontend:
- **Edge Network**: Global CDN distribution
- **Serverless Functions**: Auto-scaling based on traffic
- **No configuration needed**: Scales automatically

### Performance Optimization

1. **Enable Redis for soketi.rs:**

```bash
# Use Redis adapter for horizontal scaling
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=your-redis-host
SOKETI_REDIS_PORT=6379
```

2. **Connection Pooling:**

```typescript
// Optimize Pusher client connections
const pusher = new Pusher(key, {
  cluster: 'mt1',
  enabledTransports: ['ws', 'wss'],
  activityTimeout: 30000,
  pongTimeout: 10000,
});
```

3. **Caching Strategy:**

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

## Troubleshooting

### Common Issues

**1. WebSocket Connection Failed**

```
Error: WebSocket connection to 'wss://...' failed
```

**Solution:**
- Verify `NEXT_PUBLIC_PUSHER_HOST` is correct
- Ensure soketi.rs server is running
- Check firewall rules allow WebSocket connections
- Verify SSL certificate is valid

**2. CORS Errors**

```
Access to XMLHttpRequest blocked by CORS policy
```

**Solution:**
```bash
# Add Vercel domain to soketi.rs CORS
SOKETI_CORS_ALLOWED_ORIGINS=https://your-app.vercel.app,https://your-app-*.vercel.app
```

**3. Environment Variables Not Loading**

**Solution:**
```bash
# Redeploy after setting environment variables
vercel env pull
vercel --prod
```

**4. Build Failures**

```
Error: Module not found
```

**Solution:**
```bash
# Clear cache and rebuild
vercel --force
```

**5. High Latency**

**Solution:**
- Deploy soketi.rs in same region as Vercel deployment
- Use Vercel's edge network
- Enable compression in Next.js config

### Debug Mode

Enable debug logging:

```typescript
// lib/pusher.ts
Pusher.logToConsole = true;

const pusher = new Pusher(key, {
  enabledTransports: ['ws', 'wss'],
  // ... other config
});
```

### Health Checks

Monitor your deployment:

```bash
# Check Vercel deployment
curl https://your-app.vercel.app/api/health

# Check soketi.rs server
curl https://your-soketi-host.railway.app/health
```

## Related Resources

- [Vercel Documentation](https://vercel.com/docs)
- [Next.js Deployment Guide](https://nextjs.org/docs/deployment)
- [soketi.rs Documentation](https://docs.soketi.app)
- [Railway Documentation](https://docs.railway.app)
- [Fly.io Documentation](https://fly.io/docs)
- [Netlify Deployment Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/deployment/netlify.md)
- [Reverse Proxy Setup](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/deployment/reverse-proxy.md)
- [Getting Started Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md)
- [API Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/api-reference.md)
