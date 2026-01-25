# Kurulum Kılavuzu

> Farklı platformlar ve ortamlar için Soketi'nin kapsamlı kurulum talimatları.

## İçindekiler

- [Sistem Gereksinimleri](#sistem-gereksinimleri)
- [Docker Kurulumu](#docker-kurulumu)
- [Kaynak Koddan Derleme](#kaynak-koddan-derleme)
- [Önceden Derlenmiş Binary'ler](#önceden-derlenmiş-binaryler)
- [Platforma Özel Talimatlar](#platforma-özel-talimatlar)
- [Doğrulama](#doğrulama)

## Sistem Gereksinimleri

- **İşletim Sistemi**: Linux, macOS veya Windows
- **Bellek**: Minimum 512MB RAM (production için 2GB+ önerilir)
- **CPU**: 1+ çekirdek (production için 2+ çekirdek önerilir)
- **Ağ**: Açık portlar 6001 (WebSocket) ve 9601 (Metrikler)

Kaynak koddan derleme için:
- **Rust**: 1.70 veya üzeri
- **Cargo**: En son sürüm

## Docker Kurulumu

Docker, çoğu kullanım durumu için Soketi'yi çalıştırmanın önerilen yoludur.

### Image'ı İndirme

```bash
docker pull quay.io/soketi/soketi:latest-16-alpine
```

### Varsayılan Yapılandırma ile Çalıştırma

```bash
docker run -p 6001:6001 -p 9601:9601 \
  -e SOKETI_DEFAULT_APP_ID=app-id \
  -e SOKETI_DEFAULT_APP_KEY=app-key \
  -e SOKETI_DEFAULT_APP_SECRET=app-secret \
  quay.io/soketi/soketi:latest-16-alpine
```

### Özel Yapılandırma ile Çalıştırma

Bir `config.json` dosyası oluşturun ve mount edin:

```bash
docker run -p 6001:6001 -p 9601:9601 \
  -v $(pwd)/config.json:/app/config.json \
  quay.io/soketi/soketi:latest-16-alpine \
  --config /app/config.json
```

### Docker Compose Kullanma

Bir `docker-compose.yml` oluşturun:

```yaml
version: '3.8'

services:
  soketi:
    image: quay.io/soketi/soketi:latest-16-alpine
    ports:
      - "6001:6001"
      - "9601:9601"
    environment:
      - SOKETI_DEFAULT_APP_ID=app-id
      - SOKETI_DEFAULT_APP_KEY=app-key
      - SOKETI_DEFAULT_APP_SECRET=app-secret
    volumes:
      - ./config.json:/app/config.json
    command: --config /app/config.json
```

Başlatmak için:

```bash
docker-compose up -d
```

## Kaynak Koddan Derleme

### Ön Gereksinimler

Rust ve Cargo'yu yükleyin:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Klonlama ve Derleme

```bash
# Depoyu klonlayın
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs

# Release modunda derleyin
cargo build --release

# Binary target/release/soketi konumunda olacak
```

### Binary'yi Çalıştırma

```bash
./target/release/soketi --config config.json
```

### Sistem Genelinde Kurulum (Opsiyonel)

```bash
sudo cp target/release/soketi /usr/local/bin/
soketi --version
```

## Önceden Derlenmiş Binary'ler

[GitHub Sürümler sayfasından](https://github.com/soketi/soketi.rs/releases) önceden derlenmiş binary'leri indirin.

### Linux

```bash
# En son sürümü indirin
wget https://github.com/soketi/soketi.rs/releases/latest/download/soketi-linux-x64

# Çalıştırılabilir yapın
chmod +x soketi-linux-x64

# Çalıştırın
./soketi-linux-x64 --config config.json
```

### macOS

```bash
# En son sürümü indirin
curl -L https://github.com/soketi/soketi.rs/releases/latest/download/soketi-macos-x64 -o soketi

# Çalıştırılabilir yapın
chmod +x soketi

# Çalıştırın
./soketi --config config.json
```

### Windows

Sürümler sayfasından `soketi-windows-x64.exe` dosyasını indirin ve Command Prompt veya PowerShell'den çalıştırın:

```powershell
.\soketi-windows-x64.exe --config config.json
```

## Platforma Özel Talimatlar

### Ubuntu/Debian

```bash
# Bağımlılıkları yükleyin
sudo apt-get update
sudo apt-get install -y curl build-essential

# Rust'ı yükleyin
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Soketi'yi derleyin
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
```

### CentOS/RHEL

```bash
# Bağımlılıkları yükleyin
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl

# Rust'ı yükleyin
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Soketi'yi derleyin
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
```

### macOS

```bash
# Homebrew yüklü değilse yükleyin
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Rust'ı yükleyin
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Soketi'yi derleyin
git clone https://github.com/soketi/soketi.rs.git
cd soketi.rs
cargo build --release
```

## Doğrulama

Kurulumdan sonra, Soketi'nin çalıştığını doğrulayın:

### Sürümü Kontrol Etme

```bash
soketi --version
```

### Varsayılan Yapılandırma ile Başlatma

```bash
soketi
```

Şuna benzer bir çıktı görmelisiniz:

```
[INFO] Starting Soketi server...
[INFO] WebSocket server listening on 0.0.0.0:6001
[INFO] Metrics server listening on 0.0.0.0:9601
```

### Bağlantıyı Test Etme

Health endpoint'ini kontrol etmek için curl kullanın:

```bash
curl http://localhost:6001/
```

Beklenen yanıt:

```json
{
  "version": "1.0.0",
  "status": "ok"
}
```

## Sonraki Adımlar

- **[Yapılandırma Kılavuzu](yapilandirma.md)** - Soketi'yi ihtiyaçlarınıza göre yapılandırın
- **[Başlangıç](baslangic.md)** - Hızlı başlangıç kılavuzu
- **[Deployment Kılavuzu](deployment/reverse-proxy.md)** - Production'a deploy etme

## İlgili Kaynaklar

- [GitHub Deposu](https://github.com/soketi/soketi.rs)
- [Docker Hub](https://quay.io/repository/soketi/soketi)
- [Sorun Giderme Kılavuzu](sorun-giderme.md)
