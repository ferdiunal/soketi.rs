# Hızlı Başlangıç Rehberi

## 🚀 3 Adımda Başlayın

### 1. Klonlayın & Başlatın

```bash
git clone https://github.com/ferdiunal/soketi-rs.git
cd soketi-rs
docker-compose up -d
```

### 2. Demo'yu Açın

```bash
open http://localhost:3000
```

### 3. Kodlamaya Başlayın

```javascript
// Soketi'ye bağlanın
const pusher = new Pusher('app-key', {
  wsHost: 'localhost',
  wsPort: 6001,
  forceTLS: false
});

// Kanala abone olun
const channel = pusher.subscribe('my-channel');

// Olayları dinleyin
channel.bind('my-event', (data) => {
  console.log('Alındı:', data);
});
```

## 📚 Sonraki Adımlar

- [Başlangıç](baslangic.md)
- [Kurulum](kurulum.md)
- [Yapılandırma](yapilandirma.md)
- [API Referansı](api-referans.md)
- [Deployment Rehberi](docker-deployment.md)

## 🎯 Yaygın Kullanım Senaryoları

### Gerçek Zamanlı Sohbet
```javascript
channel.bind('message', (data) => {
  displayMessage(data.user, data.message);
});
```

### Canlı Bildirimler
```javascript
channel.bind('notification', (data) => {
  showNotification(data.title, data.body);
});
```

### Kullanıcı Takibi (Presence)
```javascript
const presence = pusher.subscribe('presence-room');
presence.bind('pusher:member_added', (member) => {
  console.log('Kullanıcı katıldı:', member.info.name);
});
```

## 🔧 Yapılandırma

`config.json` dosyasını düzenleyin:

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

## 📞 Yardıma mı İhtiyacınız Var?

- [GitHub Issues](https://github.com/ferdiunal/soketi-rs/issues)
- [Sorun Giderme](sorun-giderme.md)
