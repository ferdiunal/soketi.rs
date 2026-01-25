# Basic Chat Application

> A simple real-time chat application using soketi.rs and Pusher SDK

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Setup Instructions](#setup-instructions)
- [TypeScript Implementation](#typescript-implementation)
- [JavaScript Implementation](#javascript-implementation)
- [Expected Output](#expected-output)
- [Next Steps](#next-steps)

## Overview

This example demonstrates how to build a basic real-time chat application using soketi.rs as the WebSocket server and the Pusher JavaScript SDK for client-side communication. Users can send and receive messages in real-time through a public channel.

## Prerequisites

- Node.js 18+ installed
- soketi.rs server running (see [Getting Started](../getting-started.md))
- Basic knowledge of JavaScript/TypeScript
- A text editor or IDE

## Setup Instructions

### 1. Install Dependencies

```bash
npm install pusher-js
# or
yarn add pusher-js
```

### 2. Configure soketi.rs Server

Ensure your soketi.rs server is running with the following configuration:

```bash
# Start soketi.rs with default settings
soketi start --port=6001 --app-id=app-id --key=app-key --secret=app-secret
```

### 3. Create HTML File

Create an `index.html` file:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Basic Chat - soketi.rs</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
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
            padding: 5px;
            background: white;
            border-radius: 3px;
        }
        .message-user {
            font-weight: bold;
            color: #0066cc;
        }
        .message-time {
            font-size: 0.8em;
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
        #send-button {
            padding: 10px 20px;
            background: #0066cc;
            color: white;
            border: none;
            border-radius: 3px;
            cursor: pointer;
        }
        #send-button:hover {
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
    </style>
</head>
<body>
    <h1>Basic Chat Application</h1>
    <div id="status" class="disconnected">Disconnected</div>
    <div id="messages"></div>
    <div id="input-container">
        <input type="text" id="message-input" placeholder="Type your message..." />
        <button id="send-button">Send</button>
    </div>
    <script src="https://js.pusher.com/8.2.0/pusher.min.js"></script>
    <script src="chat.js"></script>
</body>
</html>
```

## TypeScript Implementation

Create a `chat.ts` file:

```typescript
import Pusher from 'pusher-js';

// Configuration
interface ChatConfig {
  appKey: string;
  wsHost: string;
  wsPort: number;
  forceTLS: boolean;
  enabledTransports: string[];
}

interface Message {
  user: string;
  text: string;
  timestamp: string;
}

// Initialize Pusher client
const config: ChatConfig = {
  appKey: 'app-key',
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
};

const pusher = new Pusher(config.appKey, {
  wsHost: config.wsHost,
  wsPort: config.wsPort,
  forceTLS: config.forceTLS,
  enabledTransports: config.enabledTransports,
  cluster: 'mt1',
  disableStats: true,
});

// Subscribe to a public channel
const channel = pusher.subscribe('chat-room');

// DOM elements
const messagesDiv = document.getElementById('messages') as HTMLDivElement;
const messageInput = document.getElementById('message-input') as HTMLInputElement;
const sendButton = document.getElementById('send-button') as HTMLButtonElement;
const statusDiv = document.getElementById('status') as HTMLDivElement;

// Generate a random username
const username = `User${Math.floor(Math.random() * 1000)}`;

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

pusher.connection.bind('error', (err: any) => {
  console.error('Connection error:', err);
  statusDiv.textContent = 'Connection Error';
  statusDiv.className = 'disconnected';
});

// Listen for messages
channel.bind('message', (data: Message) => {
  displayMessage(data);
});

// Display message in the UI
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

// Escape HTML to prevent XSS
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Send message
function sendMessage(): void {
  const text = messageInput.value.trim();
  
  if (!text) {
    return;
  }
  
  const message: Message = {
    user: username,
    text: text,
    timestamp: new Date().toISOString(),
  };
  
  // Trigger client event (requires client events to be enabled)
  channel.trigger('client-message', message);
  
  // Clear input
  messageInput.value = '';
}

// Event listeners
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Display welcome message
setTimeout(() => {
  displayMessage({
    user: 'System',
    text: `Welcome ${username}! Start chatting...`,
    timestamp: new Date().toISOString(),
  });
}, 500);
```

### Compile TypeScript

```bash
# Install TypeScript if not already installed
npm install -g typescript

# Compile
tsc chat.ts --target ES2015 --module ES2015 --lib ES2015,DOM
```

## JavaScript Implementation

Create a `chat.js` file:

```javascript
// Initialize Pusher client
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false,
  enabledTransports: ['ws', 'wss'],
  cluster: 'mt1',
  disableStats: true,
});

// Subscribe to a public channel
const channel = pusher.subscribe('chat-room');

// DOM elements
const messagesDiv = document.getElementById('messages');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const statusDiv = document.getElementById('status');

// Generate a random username
const username = `User${Math.floor(Math.random() * 1000)}`;

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

pusher.connection.bind('error', (err) => {
  console.error('Connection error:', err);
  statusDiv.textContent = 'Connection Error';
  statusDiv.className = 'disconnected';
});

// Listen for messages
channel.bind('message', (data) => {
  displayMessage(data);
});

// Display message in the UI
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

// Escape HTML to prevent XSS
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Send message
function sendMessage() {
  const text = messageInput.value.trim();
  
  if (!text) {
    return;
  }
  
  const message = {
    user: username,
    text: text,
    timestamp: new Date().toISOString(),
  };
  
  // Trigger client event (requires client events to be enabled)
  channel.trigger('client-message', message);
  
  // Clear input
  messageInput.value = '';
}

// Event listeners
sendButton.addEventListener('click', sendMessage);

messageInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Display welcome message
setTimeout(() => {
  displayMessage({
    user: 'System',
    text: `Welcome ${username}! Start chatting...`,
    timestamp: new Date().toISOString(),
  });
}, 500);
```

## Expected Output

### Console Output

When you open the application in your browser, you should see:

```
Connected to soketi.rs
```

### Browser Display

1. **Status Bar**: Shows "Connected" in green when successfully connected to soketi.rs
2. **Messages Area**: Displays all chat messages with username, text, and timestamp
3. **Welcome Message**: "System: Welcome User123! Start chatting..."
4. **Input Field**: Text input for typing messages
5. **Send Button**: Button to send messages

### Message Flow

1. User types a message and clicks "Send" or presses Enter
2. Message is sent to the soketi.rs server via the `chat-room` channel
3. All connected clients receive the message in real-time
4. Message appears in the messages area with username and timestamp

### Example Messages

```
System: Welcome User123! Start chatting... 10:30:15 AM
User123: Hello everyone! 10:30:20 AM
User456: Hi there! 10:30:25 AM
User123: How is everyone doing? 10:30:30 AM
```

## Next Steps

- **Add Private Channels**: Learn how to implement private channels with authentication in [Private Channels Example](./private-channels.md)
- **Add Presence**: See who's online with [Presence Channels Example](./presence.md)
- **Add Authentication**: Secure your chat with [Authentication Example](./authentication.md)
- **Server-Side Events**: Learn how to trigger events from your backend
- **Message History**: Implement message persistence with a database
- **User Profiles**: Add avatars and user profiles
- **Typing Indicators**: Show when users are typing
- **File Sharing**: Allow users to share images and files

## Related Documentation

- [Getting Started Guide](../getting-started.md)
- [API Reference](../api-reference.md)
- [Configuration Guide](../configuration.md)
- [Troubleshooting](../troubleshooting.md)
