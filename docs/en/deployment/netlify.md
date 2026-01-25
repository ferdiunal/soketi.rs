# Deploying soketi.rs on Netlify

> Complete guide for deploying soketi.rs WebSocket server with Netlify platform

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

Netlify is a modern cloud platform for web projects with continuous deployment, serverless functions, and edge computing capabilities. Similar to Vercel, Netlify is optimized for static sites and serverless functions, so soketi.rs WebSocket server should be deployed separately while your frontend is hosted on Netlify.

**Recommended Architecture:**
- Frontend (Next.js chat app): Deployed on Netlify
- soketi.rs WebSocket server: Deployed on a container platform (Railway, Fly.io, or Render)
- Connection: Frontend connects to soketi.rs via WebSocket

## Prerequisites

Before deploying, ensure you have:

- A Netlify account ([sign up here](https://app.netlify.com/signup))
- Netlify CLI installed: `npm install -g netlify-cli`
- Git repository with your project
- A separate hosting solution for soketi.rs server

## Step-by-Step Deployment

### Step 1: Deploy soketi.rs Server

Deploy soketi.rs on a container platform that supports long-running WebSocket connections:

**Option A: Using Render**

```bash
# Create render.yaml in your soketi.rs project
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

# Deploy to Render
# Push to GitHub and connect via Render dashboard
```

**Option B: Using Railway**

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway init
railway up
```

**Option C: Using Fly.io**

```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Deploy
fly auth login
fly launch
fly deploy
```

### Step 2: Deploy Frontend on Netlify

**Method 1: Using Netlify CLI**

```bash
# Login to Netlify
netlify login

# Navigate to your Next.js project
cd next-chat-app

# Initialize Netlify site
netlify init

# Deploy
netlify deploy --prod
```

**Method 2: Using Netlify Dashboard**

1. Go to [Netlify Dashboard](https://app.netlify.com)
2. Click "Add new site" → "Import an existing project"
3. Connect your Git provider (GitHub, GitLab, Bitbucket)
4. Select your repository
5. Configure build settings:
   - **Build command**: `npm run build`
   - **Publish directory**: `.next`
   - **Functions directory**: `.netlify/functions`

### Step 3: Configure netlify.toml

Create `netlify.toml` in your project root:

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

### Step 4: Link Frontend to soketi.rs

Configure environment variables in Netlify:

```bash
# Using Netlify CLI
netlify env:set NEXT_PUBLIC_PUSHER_HOST your-soketi-host.railway.app
netlify env:set NEXT_PUBLIC_PUSHER_PORT 443
netlify env:set NEXT_PUBLIC_PUSHER_KEY your-app-key
netlify env:set PUSHER_SECRET your-app-secret

# Or use Netlify Dashboard:
# Site settings → Environment variables → Add variables
```

## Environment Variables

### Frontend Environment Variables (Netlify)

Configure these in Netlify Dashboard or via CLI:

```bash
# Pusher Client Configuration
NEXT_PUBLIC_PUSHER_KEY=your-app-key
NEXT_PUBLIC_PUSHER_HOST=your-soketi-host.render.com
NEXT_PUBLIC_PUSHER_PORT=443
NEXT_PUBLIC_PUSHER_CLUSTER=mt1
NEXT_PUBLIC_PUSHER_FORCE_TLS=true
NEXT_PUBLIC_PUSHER_ENCRYPTED=true

# Pusher Server Configuration
PUSHER_APP_ID=your-app-id
PUSHER_SECRET=your-app-secret
PUSHER_HOST=your-soketi-host.render.com
PUSHER_PORT=443
PUSHER_USE_TLS=true

# Database (if using Better Auth)
DATABASE_URL=postgresql://user:password@host:5432/database

# Better Auth
AUTH_SECRET=your-random-secret-key
AUTH_URL=https://your-site.netlify.app

# Netlify-specific
NETLIFY_SITE_ID=your-site-id
NETLIFY_AUTH_TOKEN=your-auth-token
```

### soketi.rs Server Environment Variables

Configure these on your container platform:

```bash
# App Configuration
SOKETI_DEFAULT_APP_ID=your-app-id
SOKETI_DEFAULT_APP_KEY=your-app-key
SOKETI_DEFAULT_APP_SECRET=your-app-secret

# Server Configuration
SOKETI_HOST=0.0.0.0
SOKETI_PORT=6001

# CORS Configuration
SOKETI_CORS_ALLOWED_ORIGINS=https://your-site.netlify.app,https://deploy-preview-*--your-site.netlify.app

# SSL Configuration (if needed)
SOKETI_SSL_CERT=/path/to/cert.pem
SOKETI_SSL_KEY=/path/to/key.pem
```

## Build Settings

### Netlify Build Configuration

**netlify.toml** (detailed configuration):

```toml
[build]
  command = "npm run build"
  publish = ".next"
  functions = ".netlify/functions"

[build.environment]
  # Node.js version
  NODE_VERSION = "20"
  
  # Disable telemetry
  NEXT_TELEMETRY_DISABLED = "1"
  
  # Build optimizations
  NODE_OPTIONS = "--max-old-space-size=4096"

# Next.js plugin for Netlify
[[plugins]]
  package = "@netlify/plugin-nextjs"

# Security headers
[[headers]]
  for = "/*"
  [headers.values]
    Strict-Transport-Security = "max-age=63072000; includeSubDomains; preload"
    X-Frame-Options = "DENY"
    X-Content-Type-Options = "nosniff"
    X-XSS-Protection = "1; mode=block"
    Referrer-Policy = "strict-origin-when-cross-origin"
    Permissions-Policy = "camera=(), microphone=(), geolocation=()"

# API routes headers
[[headers]]
  for = "/api/*"
  [headers.values]
    Access-Control-Allow-Origin = "*"
    Access-Control-Allow-Methods = "GET, POST, PUT, DELETE, OPTIONS"
    Access-Control-Allow-Headers = "Content-Type, Authorization"
    Cache-Control = "no-cache, no-store, must-revalidate"

# Static assets caching
[[headers]]
  for = "/_next/static/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"

[[headers]]
  for = "/static/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"

# Redirects for API routes
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

# Functions configuration
[functions]
  directory = ".netlify/functions"
  node_bundler = "esbuild"
  included_files = ["prisma/**/*"]

# Development settings
[dev]
  command = "npm run dev"
  port = 3000
  targetPort = 3000
  autoLaunch = false
  framework = "#custom"

# Context-specific settings
[context.production]
  environment = { NODE_ENV = "production" }

[context.deploy-preview]
  environment = { NODE_ENV = "preview" }

[context.branch-deploy]
  environment = { NODE_ENV = "development" }
```

### Next.js Configuration for Netlify

Update `next.config.js`:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  
  // Netlify-specific output
  output: 'standalone',
  
  // Image optimization
  images: {
    domains: ['localhost'],
    formats: ['image/avif', 'image/webp'],
    // Netlify Image CDN
    loader: 'custom',
    loaderFile: './lib/netlify-image-loader.js',
  },
  
  // Compression
  compress: true,
  
  // Trailing slash for Netlify
  trailingSlash: false,
  
  // Environment variables
  env: {
    NEXT_PUBLIC_PUSHER_KEY: process.env.NEXT_PUBLIC_PUSHER_KEY,
    NEXT_PUBLIC_PUSHER_HOST: process.env.NEXT_PUBLIC_PUSHER_HOST,
    NEXT_PUBLIC_PUSHER_PORT: process.env.NEXT_PUBLIC_PUSHER_PORT,
  },
  
  // Webpack configuration
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
  
  // Security headers
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

### Package.json Scripts

Add Netlify-specific scripts:

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

## WebSocket Configuration

### Client-Side WebSocket Setup

Configure Pusher client for Netlify deployment (`lib/pusher.ts`):

```typescript
import Pusher from 'pusher-js';

// Enable logging in development
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
  
  // Connection timeout settings
  activityTimeout: 30000,
  pongTimeout: 10000,
  unavailableTimeout: 10000,
});

// Connection state handlers
pusherClient.connection.bind('connected', () => {
  console.log('✅ Connected to soketi.rs');
});

pusherClient.connection.bind('disconnected', () => {
  console.log('❌ Disconnected from soketi.rs');
});

pusherClient.connection.bind('error', (err: any) => {
  console.error('❌ Connection error:', err);
});

pusherClient.connection.bind('state_change', (states: any) => {
  console.log('🔄 Connection state:', states.current);
});

export default pusherClient;
```

### Netlify Functions for Pusher Auth

Create `.netlify/functions/pusher-auth.ts`:

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
  // Only allow POST requests
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
        body: JSON.stringify({ error: 'Missing parameters' }),
      };
    }

    // Get user from session (implement your auth logic)
    // const user = await getUser(event);

    // Presence channel
    if (channelName.startsWith('presence-')) {
      const presenceData = {
        user_id: 'user-123', // Replace with actual user ID
        user_info: {
          name: 'User Name',
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
      body: JSON.stringify({ error: 'Invalid channel' }),
    };
  } catch (error) {
    console.error('Auth error:', error);
    return {
      statusCode: 500,
      body: JSON.stringify({ error: 'Internal server error' }),
    };
  }
};
```

## SSL/TLS Setup

### Automatic SSL with Netlify

Netlify provides automatic SSL certificates for all sites:

- **Netlify domains**: SSL enabled by default (*.netlify.app)
- **Custom domains**: Automatic SSL provisioning via Let's Encrypt
- **Certificate renewal**: Automatic, no manual intervention needed
- **HTTPS redirect**: Automatic redirect from HTTP to HTTPS

### Custom Domain SSL Setup

1. **Add custom domain in Netlify Dashboard:**
   - Site settings → Domain management → Add custom domain
   - Follow DNS configuration instructions

2. **Verify SSL certificate:**
   - Netlify automatically provisions SSL certificate
   - Usually takes 1-2 minutes
   - Check status in Domain management section

3. **Force HTTPS:**

Add to `netlify.toml`:

```toml
[[redirects]]
  from = "http://*"
  to = "https://:splat"
  status = 301
  force = true
```

### soketi.rs SSL Configuration

Configure SSL on your soketi.rs hosting platform:

**Render:**
- Automatic SSL for all services
- Custom domains supported with automatic SSL

**Railway:**
- Automatic SSL for Railway domains
- Custom domain SSL available

**Fly.io:**
```bash
# Add custom domain with SSL
fly certs add your-domain.com

# Check certificate status
fly certs show your-domain.com
```

## Monitoring and Logging

### Netlify Monitoring

**1. Real-time Logs:**

```bash
# View function logs
netlify functions:log

# View build logs
netlify watch

# View site logs
netlify logs
```

**2. Netlify Dashboard:**
   - Navigate to your site
   - **Deploys**: View deployment history and logs
   - **Functions**: Monitor function invocations and errors
   - **Analytics**: Track page views and performance

**3. Netlify Analytics:**

Enable in site settings:
- Real user metrics
- Page views and unique visitors
- Top pages and sources
- No JavaScript required (server-side analytics)

### Function Monitoring

Monitor Netlify Functions:

```typescript
// Add logging to functions
export const handler: Handler = async (event) => {
  console.log('Function invoked:', {
    path: event.path,
    method: event.httpMethod,
    timestamp: new Date().toISOString(),
  });
  
  // Your function logic
  
  return {
    statusCode: 200,
    body: JSON.stringify({ success: true }),
  };
};
```

### soketi.rs Monitoring

**Health Check:**

```bash
# Check server health
curl https://your-soketi-host.render.com/health
```

**Logging:**

```bash
# Enable debug logging
SOKETI_DEBUG=true
SOKETI_LOG_LEVEL=debug
```

### External Monitoring Tools

Integrate third-party monitoring:

**Sentry for Error Tracking:**

```typescript
// lib/sentry.ts
import * as Sentry from '@sentry/nextjs';

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.NETLIFY_CONTEXT || 'development',
  tracesSampleRate: 1.0,
});
```

**LogRocket for Session Replay:**

```typescript
// lib/logrocket.ts
import LogRocket from 'logrocket';

if (typeof window !== 'undefined' && process.env.NODE_ENV === 'production') {
  LogRocket.init(process.env.NEXT_PUBLIC_LOGROCKET_ID!);
}
```

## Cost Estimation

### Netlify Pricing

**Starter Plan (Free):**
- 100 GB bandwidth/month
- 300 build minutes/month
- Automatic SSL
- Deploy previews
- **Best for**: Personal projects and development

**Pro Plan ($19/month):**
- 400 GB bandwidth/month
- 1,000 build minutes/month
- Password protection
- Analytics
- **Best for**: Professional sites

**Business Plan ($99/month):**
- 1 TB bandwidth/month
- Unlimited build minutes
- SSO/SAML
- Priority support
- **Best for**: Team projects

**Enterprise Plan (Custom):**
- Custom bandwidth and builds
- SLA guarantees
- Dedicated support
- **Best for**: Large organizations

### soketi.rs Hosting Costs

**Render:**
- **Free tier**: 750 hours/month (sleeps after inactivity)
- **Starter**: $7/month (always on, 512 MB RAM)
- **Standard**: $25/month (2 GB RAM)

**Railway:**
- **Starter**: $5/month (512 MB RAM)
- **Developer**: $10/month (1 GB RAM)

**Fly.io:**
- **Free tier**: 3 shared VMs, 160 GB bandwidth
- **Paid**: ~$5-20/month depending on resources

### Total Monthly Cost Estimate

**Small Project:**
- Netlify Starter: $0
- Render Free: $0
- **Total**: $0/month (with limitations)

**Production Application:**
- Netlify Pro: $19
- Render Starter: $7
- Database (optional): $7
- **Total**: $33/month

**Business Application:**
- Netlify Business: $99
- Render Standard: $25
- Database: $15
- Monitoring: $10
- **Total**: $149/month

**Enterprise:**
- Netlify Enterprise: Custom ($200+)
- Dedicated servers: $100-500+
- **Total**: $300-1000+/month

## Scaling Recommendations

### Horizontal Scaling

**soketi.rs Cluster:**

Deploy multiple instances with Redis adapter:

```bash
# soketi.rs environment variables
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=your-redis-host.com
SOKETI_REDIS_PORT=6379
SOKETI_REDIS_PASSWORD=your-redis-password
```

**Load Balancing:**

Use your hosting platform's load balancer or deploy behind Nginx/Caddy.

### Vertical Scaling

**Increase Resources:**

Render:
```bash
# Upgrade plan in Render dashboard
# Settings → Instance Type → Select higher tier
```

Railway:
```bash
# Upgrade in Railway dashboard
# Settings → Plan → Select higher tier
```

### Netlify Scaling

Netlify automatically scales:
- **Edge Network**: Global CDN with 100+ locations
- **Functions**: Auto-scaling serverless functions
- **Instant Cache Invalidation**: Fast content updates
- **No configuration needed**: Scales automatically with traffic

### Performance Optimization

**1. Enable Redis for soketi.rs:**

```bash
# Horizontal scaling with Redis
SOKETI_ADAPTER_DRIVER=redis
SOKETI_REDIS_HOST=redis-host
SOKETI_REDIS_PORT=6379
```

**2. Optimize Netlify Functions:**

```typescript
// Use edge functions for better performance
export const config = {
  path: '/api/pusher/auth',
  cache: 'manual',
};
```

**3. Implement Caching:**

```toml
# netlify.toml
[[headers]]
  for = "/_next/static/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
```

**4. Use Netlify Image CDN:**

```javascript
// next.config.js
module.exports = {
  images: {
    loader: 'custom',
    loaderFile: './lib/netlify-image-loader.js',
  },
};
```

## Troubleshooting

### Common Issues

**1. Build Failures**

```
Error: Command failed with exit code 1
```

**Solution:**
```bash
# Check build logs in Netlify dashboard
# Clear cache and retry
netlify build --clear-cache

# Test build locally
npm run build
```

**2. Function Timeout**

```
Error: Function execution timed out
```

**Solution:**
```toml
# Increase function timeout in netlify.toml
[functions]
  timeout = 30
```

**3. WebSocket Connection Failed**

```
Error: WebSocket connection failed
```

**Solution:**
- Verify soketi.rs server is running
- Check CORS configuration
- Ensure SSL certificates are valid
- Verify environment variables are set correctly

**4. Environment Variables Not Loading**

**Solution:**
```bash
# Pull environment variables locally
netlify env:list

# Set missing variables
netlify env:set VARIABLE_NAME value

# Redeploy
netlify deploy --prod
```

**5. CORS Errors**

```
Access-Control-Allow-Origin error
```

**Solution:**
```bash
# Update soketi.rs CORS settings
SOKETI_CORS_ALLOWED_ORIGINS=https://your-site.netlify.app,https://deploy-preview-*--your-site.netlify.app
```

**6. Deploy Preview Issues**

**Solution:**
```toml
# Configure deploy previews in netlify.toml
[context.deploy-preview]
  environment = { NODE_ENV = "preview" }
  
[context.deploy-preview.environment]
  NEXT_PUBLIC_PUSHER_HOST = "preview-soketi-host.com"
```

### Debug Mode

Enable debug logging:

```typescript
// lib/pusher.ts
if (process.env.NODE_ENV !== 'production') {
  Pusher.logToConsole = true;
}

const pusher = new Pusher(key, {
  enabledTransports: ['ws', 'wss'],
  // ... other config
});
```

### Health Checks

Monitor your deployment:

```bash
# Check Netlify site
curl https://your-site.netlify.app/api/health

# Check soketi.rs server
curl https://your-soketi-host.render.com/health

# Check Netlify function
curl https://your-site.netlify.app/.netlify/functions/pusher-auth
```

### Netlify CLI Debugging

```bash
# Test functions locally
netlify dev

# View function logs
netlify functions:log pusher-auth

# Check site status
netlify status

# View recent deploys
netlify deploy:list
```

## Related Resources

- [Netlify Documentation](https://docs.netlify.com)
- [Netlify Functions Guide](https://docs.netlify.com/functions/overview/)
- [Next.js on Netlify](https://docs.netlify.com/integrations/frameworks/next-js/)
- [soketi.rs Documentation](https://docs.soketi.app)
- [Render Documentation](https://render.com/docs)
- [Vercel Deployment Guide](./vercel.md)
- [Reverse Proxy Setup](./reverse-proxy.md)
- [Getting Started Guide](../getting-started.md)
- [API Reference](../api-reference.md)
