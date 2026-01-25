# Commit Convention

Soketi.rs, [Conventional Commits](https://www.conventionalcommits.org/) standardını kullanır.

## Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

## Commit Types

| Type | Description | Version Bump | Example |
|------|-------------|--------------|---------|
| `feat` | Yeni özellik | **MINOR** (0.1.0 → 0.2.0) | `feat: add NATS adapter support` |
| `fix` | Bug fix | **PATCH** (0.1.0 → 0.1.1) | `fix: resolve memory leak in presence channels` |
| `perf` | Performance iyileştirmesi | **PATCH** | `perf: optimize WebSocket message handling` |
| `docs` | Dokümantasyon | **PATCH** | `docs: update installation guide` |
| `style` | Code style (formatting, vb.) | **PATCH** | `style: format code with rustfmt` |
| `refactor` | Code refactoring | **PATCH** | `refactor: simplify adapter interface` |
| `test` | Test ekleme/düzeltme | **PATCH** | `test: add integration tests for Redis adapter` |
| `build` | Build system değişiklikleri | **PATCH** | `build: update Dockerfile` |
| `ci` | CI/CD değişiklikleri | **PATCH** | `ci: add semantic release workflow` |
| `chore` | Diğer değişiklikler | **NO RELEASE** | `chore: update dependencies` |
| `revert` | Commit geri alma | **PATCH** | `revert: revert "feat: add feature X"` |

## Breaking Changes

Breaking change için `BREAKING CHANGE:` footer ekle veya `!` kullan:

```bash
# Footer ile
feat: change API endpoint structure

BREAKING CHANGE: API endpoints now use /v2/ prefix

# ! ile
feat!: change API endpoint structure
```

Bu **MAJOR** version bump yapar (0.1.0 → 1.0.0)

## Scope (Opsiyonel)

Değişikliğin hangi modülü etkilediğini belirtir:

```bash
feat(adapter): add NATS support
fix(websocket): resolve connection timeout
docs(readme): update installation steps
```

Yaygın scope'lar:
- `adapter` - Adapter modülü
- `websocket` - WebSocket işlemleri
- `auth` - Authentication
- `channels` - Channel yönetimi
- `metrics` - Metrics ve monitoring
- `docker` - Docker deployment
- `config` - Configuration
- `api` - HTTP API

## Örnekler

### Feature (Minor Release)

```bash
git commit -m "feat: add Redis cluster support

Implements Redis cluster mode for horizontal scaling.
Supports multiple Redis nodes with automatic failover.
"
```

### Bug Fix (Patch Release)

```bash
git commit -m "fix: resolve WebSocket connection timeout

Fixes issue where connections would timeout after 30 seconds.
Increases default timeout to 60 seconds and makes it configurable.

Closes #123
"
```

### Breaking Change (Major Release)

```bash
git commit -m "feat!: redesign configuration structure

BREAKING CHANGE: Configuration file format has changed.
Old config files need to be migrated to new format.

Migration guide: docs/MIGRATION.md
"
```

### Documentation (Patch Release)

```bash
git commit -m "docs: add Docker deployment guide

Adds comprehensive guide for deploying with Docker.
Includes examples for nginx and caddy reverse proxy.
"
```

### Chore (No Release)

```bash
git commit -m "chore: update dependencies

Updates tokio to 1.49.0 and axum to 0.8.8
"
```

## Multi-line Commits

```bash
git commit -m "feat(adapter): add NATS adapter support" -m "
Implements NATS adapter for distributed messaging.

Features:
- Connection pooling
- Automatic reconnection
- Message persistence
- Cluster support

Closes #45
"
```

## Commit Message Template

`.gitmessage` dosyası oluştur:

```
# <type>(<scope>): <subject>
# |<----  50 chars  ---->|

# <body>
# |<----  72 chars  ---->|

# <footer>

# Type: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
# Scope: adapter, websocket, auth, channels, metrics, docker, config, api
# Subject: imperative mood, lowercase, no period
# Body: explain what and why (not how)
# Footer: breaking changes, issue references
```

Kullanım:
```bash
git config commit.template .gitmessage
```

## Semantic Release Davranışı

### Otomatik Version Bump

| Commit Type | Version Change | Example |
|-------------|----------------|---------|
| `fix:` | 0.1.0 → 0.1.1 | Patch |
| `feat:` | 0.1.0 → 0.2.0 | Minor |
| `BREAKING CHANGE:` | 0.1.0 → 1.0.0 | Major |
| `chore:` | No release | - |

### Otomatik Changelog

Semantic Release otomatik olarak CHANGELOG.md oluşturur:

```markdown
## [0.2.0] - 2026-01-25

### ✨ Features
- add NATS adapter support (#45)
- implement Redis cluster mode (#46)

### 🐛 Bug Fixes
- resolve WebSocket timeout issue (#123)
- fix memory leak in presence channels (#124)

### 📚 Documentation
- add Docker deployment guide
- update API reference
```

### Otomatik GitHub Release

Her release için GitHub Release oluşturulur:
- Release notes (changelog'den)
- Git tag (v0.2.0)
- Docker images (funal/soketi-rs:0.2.0)

## Workflow

1. **Değişiklik yap**
   ```bash
   # Feature branch oluştur
   git checkout -b feature/nats-adapter
   
   # Değişiklikleri yap
   # ...
   ```

2. **Conventional commit ile commit et**
   ```bash
   git add .
   git commit -m "feat(adapter): add NATS adapter support"
   ```

3. **Main branch'e merge et**
   ```bash
   git checkout main
   git merge feature/nats-adapter
   git push origin main
   ```

4. **Semantic Release otomatik çalışır**
   - Version bump yapar
   - CHANGELOG.md günceller
   - GitHub Release oluşturur
   - Docker images build eder
   - Docker Hub'a push eder

## Commitlint (Opsiyonel)

Commit mesajlarını otomatik kontrol etmek için:

```bash
# .commitlintrc.json
{
  "extends": ["@commitlint/config-conventional"]
}
```

## Tips

✅ **DO:**
- Imperative mood kullan: "add feature" (not "added feature")
- Küçük harf kullan: "feat: add feature" (not "Feat: Add Feature")
- Nokta koyma: "feat: add feature" (not "feat: add feature.")
- Açıklayıcı ol: Ne ve neden yaptığını açıkla

❌ **DON'T:**
- Belirsiz mesajlar: "fix bug", "update code"
- Çok uzun subject: 50 karakteri geçme
- Birden fazla değişiklik: Her değişiklik için ayrı commit

## Kaynaklar

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Angular Commit Guidelines](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
