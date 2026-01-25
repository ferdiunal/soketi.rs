# Quick Start Guide

## 🚀 Get Started in 3 Steps

### 1. Clone & Start

```bash
git clone https://github.com/ferdiunal/soketi.rs.git
cd soketi-rs
docker-compose up -d
```

### 2. Open Demo

```bash
open http://localhost:3000
```

### 3. Start Coding

```javascript
// Connect to Soketi
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false
});

// Subscribe to channel
const channel = pusher.subscribe('my-channel');

// Listen for events
channel.bind('my-event', (data) => {
  console.log('Received:', data);
});
```

## 📚 Next Steps

- [Getting Started](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/getting-started.md)
- [Installation](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/installation.md)
- [Configuration](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/configuration.md)
- [API Reference](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/api-reference.md)
- [Deployment Guide](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/deployment.md)

## 🎯 Common Use Cases

### Real-time Chat
```javascript
channel.bind('message', (data) => {
  displayMessage(data.user, data.message);
});
```

### Live Notifications
```javascript
channel.bind('notification', (data) => {
  showNotification(data.title, data.body);
});
```

### Presence Tracking
```javascript
const presence = pusher.subscribe('presence-room');
presence.bind('pusher:member_added', (member) => {
  console.log('User joined:', member.info.name);
});
```

## 🔧 Configuration

Edit `config.json`:

```json
{
  "host": "0.0.0.0",
  "port": 6001,
  "app_manager": {
    "driver": "Array",
    "array": {
      "apps": [{
        "id": "app-id",
        "key": "app-key",
        "secret": "app-secret"
      }]
    }
  }
}
```

## 📞 Need Help?

- [GitHub Issues](https://github.com/ferdiunal/soketi.rs/issues)
- [Troubleshooting](https://github.com/ferdiunal/soketi.rs/blob/main/docs/en/troubleshooting.md)
