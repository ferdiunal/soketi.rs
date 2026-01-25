# Private Channels

> Secure channel authentication with soketi.rs and Pusher SDK

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Setup Instructions](#setup-instructions)
- [Server-Side Authentication](#server-side-authentication)
- [TypeScript Implementation](#typescript-implementation)
- [JavaScript Implementation](#javascript-implementation)
- [Expected Output](#expected-output)
- [Security Best Practices](#security-best-practices)

## Overview

Private channels provide secure, authenticated communication channels. Unlike public channels, private channels require server-side authentication before a client can subscribe. This is essential for:

- User-specific notifications
- Private messaging between users
- Secure data transmission
- Access-controlled features
- Multi-tenant applications

Private channels are prefixed with `private-` and require an authentication endpoint that validates subscription requests.

## Prerequisites

- Node.js 18+ installed
- soketi.rs server running (see [Getting Started](../getting-started.md))
- Express.js or similar backend framework
- Basic knowledge of JavaScript/TypeScript
- Understanding of [Basic Chat Example](./basic-chat.md)

## Setup Instructions

### 1. Install Dependencies

**Client-side:**
```bash
npm install pusher-js
```

**Server-side:**
```bash
npm install express pusher cors body-parser jsonwebtoken
```

### 2. Configure soketi.rs Server

Start soketi.rs:

```bash
soketi start \
  --port=6001 \
  --app-id=app-id \
  --key=app-key \
  --secret=app-secret
```

### 3. Create HTML File

Create a `private-channel.html` file:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Private Channel - soketi.rs</title>
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
    <h1>Private Channel Demo</h1>
    
    <div id="auth-section" class="auth-section">
        <h2>Login</h2>
        <input type="text" id="username" placeholder="Username" value="alice" />
        <input type="password" id="password" placeholder="Password" value="password123" />
        <button id="login-button">Login</button>
    </div>
    
    <div id="chat-section" class="hidden">
        <div id="status" class="disconnected">Disconnected</div>
        <h2>Private Messages</h2>
        <div id="messages"></div>
        <div style="display: flex; gap: 10px;">
            <input type="text" id="message-input" placeholder="Type your message..." style="flex: 1;" />
            <button id="send-button" style="width: auto;">Send</button>
        </div>
    </div>
    
    <script src="https://js.pusher.com/8.2.0/pusher.min.js"></script>
    <script src="private-channel.js"></script>
</body>
</html>
```

## Server-Side Authentication


### TypeScript Server Implementation

Create an `auth-server.ts` file:

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

// Configuration
const JWT_SECRET = 'your-secret-key-change-in-production';

// Initialize Pusher server SDK
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  cluster: 'mt1',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Mock user database
interface User {
  id: string;
  username: string;
  password: string;
  name: string;
}

const users: User[] = [
  { id: 'user-1', username: 'alice', password: 'password123', name: 'Alice Johnson' },
  { id: 'user-2', username: 'bob', password: 'password123', name: 'Bob Smith' },
  { id: 'user-3', username: 'charlie', password: 'password123', name: 'Charlie Brown' },
];

// Login endpoint
app.post('/login', (req: Request, res: Response) => {
  const { username, password } = req.body;
  
  const user = users.find(u => u.username === username && u.password === password);
  
  if (!user) {
    return res.status(401).json({ error: 'Invalid credentials' });
  }
  
  // Generate JWT token
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

// Middleware to verify JWT token
function authenticateToken(req: Request, res: Response, next: Function) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }
  
  jwt.verify(token, JWT_SECRET, (err: any, decoded: any) => {
    if (err) {
      return res.status(403).json({ error: 'Invalid token' });
    }
    
    req.user = decoded;
    next();
  });
}

// Pusher authentication endpoint
app.post('/pusher/auth', authenticateToken, (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Missing parameters' });
  }
  
  // Verify channel access
  if (!channelName.startsWith('private-')) {
    return res.status(403).json({ error: 'Invalid channel type' });
  }
  
  // Check if user has access to this specific channel
  // Example: private-user-{userId}
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Access denied to this channel' });
  }
  
  try {
    const authResponse = pusher.authorizeChannel(socketId, channelName);
    res.json(authResponse);
  } catch (error) {
    console.error('Auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

// Trigger event endpoint (for server-side events)
app.post('/pusher/trigger', authenticateToken, async (req: Request, res: Response) => {
  const { channel, event, data } = req.body;
  
  try {
    await pusher.trigger(channel, event, data);
    res.json({ success: true });
  } catch (error) {
    console.error('Trigger error:', error);
    res.status(500).json({ error: 'Failed to trigger event' });
  }
});

// Start server
const PORT = 3000;
app.listen(PORT, () => {
  console.log(`Auth server running on http://localhost:${PORT}`);
});
```

### JavaScript Server Implementation

Create an `auth-server.js` file:

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

// Configuration
const JWT_SECRET = 'your-secret-key-change-in-production';

// Initialize Pusher server SDK
const pusher = new Pusher({
  appId: 'app-id',
  key: 'app-key',
  secret: 'app-secret',
  cluster: 'mt1',
  host: 'localhost',
  port: 6001,
  useTLS: false,
});

// Mock user database
const users = [
  { id: 'user-1', username: 'alice', password: 'password123', name: 'Alice Johnson' },
  { id: 'user-2', username: 'bob', password: 'password123', name: 'Bob Smith' },
  { id: 'user-3', username: 'charlie', password: 'password123', name: 'Charlie Brown' },
];

// Login endpoint
app.post('/login', (req, res) => {
  const { username, password } = req.body;
  
  const user = users.find(u => u.username === username && u.password === password);
  
  if (!user) {
    return res.status(401).json({ error: 'Invalid credentials' });
  }
  
  // Generate JWT token
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

// Middleware to verify JWT token
function authenticateToken(req, res, next) {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1];
  
  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }
  
  jwt.verify(token, JWT_SECRET, (err, decoded) => {
    if (err) {
      return res.status(403).json({ error: 'Invalid token' });
    }
    
    req.user = decoded;
    next();
  });
}

// Pusher authentication endpoint
app.post('/pusher/auth', authenticateToken, (req, res) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  const user = req.user;
  
  if (!socketId || !channelName) {
    return res.status(400).json({ error: 'Missing parameters' });
  }
  
  // Verify channel access
  if (!channelName.startsWith('private-')) {
    return res.status(403).json({ error: 'Invalid channel type' });
  }
  
  // Check if user has access to this specific channel
  if (channelName.includes('user-') && !channelName.includes(user.userId)) {
    return res.status(403).json({ error: 'Access denied to this channel' });
  }
  
  try {
    const authResponse = pusher.authorizeChannel(socketId, channelName);
    res.json(authResponse);
  } catch (error) {
    console.error('Auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

// Trigger event endpoint
app.post('/pusher/trigger', authenticateToken, async (req, res) => {
  const { channel, event, data } = req.body;
  
  try {
    await pusher.trigger(channel, event, data);
    res.json({ success: true });
  } catch (error) {
    console.error('Trigger error:', error);
    res.status(500).json({ error: 'Failed to trigger event' });
  }
});

// Start server
const PORT = 3000;
app.listen(PORT, () => {
  console.log(`Auth server running on http://localhost:${PORT}`);
});
```

## TypeScript Implementation


Create a `private-channel.ts` file:

```typescript
import Pusher from 'pusher-js';

// Configuration
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

// State
let authToken: string | null = null;
let currentUser: AuthResponse['user'] | null = null;
let pusher: Pusher | null = null;
let channel: any = null;

// DOM elements
const authSection = document.getElementById('auth-section') as HTMLDivElement;
const chatSection = document.getElementById('chat-section') as HTMLDivElement;
const usernameInput = document.getElementById('username') as HTMLInputElement;
const passwordInput = document.getElementById('password') as HTMLInputElement;
const loginButton = document.getElementById('login-button') as HTMLButtonElement;
const messagesDiv = document.getElementById('messages') as HTMLDivElement;
const messageInput = document.getElementById('message-input') as HTMLInputElement;
const sendButton = document.getElementById('send-button') as HTMLButtonElement;
const statusDiv = document.getElementById('status') as HTMLDivElement;

// Login function
async function login(): Promise<void> {
  const username = usernameInput.value.trim();
  const password = passwordInput.value.trim();
  
  if (!username || !password) {
    displayErrorMessage('Please enter username and password');
    return;
  }
  
  try {
    loginButton.disabled = true;
    loginButton.textContent = 'Logging in...';
    
    const response = await fetch('http://localhost:3000/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });
    
    if (!response.ok) {
      throw new Error('Login failed');
    }
    
    const data: AuthResponse = await response.json();
    authToken = data.token;
    currentUser = data.user;
    
    // Hide auth section, show chat
    authSection.classList.add('hidden');
    chatSection.classList.remove('hidden');
    
    // Initialize Pusher
    initializePusher();
    
  } catch (error) {
    console.error('Login error:', error);
    displayErrorMessage('Login failed. Please check your credentials.');
    loginButton.disabled = false;
    loginButton.textContent = 'Login';
  }
}

// Initialize Pusher with authentication
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
  
  // Connection state handlers
  pusher.connection.bind('connected', () => {
    console.log('Connected to soketi.rs');
    statusDiv.textContent = 'Connected';
    statusDiv.className = 'connected';
    
    // Subscribe to private channel
    subscribeToPrivateChannel();
  });
  
  pusher.connection.bind('disconnected', () => {
    console.log('Disconnected from soketi.rs');
    statusDiv.textContent = 'Disconnected';
    statusDiv.className = 'disconnected';
  });
  
  pusher.connection.bind('error', (err: any) => {
    console.error('Connection error:', err);
    statusDiv.textContent = 'Connection Error';
    statusDiv.className = 'disconnected';
  });
}

// Subscribe to private channel
function subscribeToPrivateChannel(): void {
  if (!pusher || !currentUser) return;
  
  // Subscribe to user-specific private channel
  const channelName = `private-user-${currentUser.id}`;
  channel = pusher.subscribe(channelName);
  
  channel.bind('pusher:subscription_succeeded', () => {
    console.log(`Successfully subscribed to ${channelName}`);
    displaySystemMessage(`Connected to your private channel`);
  });
  
  channel.bind('pusher:subscription_error', (status: any) => {
    console.error('Subscription error:', status);
    displayErrorMessage('Failed to subscribe to private channel');
  });
  
  // Listen for messages
  channel.bind('message', (data: Message) => {
    displayMessage(data);
  });
  
  // Listen for notifications
  channel.bind('notification', (data: any) => {
    displaySystemMessage(`Notification: ${data.text}`);
  });
}

// Display message
function displayMessage(data: Message): void {
  const messageEl = document.createElement('div');
  messageEl.className = 'message';
  
  const time = new Date(data.timestamp).toLocaleTimeString();
  
  messageEl.innerHTML = `
    <span class="message-user">${data.user}:</span>
    <span class="message-text">${escapeHtml(data.text)}</span>
    <span class="message-time">${time}</span>
  `;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Display system message
function displaySystemMessage(text: string): void {
  const messageEl = document.createElement('div');
  messageEl.className = 'message system-message';
  messageEl.textContent = text;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Display error message
function displayErrorMessage(text: string): void {
  const messageEl = document.createElement('div');
  messageEl.className = 'message error-message';
  messageEl.textContent = `Error: ${text}`;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Escape HTML
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Send message via server
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
      throw new Error('Failed to send message');
    }
    
    messageInput.value = '';
    
  } catch (error) {
    console.error('Send error:', error);
    displayErrorMessage('Failed to send message');
  }
}

// Event listeners
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

## JavaScript Implementation

Create a `private-channel.js` file:

```javascript
// State
let authToken = null;
let currentUser = null;
let pusher = null;
let channel = null;

// DOM elements
const authSection = document.getElementById('auth-section');
const chatSection = document.getElementById('chat-section');
const usernameInput = document.getElementById('username');
const passwordInput = document.getElementById('password');
const loginButton = document.getElementById('login-button');
const messagesDiv = document.getElementById('messages');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const statusDiv = document.getElementById('status');

// Login function
async function login() {
  const username = usernameInput.value.trim();
  const password = passwordInput.value.trim();
  
  if (!username || !password) {
    displayErrorMessage('Please enter username and password');
    return;
  }
  
  try {
    loginButton.disabled = true;
    loginButton.textContent = 'Logging in...';
    
    const response = await fetch('http://localhost:3000/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username, password }),
    });
    
    if (!response.ok) {
      throw new Error('Login failed');
    }
    
    const data = await response.json();
    authToken = data.token;
    currentUser = data.user;
    
    // Hide auth section, show chat
    authSection.classList.add('hidden');
    chatSection.classList.remove('hidden');
    
    // Initialize Pusher
    initializePusher();
    
  } catch (error) {
    console.error('Login error:', error);
    displayErrorMessage('Login failed. Please check your credentials.');
    loginButton.disabled = false;
    loginButton.textContent = 'Login';
  }
}

// Initialize Pusher with authentication
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
  
  // Connection state handlers
  pusher.connection.bind('connected', () => {
    console.log('Connected to soketi.rs');
    statusDiv.textContent = 'Connected';
    statusDiv.className = 'connected';
    
    // Subscribe to private channel
    subscribeToPrivateChannel();
  });
  
  pusher.connection.bind('disconnected', () => {
    console.log('Disconnected from soketi.rs');
    statusDiv.textContent = 'Disconnected';
    statusDiv.className = 'disconnected';
  });
  
  pusher.connection.bind('error', (err) => {
    console.error('Connection error:', err);
    statusDiv.textContent = 'Connection Error';
    statusDiv.className = 'disconnected';
  });
}

// Subscribe to private channel
function subscribeToPrivateChannel() {
  if (!pusher || !currentUser) return;
  
  // Subscribe to user-specific private channel
  const channelName = `private-user-${currentUser.id}`;
  channel = pusher.subscribe(channelName);
  
  channel.bind('pusher:subscription_succeeded', () => {
    console.log(`Successfully subscribed to ${channelName}`);
    displaySystemMessage(`Connected to your private channel`);
  });
  
  channel.bind('pusher:subscription_error', (status) => {
    console.error('Subscription error:', status);
    displayErrorMessage('Failed to subscribe to private channel');
  });
  
  // Listen for messages
  channel.bind('message', (data) => {
    displayMessage(data);
  });
  
  // Listen for notifications
  channel.bind('notification', (data) => {
    displaySystemMessage(`Notification: ${data.text}`);
  });
}

// Display message
function displayMessage(data) {
  const messageEl = document.createElement('div');
  messageEl.className = 'message';
  
  const time = new Date(data.timestamp).toLocaleTimeString();
  
  messageEl.innerHTML = `
    <span class="message-user">${data.user}:</span>
    <span class="message-text">${escapeHtml(data.text)}</span>
    <span class="message-time">${time}</span>
  `;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Display system message
function displaySystemMessage(text) {
  const messageEl = document.createElement('div');
  messageEl.className = 'message system-message';
  messageEl.textContent = text;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Display error message
function displayErrorMessage(text) {
  const messageEl = document.createElement('div');
  messageEl.className = 'message error-message';
  messageEl.textContent = `Error: ${text}`;
  
  messagesDiv.appendChild(messageEl);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Escape HTML
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Send message via server
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
      throw new Error('Failed to send message');
    }
    
    messageInput.value = '';
    
  } catch (error) {
    console.error('Send error:', error);
    displayErrorMessage('Failed to send message');
  }
}

// Event listeners
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

## Expected Output

### Server Console

```
Auth server running on http://localhost:3000
```

### Browser Console (After Login)

```
Connected to soketi.rs
Successfully subscribed to private-user-1
```

### Browser Display

**Login Screen:**
```
Login
Username: [alice]
Password: [••••••••••]
[Login Button]
```

**After Login:**
```
Status: Connected

Private Messages
┌─────────────────────────────────────┐
│ Connected to your private channel   │
│ Alice Johnson: Hello!                │
│ Notification: You have a new message │
└─────────────────────────────────────┘
[Type your message...] [Send]
```

## Security Best Practices

### 1. Use HTTPS in Production

```typescript
const pusher = new Pusher('app-key', {
  wsHost: 'your-domain.com',
  forceTLS: true,  // Always use TLS in production
  encrypted: true,
});
```

### 2. Implement Proper Token Management

```typescript
// Store token securely
localStorage.setItem('authToken', token);

// Clear token on logout
function logout() {
  localStorage.removeItem('authToken');
  if (pusher) {
    pusher.disconnect();
  }
}
```

### 3. Validate Channel Access

```typescript
// Server-side: Check user permissions
app.post('/pusher/auth', authenticateToken, (req, res) => {
  const { channel_name } = req.body;
  const userId = req.user.userId;
  
  // Verify user has access to this channel
  if (!hasChannelAccess(userId, channel_name)) {
    return res.status(403).json({ error: 'Access denied' });
  }
  
  // Authorize
  const authResponse = pusher.authorizeChannel(socketId, channel_name);
  res.json(authResponse);
});
```

### 4. Rate Limiting

```typescript
// Add rate limiting to prevent abuse
const rateLimit = require('express-rate-limit');

const authLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Limit each IP to 100 requests per windowMs
});

app.post('/pusher/auth', authLimiter, authenticateToken, (req, res) => {
  // ... auth logic
});
```

### 5. Input Validation

```typescript
// Validate and sanitize all inputs
function validateChannelName(channelName: string): boolean {
  // Only allow alphanumeric, hyphens, and underscores
  return /^private-[a-zA-Z0-9_-]+$/.test(channelName);
}
```

## Related Documentation

- [Basic Chat Example](./basic-chat.md)
- [Presence Channels Example](./presence.md)
- [Authentication Example](./authentication.md)
- [API Reference](../api-reference.md)
- [Security Best Practices](../getting-started.md#security)
