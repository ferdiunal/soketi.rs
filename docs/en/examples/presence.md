# Presence Channels

> Real-time user presence tracking with soketi.rs and Pusher SDK

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Setup Instructions](#setup-instructions)
- [Server-Side Authentication](#server-side-authentication)
- [TypeScript Implementation](#typescript-implementation)
- [JavaScript Implementation](#javascript-implementation)
- [Expected Output](#expected-output)
- [Advanced Features](#advanced-features)

## Overview

Presence channels allow you to track which users are currently subscribed to a channel. This is perfect for features like:

- Showing who's online in a chat room
- Displaying active users in a collaborative document
- Building multiplayer game lobbies
- Creating "who's viewing" indicators

Unlike public channels, presence channels require authentication and provide member information to all subscribers.

## Prerequisites

- Node.js 18+ installed
- soketi.rs server running (see [Getting Started](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md))
- Express.js or similar backend framework for authentication
- Basic knowledge of JavaScript/TypeScript
- Understanding of [Basic Chat Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/basic-chat.md)

## Setup Instructions

### 1. Install Dependencies

**Client-side:**
```bash
npm install pusher-js
```

**Server-side:**
```bash
npm install express pusher cors body-parser
```

### 2. Configure soketi.rs Server

Start soketi.rs with client events enabled:

```bash
soketi start \
  --port=6001 \
  --app-id=app-id \
  --key=app-key \
  --secret=app-secret \
  --enable-client-messages
```

### 3. Create HTML File

Create an `presence.html` file:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Presence Channel - soketi.rs</title>
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
    <h1>Presence Channel Demo</h1>
    <div id="status" class="disconnected">Disconnected</div>
    
    <div class="container">
        <div class="chat-section">
            <h2>Chat</h2>
            <div id="messages"></div>
            <div id="input-container">
                <input type="text" id="message-input" placeholder="Type your message..." />
                <button id="send-button">Send</button>
            </div>
        </div>
        
        <div class="users-section">
            <h2>Online Users</h2>
            <div class="member-count">
                <span id="member-count">0</span> users online
            </div>
            <ul id="users-list"></ul>
        </div>
    </div>
    
    <script src="https://js.pusher.com/8.2.0/pusher.min.js"></script>
    <script src="presence.js"></script>
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

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

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

// Mock user database (replace with real authentication)
interface User {
  id: string;
  name: string;
  email: string;
}

const users: Record<string, User> = {
  'user-1': { id: 'user-1', name: 'Alice Johnson', email: 'alice@example.com' },
  'user-2': { id: 'user-2', name: 'Bob Smith', email: 'bob@example.com' },
  'user-3': { id: 'user-3', name: 'Charlie Brown', email: 'charlie@example.com' },
};

// Authentication endpoint for presence channels
app.post('/pusher/auth', (req: Request, res: Response) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  
  // Get user from session/token (simplified for demo)
  const userId = req.query.user_id as string || 'user-1';
  const user = users[userId];
  
  if (!user) {
    return res.status(403).json({ error: 'User not found' });
  }
  
  // Only authenticate presence channels
  if (!channelName.startsWith('presence-')) {
    return res.status(403).json({ error: 'Invalid channel' });
  }
  
  // Presence data that will be shared with other users
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
    console.error('Auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
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

const app = express();

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

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

// Mock user database (replace with real authentication)
const users = {
  'user-1': { id: 'user-1', name: 'Alice Johnson', email: 'alice@example.com' },
  'user-2': { id: 'user-2', name: 'Bob Smith', email: 'bob@example.com' },
  'user-3': { id: 'user-3', name: 'Charlie Brown', email: 'charlie@example.com' },
};

// Authentication endpoint for presence channels
app.post('/pusher/auth', (req, res) => {
  const socketId = req.body.socket_id;
  const channelName = req.body.channel_name;
  
  // Get user from session/token (simplified for demo)
  const userId = req.query.user_id || 'user-1';
  const user = users[userId];
  
  if (!user) {
    return res.status(403).json({ error: 'User not found' });
  }
  
  // Only authenticate presence channels
  if (!channelName.startsWith('presence-')) {
    return res.status(403).json({ error: 'Invalid channel' });
  }
  
  // Presence data that will be shared with other users
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
    console.error('Auth error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
});

// Start server
const PORT = 3000;
app.listen(PORT, () => {
  console.log(`Auth server running on http://localhost:${PORT}`);
});
```

## TypeScript Implementation

Create a `presence.ts` file:

```typescript
import Pusher, { PresenceChannel } from 'pusher-js';

// Configuration
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

// Simulate user ID (in production, get from authentication)
const currentUserId = `user-${Math.floor(Math.random() * 3) + 1}`;

// Initialize Pusher client
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1',
  disableStats: true,
  authEndpoint: `http://localhost:3000/pusher/auth?user_id=${currentUserId}`,
});

// Subscribe to presence channel
const channel = pusher.subscribe('presence-chat-room') as PresenceChannel;

// DOM elements
const messagesDiv = document.getElementById('messages') as HTMLDivElement;
const messageInput = document.getElementById('message-input') as HTMLInputElement;
const sendButton = document.getElementById('send-button') as HTMLButtonElement;
const statusDiv = document.getElementById('status') as HTMLDivElement;
const usersListDiv = document.getElementById('users-list') as HTMLUListElement;
const memberCountSpan = document.getElementById('member-count') as HTMLSpanElement;

// Connection state handlers
pusher.connection.bind('connected', () => {
  console.log('Connected to soketi.rs');
  statusDiv.textContent = 'Connected';
  statusDiv.className = 'connected';
});

pusher.connection.bind('disconnected', () => {
  console.log('Disconnected from soketi.rs');
  statusDiv.textContent = 'Disconnected';
  statusDiv.className = 'disconnected';
});

// Presence channel events
channel.bind('pusher:subscription_succeeded', (members: any) => {
  console.log('Successfully subscribed to presence channel');
  updateUsersList(members);
  
  displaySystemMessage(`You joined the room as ${members.me.info.name}`);
});

channel.bind('pusher:member_added', (member: PresenceMember) => {
  console.log('Member added:', member);
  addUserToList(member);
  displaySystemMessage(`${member.info.name} joined the room`);
});

channel.bind('pusher:member_removed', (member: PresenceMember) => {
  console.log('Member removed:', member);
  removeUserFromList(member.id);
  displaySystemMessage(`${member.info.name} left the room`);
});

// Chat message event
channel.bind('client-message', (data: Message) => {
  displayMessage(data);
});

// Update users list
function updateUsersList(members: any): void {
  usersListDiv.innerHTML = '';
  memberCountSpan.textContent = members.count.toString();
  
  members.each((member: PresenceMember) => {
    addUserToList(member);
  });
}

// Add user to list
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

// Remove user from list
function removeUserFromList(userId: string): void {
  const userEl = document.getElementById(`user-${userId}`);
  if (userEl) {
    userEl.remove();
    updateMemberCount();
  }
}

// Update member count
function updateMemberCount(): void {
  const count = usersListDiv.children.length;
  memberCountSpan.textContent = count.toString();
}

// Display chat message
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

// Escape HTML
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Send message
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

// Event listeners
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});
```

## JavaScript Implementation

Create a `presence.js` file:

```javascript
// Simulate user ID (in production, get from authentication)
const currentUserId = `user-${Math.floor(Math.random() * 3) + 1}`;

// Initialize Pusher client
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1',
  disableStats: true,
  authEndpoint: `http://localhost:3000/pusher/auth?user_id=${currentUserId}`,
});

// Subscribe to presence channel
const channel = pusher.subscribe('presence-chat-room');

// DOM elements
const messagesDiv = document.getElementById('messages');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const statusDiv = document.getElementById('status');
const usersListDiv = document.getElementById('users-list');
const memberCountSpan = document.getElementById('member-count');

// Connection state handlers
pusher.connection.bind('connected', () => {
  console.log('Connected to soketi.rs');
  statusDiv.textContent = 'Connected';
  statusDiv.className = 'connected';
});

pusher.connection.bind('disconnected', () => {
  console.log('Disconnected from soketi.rs');
  statusDiv.textContent = 'Disconnected';
  statusDiv.className = 'disconnected';
});

// Presence channel events
channel.bind('pusher:subscription_succeeded', (members) => {
  console.log('Successfully subscribed to presence channel');
  updateUsersList(members);
  
  displaySystemMessage(`You joined the room as ${members.me.info.name}`);
});

channel.bind('pusher:member_added', (member) => {
  console.log('Member added:', member);
  addUserToList(member);
  displaySystemMessage(`${member.info.name} joined the room`);
});

channel.bind('pusher:member_removed', (member) => {
  console.log('Member removed:', member);
  removeUserFromList(member.id);
  displaySystemMessage(`${member.info.name} left the room`);
});

// Chat message event
channel.bind('client-message', (data) => {
  displayMessage(data);
});

// Update users list
function updateUsersList(members) {
  usersListDiv.innerHTML = '';
  memberCountSpan.textContent = members.count;
  
  members.each((member) => {
    addUserToList(member);
  });
}

// Add user to list
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

// Remove user from list
function removeUserFromList(userId) {
  const userEl = document.getElementById(`user-${userId}`);
  if (userEl) {
    userEl.remove();
    updateMemberCount();
  }
}

// Update member count
function updateMemberCount() {
  const count = usersListDiv.children.length;
  memberCountSpan.textContent = count;
}

// Display chat message
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

// Escape HTML
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Send message
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

// Event listeners
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

### Browser Console

```
Connected to soketi.rs
Successfully subscribed to presence channel
Member added: { id: 'user-2', info: { name: 'Bob Smith', email: 'bob@example.com' } }
```

### Browser Display

**Status Bar:**
```
Connected
```

**Online Users Panel:**
```
Online Users
2 users online

● Alice Johnson
  alice@example.com

● Bob Smith
  bob@example.com
```

**Chat Messages:**
```
You joined the room as Alice Johnson
Bob Smith joined the room
Alice Johnson: Hello everyone!
Bob Smith: Hi Alice!
Charlie Brown joined the room
Charlie Brown: Hey folks!
Bob Smith left the room
```

## Advanced Features

### 1. Typing Indicators

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
  // Show typing indicator
  console.log(`${data.user} is typing...`);
});
```

### 2. User Status

```typescript
// Update user status
function updateUserStatus(status: 'online' | 'away' | 'busy') {
  channel.trigger('client-status-change', {
    userId: channel.members.me.id,
    status: status,
  });
}
```

### 3. Direct Messages

```typescript
// Send direct message to specific user
function sendDirectMessage(recipientId: string, message: string) {
  channel.trigger('client-direct-message', {
    from: channel.members.me.id,
    to: recipientId,
    message: message,
  });
}
```

## Related Documentation

- [Basic Chat Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/basic-chat.md)
- [Private Channels Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/private-channels.md)
- [Authentication Example](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/examples/authentication.md)
- [API Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/api-reference.md)
- [Getting Started Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md)
