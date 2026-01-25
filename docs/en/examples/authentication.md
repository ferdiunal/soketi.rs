# Authentication Examples

> Complete authentication patterns for soketi.rs applications

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Authentication Patterns](#authentication-patterns)
- [JWT-Based Authentication](#jwt-based-authentication)
- [Session-Based Authentication](#session-based-authentication)
- [OAuth Integration](#oauth-integration)
- [Best Practices](#best-practices)

## Overview

This guide covers various authentication patterns for securing your soketi.rs applications. We'll explore:

- JWT (JSON Web Token) authentication
- Session-based authentication
- OAuth 2.0 integration
- Multi-factor authentication (MFA)
- Token refresh strategies
- Security best practices

## Prerequisites

- Node.js 18+ installed
- soketi.rs server running
- Express.js or similar backend framework
- Understanding of [Private Channels](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/private-channels.md)
- Basic knowledge of authentication concepts

## Authentication Patterns

### Pattern 1: JWT Authentication

Best for:
- Stateless authentication
- Microservices architecture
- Mobile applications
- API-first applications

### Pattern 2: Session Authentication

Best for:
- Traditional web applications
- Server-side rendered apps
- Applications requiring server-side state

### Pattern 3: OAuth 2.0

Best for:
- Third-party integrations
- Social login
- Enterprise SSO
- Multi-tenant applications

## JWT-Based Authentication


### TypeScript Implementation

Create an `auth-jwt.ts` file:

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

// Configuration
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key-change-in-production';
const JWT_REFRESH_SECRET = process.env.JWT_REFRESH_SECRET || 'your-refresh-secret';
const ACCESS_TOKEN_EXPIRY = '15m';
const REFRESH_TOKEN_EXPIRY = '7d';

// Initialize Pusher
const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID || 'app-id',
  key: process.env.PUSHER_KEY || 'app-key',
  secret: process.env.PUSHER_SECRET || 'app-secret',
  cluster: process.env.PUSHER_CLUSTER || 'mt1',
  host: process.env.PUSHER_HOST || 'localhost',
  port: parseInt(process.env.PUSHER_PORT || '6001'),
  useTLS: process.env.PUSHER_USE_TLS === 'true',
});

// Types
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

// Mock user database (replace with real database)
const users: User[] = [
  {
    id: 'user-1',
    email: 'alice@example.com',
    password: '$2b$10$rBV2kHf7Yw3KxXqX5xXxXeX5xXxXxXxXxXxXxXxXxXxXxXxXxXxXx', // 'password123'
    name: 'Alice Johnson',
    role: 'admin',
  },
];

// Refresh token storage (use Redis in production)
const refreshTokens = new Set<string>();

// Helper: Generate tokens
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

// Middleware: Verify access token
function authenticateToken(req: Request, res: Response, next: NextFunction) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }
  
  jwt.verify(token, JWT_SECRET, (err: any, decoded: any) => {
    if (err) {
      if (err.name === 'TokenExpiredError') {
        return res.status(401).json({ error: 'Token expired' });
      }
      return res.status(403).json({ error: 'Invalid token' });
    }
    
    req.user = decoded as JWTPayload;
    next();
  });
}

// Route: Register
app.post('/auth/register', async (req: Request, res: Response) => {
  try {
    const { email, password, name } = req.body;
    
    // Validate input
    if (!email || !password || !name) {
      return res.status(400).json({ error: 'Missing required fields' });
    }
    
    // Check if user exists
    if (users.find(u => u.email === email)) {
      return res.status(409).json({ error: 'User already exists' });
    }
    
    // Hash password
    const hashedPassword = await bcrypt.hash(password, 10);
    
    // Create user
    const user: User = {
      id: `user-${Date.now()}`,
      email,
      password: hashedPassword,
      name,
      role: 'user',
    };
    
    users.push(user);
    
    // Generate tokens
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
    console.error('Register error:', error);
    res.status(500).json({ error: 'Registration failed' });
  }
});

// Route: Login
app.post('/auth/login', async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body;
    
    // Find user
    const user = users.find(u => u.email === email);
    if (!user) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
    // Verify password
    const validPassword = await bcrypt.compare(password, user.password);
    if (!validPassword) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
    // Generate tokens
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
    console.error('Login error:', error);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Route: Refresh token
app.post('/auth/refresh', (req: Request, res: Response) => {
  const { refreshToken } = req.body;
  
  if (!refreshToken) {
    return res.status(401).json({ error: 'No refresh token provided' });
  }
  
  if (!refreshTokens.has(refreshToken)) {
    return res.status(403).json({ error: 'Invalid refresh token' });
  }
  
  jwt.verify(refreshToken, JWT_REFRESH_SECRET, (err: any, decoded: any) => {
    if (err) {
      refreshTokens.delete(refreshToken);
      return res.status(403).json({ error: 'Invalid refresh token' });
    }
    
    const user = users.find(u => u.id === decoded.userId);
    if (!user) {
      return res.status(403).json({ error: 'User not found' });
    }
    
    // Generate new tokens
    refreshTokens.delete(refreshToken);
    const tokens = generateTokens(user);
    
    res.json(tokens);
  });
});

// Route: Logout
app.post('/auth/logout', authenticateToken, (req: Request, res: Response) => {
  const { refreshToken } = req.body;
  
  if (refreshToken) {
    refreshTokens.delete(refreshToken);
  }
  
  res.json({ message: 'Logged out successfully' });
});

// Route: Get current user
app.get('/auth/me', authenticateToken, (req: Request, res: Response) => {
  const user = users.find(u => u.id === req.user.userId);
  
  if (!user) {
    return res.status(404).json({ error: 'User not found' });
  }
  
  res.json({
    id: user.id,
    email: user.email,
    name: user.name,
    role: user.role,
  });
});

// Route: Pusher authentication
app.post('/pusher/auth', authenticateToken, (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Missing parameters' });
  }
  
  // Validate channel access based on user role
  if (channelName.startsWith('private-admin-') && user.role !== 'admin') {
    return res.status(403).json({ error: 'Admin access required' });
  }
  
  // Validate user-specific channels
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Access denied' });
  }
  
  try {
    let authResponse;
    
    // Presence channel
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
      // Private channel
      authResponse = pusher.authorizeChannel(socketId, channelName);
    }
    
    res.json(authResponse);
  } catch (error) {
    console.error('Pusher auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

// Start server
const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Auth server running on http://localhost:${PORT}`);
});
```

### JavaScript Implementation

Create an `auth-jwt.js` file:

```javascript
const express = require('express');
const Pusher = require('pusher');
const jwt = require('jsonwebtoken');
const bcrypt = require('bcrypt');
const cors = require('cors');
const bodyParser = require('body-parser');

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());

// Configuration
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key-change-in-production';
const JWT_REFRESH_SECRET = process.env.JWT_REFRESH_SECRET || 'your-refresh-secret';
const ACCESS_TOKEN_EXPIRY = '15m';
const REFRESH_TOKEN_EXPIRY = '7d';

// Initialize Pusher
const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID || 'app-id',
  key: process.env.PUSHER_KEY || 'app-key',
  secret: process.env.PUSHER_SECRET || 'app-secret',
  cluster: process.env.PUSHER_CLUSTER || 'mt1',
  host: process.env.PUSHER_HOST || 'localhost',
  port: parseInt(process.env.PUSHER_PORT || '6001'),
  useTLS: process.env.PUSHER_USE_TLS === 'true',
});

// Mock user database
const users = [];

// Refresh token storage
const refreshTokens = new Set();

// Helper: Generate tokens
function generateTokens(user) {
  const payload = {
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

// Middleware: Verify access token
function authenticateToken(req, res, next) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }
  
  jwt.verify(token, JWT_SECRET, (err, decoded) => {
    if (err) {
      if (err.name === 'TokenExpiredError') {
        return res.status(401).json({ error: 'Token expired' });
      }
      return res.status(403).json({ error: 'Invalid token' });
    }
    
    req.user = decoded;
    next();
  });
}

// Route: Register
app.post('/auth/register', async (req, res) => {
  try {
    const { email, password, name } = req.body;
    
    if (!email || !password || !name) {
      return res.status(400).json({ error: 'Missing required fields' });
    }
    
    if (users.find(u => u.email === email)) {
      return res.status(409).json({ error: 'User already exists' });
    }
    
    const hashedPassword = await bcrypt.hash(password, 10);
    
    const user = {
      id: `user-${Date.now()}`,
      email,
      password: hashedPassword,
      name,
      role: 'user',
    };
    
    users.push(user);
    
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
    console.error('Register error:', error);
    res.status(500).json({ error: 'Registration failed' });
  }
});

// Route: Login
app.post('/auth/login', async (req, res) => {
  try {
    const { email, password } = req.body;
    
    const user = users.find(u => u.email === email);
    if (!user) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
    const validPassword = await bcrypt.compare(password, user.password);
    if (!validPassword) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
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
    console.error('Login error:', error);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Route: Refresh token
app.post('/auth/refresh', (req, res) => {
  const { refreshToken } = req.body;
  
  if (!refreshToken || !refreshTokens.has(refreshToken)) {
    return res.status(403).json({ error: 'Invalid refresh token' });
  }
  
  jwt.verify(refreshToken, JWT_REFRESH_SECRET, (err, decoded) => {
    if (err) {
      refreshTokens.delete(refreshToken);
      return res.status(403).json({ error: 'Invalid refresh token' });
    }
    
    const user = users.find(u => u.id === decoded.userId);
    if (!user) {
      return res.status(403).json({ error: 'User not found' });
    }
    
    refreshTokens.delete(refreshToken);
    const tokens = generateTokens(user);
    
    res.json(tokens);
  });
});

// Route: Logout
app.post('/auth/logout', authenticateToken, (req, res) => {
  const { refreshToken } = req.body;
  
  if (refreshToken) {
    refreshTokens.delete(refreshToken);
  }
  
  res.json({ message: 'Logged out successfully' });
});

// Route: Pusher authentication
app.post('/pusher/auth', authenticateToken, (req, res) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Missing parameters' });
  }
  
  // Validate channel access
  if (channelName.startsWith('private-admin-') && user.role !== 'admin') {
    return res.status(403).json({ error: 'Admin access required' });
  }
  
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Access denied' });
  }
  
  try {
    let authResponse;
    
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
      authResponse = pusher.authorizeChannel(socketId, channelName);
    }
    
    res.json(authResponse);
  } catch (error) {
    console.error('Pusher auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

// Start server
const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Auth server running on http://localhost:${PORT}`);
});
```

## Session-Based Authentication


### TypeScript Implementation

Create an `auth-session.ts` file:

```typescript
import express, { Request, Response, NextFunction } from 'express';
import session from 'express-session';
import Pusher from 'pusher';
import bcrypt from 'bcrypt';
import cors from 'cors';
import bodyParser from 'body-parser';

const app = express();

// Middleware
app.use(cors({
  origin: 'http://localhost:8080',
  credentials: true,
}));
app.use(bodyParser.json());

// Session configuration
app.use(session({
  secret: process.env.SESSION_SECRET || 'your-session-secret',
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 24 * 60 * 60 * 1000, // 24 hours
  },
}));

// Initialize Pusher
const pusher = new Pusher({
  appId: process.env.PUSHER_APP_ID || 'app-id',
  key: process.env.PUSHER_KEY || 'app-key',
  secret: process.env.PUSHER_SECRET || 'app-secret',
  cluster: 'mt1',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Extend session type
declare module 'express-session' {
  interface SessionData {
    userId: string;
    email: string;
    role: string;
  }
}

// Middleware: Check authentication
function requireAuth(req: Request, res: Response, next: NextFunction) {
  if (!req.session.userId) {
    return res.status(401).json({ error: 'Not authenticated' });
  }
  next();
}

// Route: Login
app.post('/auth/login', async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body;
    
    // Verify credentials (simplified)
    // In production, query your database
    
    // Set session
    req.session.userId = 'user-1';
    req.session.email = email;
    req.session.role = 'user';
    
    res.json({
      user: {
        id: req.session.userId,
        email: req.session.email,
        role: req.session.role,
      },
    });
  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Route: Logout
app.post('/auth/logout', (req: Request, res: Response) => {
  req.session.destroy((err) => {
    if (err) {
      return res.status(500).json({ error: 'Logout failed' });
    }
    res.clearCookie('connect.sid');
    res.json({ message: 'Logged out successfully' });
  });
});

// Route: Pusher authentication
app.post('/pusher/auth', requireAuth, (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  
  try {
    let authResponse;
    
    if (channelName.startsWith('presence-')) {
      const presenceData = {
        user_id: req.session.userId!,
        user_info: {
          email: req.session.email!,
          role: req.session.role!,
        },
      };
      authResponse = pusher.authorizeChannel(socketId, channelName, presenceData);
    } else {
      authResponse = pusher.authorizeChannel(socketId, channelName);
    }
    
    res.json(authResponse);
  } catch (error) {
    console.error('Pusher auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

app.listen(3000, () => {
  console.log('Session auth server running on http://localhost:3000');
});
```

### JavaScript Implementation

Create an `auth-session.js` file:

```javascript
const express = require('express');
const session = require('express-session');
const Pusher = require('pusher');
const cors = require('cors');
const bodyParser = require('body-parser');

const app = express();

// Middleware
app.use(cors({
  origin: 'http://localhost:8080',
  credentials: true,
}));
app.use(bodyParser.json());

// Session configuration
app.use(session({
  secret: process.env.SESSION_SECRET || 'your-session-secret',
  resave: false,
  saveUninitialized: false,
  cookie: {
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    maxAge: 24 * 60 * 60 * 1000,
  },
}));

// Initialize Pusher
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  cluster: 'mt1',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Middleware: Check authentication
function requireAuth(req, res, next) {
  if (!req.session.userId) {
    return res.status(401).json({ error: 'Not authenticated' });
  }
  next();
}

// Route: Login
app.post('/auth/login', async (req, res) => {
  try {
    const { email, password } = req.body;
    
    // Set session
    req.session.userId = 'user-1';
    req.session.email = email;
    req.session.role = 'user';
    
    res.json({
      user: {
        id: req.session.userId,
        email: req.session.email,
        role: req.session.role,
      },
    });
  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Route: Logout
app.post('/auth/logout', (req, res) => {
  req.session.destroy((err) => {
    if (err) {
      return res.status(500).json({ error: 'Logout failed' });
    }
    res.clearCookie('connect.sid');
    res.json({ message: 'Logged out successfully' });
  });
});

// Route: Pusher authentication
app.post('/pusher/auth', requireAuth, (req, res) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  
  try {
    let authResponse;
    
    if (channelName.startsWith('presence-')) {
      const presenceData = {
        user_id: req.session.userId,
        user_info: {
          email: req.session.email,
          role: req.session.role,
        },
      };
      authResponse = pusher.authorizeChannel(socketId, channelName, presenceData);
    } else {
      authResponse = pusher.authorizeChannel(socketId, channelName);
    }
    
    res.json(authResponse);
  } catch (error) {
    console.error('Pusher auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

app.listen(3000, () => {
  console.log('Session auth server running on http://localhost:3000');
});
```

## OAuth Integration

### Google OAuth Example (TypeScript)

```typescript
import { OAuth2Client } from 'google-auth-library';

const client = new OAuth2Client(
  process.env.GOOGLE_CLIENT_ID,
  process.env.GOOGLE_CLIENT_SECRET,
  'http://localhost:3000/auth/google/callback'
);

// Route: Initiate OAuth
app.get('/auth/google', (req: Request, res: Response) => {
  const authorizeUrl = client.generateAuthUrl({
    access_type: 'offline',
    scope: ['profile', 'email'],
  });
  res.redirect(authorizeUrl);
});

// Route: OAuth callback
app.get('/auth/google/callback', async (req: Request, res: Response) => {
  try {
    const { code } = req.query;
    
    const { tokens } = await client.getToken(code as string);
    client.setCredentials(tokens);
    
    const ticket = await client.verifyIdToken({
      idToken: tokens.id_token!,
      audience: process.env.GOOGLE_CLIENT_ID,
    });
    
    const payload = ticket.getPayload();
    
    // Create or update user
    const user = {
      id: payload!.sub,
      email: payload!.email,
      name: payload!.name,
      picture: payload!.picture,
    };
    
    // Generate JWT tokens
    const authTokens = generateTokens(user);
    
    res.json({
      user,
      ...authTokens,
    });
  } catch (error) {
    console.error('OAuth error:', error);
    res.status(500).json({ error: 'OAuth authentication failed' });
  }
});
```

## Best Practices

### 1. Secure Token Storage

**Client-side (TypeScript):**
```typescript
class TokenManager {
  private static readonly ACCESS_TOKEN_KEY = 'access_token';
  private static readonly REFRESH_TOKEN_KEY = 'refresh_token';
  
  static setTokens(accessToken: string, refreshToken: string): void {
    // Use httpOnly cookies in production
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

### 2. Automatic Token Refresh

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
    // Refresh 1 minute before expiry (14 minutes for 15-minute tokens)
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
      console.error('Token refresh failed:', error);
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

### 3. Secure Pusher Client Configuration

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

### 4. Role-Based Access Control

```typescript
// Server-side
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
    return res.status(403).json({ error: 'Insufficient permissions' });
  }
  
  // Authorize channel...
});
```

### 5. Rate Limiting

```typescript
import rateLimit from 'express-rate-limit';

// Login rate limiter
const loginLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 5, // 5 attempts
  message: 'Too many login attempts, please try again later',
});

app.post('/auth/login', loginLimiter, async (req, res) => {
  // Login logic...
});

// Pusher auth rate limiter
const pusherAuthLimiter = rateLimit({
  windowMs: 1 * 60 * 1000, // 1 minute
  max: 30, // 30 requests
});

app.post('/pusher/auth', pusherAuthLimiter, authenticateToken, (req, res) => {
  // Auth logic...
});
```

### 6. Security Headers

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

## Expected Output

### JWT Authentication Flow

```
1. User registers/logs in
   POST /auth/login
   Response: { accessToken, refreshToken, user }

2. Client stores tokens
   localStorage.setItem('access_token', accessToken)

3. Client connects to Pusher with auth
   Authorization: Bearer <accessToken>

4. Token expires, client refreshes
   POST /auth/refresh
   Response: { accessToken, refreshToken }

5. User logs out
   POST /auth/logout
   Tokens cleared
```

### Console Output

```
Auth server running on http://localhost:3000
User logged in: alice@example.com
Pusher channel authorized: private-user-1
Token refreshed for user: alice@example.com
User logged out: alice@example.com
```

## Related Documentation

- [Basic Chat Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/basic-chat.md)
- [Private Channels Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/private-channels.md)
- [Presence Channels Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/presence.md)
- [API Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/api-reference.md)
- [Security Best Practices](../getting-started.md#security)
- [Configuration Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md)
