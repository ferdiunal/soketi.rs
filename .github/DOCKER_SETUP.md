# Docker Hub Setup Guide

Bu dosya, Docker Hub'a otomatik image yayını için gerekli ayarları açıklar.

## 1. Docker Hub Token Oluşturma

1. [Docker Hub](https://hub.docker.com/) hesabına giriş yap
2. Account Settings → Security → New Access Token
3. Token açıklaması: `GitHub Actions - soketi-rs`
4. Access permissions: `Read, Write, Delete`
5. Token'ı kopyala (bir daha gösterilmeyecek!)

## 2. GitHub Secrets Ekleme

1. GitHub repository'ye git: https://github.com/ferdiunal/soketi.rs
2. Settings → Secrets and variables → Actions
3. "New repository secret" butonuna tıkla
4. Secret ekle:
   - **Name**: `DOCKER_HUB_TOKEN`
   - **Value**: Docker Hub'dan kopyaladığın token
5. "Add secret" butonuna tıkla

## 3. Docker Hub Repository Oluşturma

1. [Docker Hub](https://hub.docker.com/) → Repositories → Create Repository
2. Repository bilgileri:
   - **Name**: `soketi-rs`
   - **Visibility**: Public
   - **Description**: High-performance Pusher protocol compatible WebSocket server written in Rust
3. "Create" butonuna tıkla

## 4. Otomatik Yayın Tetikleme

### Tag ile yayın (Önerilen)
```bash
# Version tag oluştur
git tag v1.0.0
git push origin v1.0.0

# Bu şu image'ları oluşturur:
# - ferdiunal/soketi-rs:v1.0.0
# - ferdiunal/soketi-rs:1.0
# - ferdiunal/soketi-rs:1
# - funal/soketi-rs:latest
```

### Branch push ile yayın
```bash
# Main branch'e push
git push origin main

# Bu şu image'ı oluşturur:
# - ferdiunal/soketi-rs:main
# - funal/soketi-rs:latest
```

### Manuel tetikleme
1. GitHub → Actions → Docker Image CI/CD
2. "Run workflow" butonuna tıkla
3. Branch seç ve "Run workflow"

## 5. Workflow Durumu Kontrol

1. GitHub → Actions sekmesi
2. "Docker Image CI/CD" workflow'unu seç
3. Son çalıştırmaları görüntüle
4. Başarılı olursa ✅, hata varsa ❌ gösterir

## 6. Docker Hub'da Kontrol

1. [Docker Hub](https://hub.docker.com/r/funal/soketi-rs)
2. Tags sekmesinde yeni image'ları görebilirsin
3. README otomatik güncellenecek

## 7. Image'ı Test Etme

```bash
# Latest image'ı çek
docker pull funal/soketi-rs:latest

# Çalıştır
docker run -d \
  --name soketi-test \
  -p 6001:6001 \
  -p 9601:9601 \
  funal/soketi-rs:latest

# Logları kontrol et
docker logs soketi-test

# Health check
curl http://localhost:6001/health

# Temizle
docker stop soketi-test
docker rm soketi-test
```

## Troubleshooting

### "Error: buildx failed with: ERROR: failed to solve: failed to push"
- Docker Hub token'ın doğru olduğundan emin ol
- Token'ın Write yetkisi olduğunu kontrol et
- GitHub secret'ın doğru eklendiğini kontrol et

### "Error: denied: requested access to the resource is denied"
- Docker Hub repository'nin public olduğundan emin ol
- Repository adının doğru olduğunu kontrol et: `ferdiunal/soketi-rs`

### Build başarılı ama image yok
- Workflow'un push adımının çalıştığını kontrol et
- Pull request'lerde push yapılmaz, sadece build edilir

## Notlar

- ✅ Multi-platform build (amd64, arm64)
- ✅ Build cache kullanılıyor (hızlı build)
- ✅ README otomatik güncelleniyor
- ✅ Semantic versioning destekleniyor
- ✅ Latest tag otomatik güncelleniyor

## Kullanıcı Adı Değişikliği

Eğer Docker Hub kullanıcı adını değiştirmek istersen:

1. `.github/workflows/docker-publish.yml` dosyasını aç
2. `DOCKER_USERNAME: ferdiunal` satırını değiştir
3. Commit ve push yap

## Sonraki Adımlar

1. ✅ Docker Hub token oluştur
2. ✅ GitHub secret ekle
3. ✅ Docker Hub repository oluştur
4. ✅ İlk tag'i oluştur ve push et
5. ✅ GitHub Actions'da build'i izle
6. ✅ Docker Hub'da image'ı kontrol et
7. ✅ Image'ı test et

Başarılar! 🚀
