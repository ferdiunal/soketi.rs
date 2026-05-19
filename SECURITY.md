# Security Policy

This project follows responsible disclosure for vulnerabilities that affect the Soketi.rs runtime, Docker image, deployment assets, or Pusher-compatible HTTP and WebSocket surfaces.

## Supported Versions

Security fixes are prepared for the latest release line and the current `main` branch. If you run an older release, upgrade to the latest published image or release before reporting an issue that may already be fixed.

## Reporting a Vulnerability

Do not open a public issue for a suspected vulnerability.

- Prefer GitHub private vulnerability reporting or GitHub Security Advisories when available for this repository.
- If private reporting is unavailable, contact the maintainer listed in `Cargo.toml`: `ferdi@ferdiunal.com`.
- Include the affected version, deployment mode, reproduction steps, logs that do not expose secrets, and the expected impact.

## Recent Security Hardening

The following hardening items summarize a Codex Security review completed on 2026-05-19 and the follow-up commits that closed the reportable findings.

| Finding | Severity | Fixed by | Status |
| --- | --- | --- | --- |
| WebSocket app policy bypass: the deployed `/app/{app_key}` route could skip disabled-app, max-connection, and user-authentication controls. | High / P1 | `c29b3b5` `fix: enforce websocket app policies` | Fixed |
| HTTP API body integrity gap: signed POST requests did not bind the request body to the signature. | Medium / P2 | `172fb47` `fix: bind api signatures to request bodies` | Fixed |
| WebSocket close-path cleanup gap: disconnected sockets could leave stale adapter state. | Medium / P2 | `04472e2` `fix: clean websocket state on all close paths` | Fixed |
| HTTP API body limit gap: event handlers could buffer oversized bodies and validate against default limits instead of live config. | Medium / P2 | `31d98fa` `fix: enforce configured api body limits` | Fixed |

Follow-up hardening items remain deployment-sensitive and should be tracked separately:

- Webhook egress and SSRF controls when operators allow lower-trust users to manage webhook URLs.
- Redis/SQS queue trust boundaries because queued webhook jobs may include app secrets.
- Dependency advisory scanning in CI, including Rust advisory checks.

## Recent Performance Hardening

The latest performance commits reduce runtime overhead without changing the public protocol:

- `d64b3ed` speeds up local adapter sends.
- `2390870` reduces client event hot-path overhead.
- `c3bd38a` stabilizes the local end-to-end latency benchmark.

## Güvenlik Politikası

Bu proje, Soketi.rs çalışma zamanı, Docker imajı, dağıtım varlıkları ve Pusher uyumlu HTTP/WebSocket yüzeylerini etkileyen açıklar için sorumlu bildirim akışını takip eder.

## Desteklenen Sürümler

Güvenlik düzeltmeleri en güncel sürüm hattı ve mevcut `main` dalı için hazırlanır. Daha eski bir sürüm kullanıyorsanız, daha önce giderilmiş bir bulguyu raporlamadan önce son yayımlanan imaja veya sürüme yükseltin.

## Güvenlik Açığı Bildirme

Şüpheli bir güvenlik açığı için herkese açık issue açmayın.

- Bu repository için aktifse GitHub private vulnerability reporting veya GitHub Security Advisories kullanın.
- Özel bildirim akışı yoksa `Cargo.toml` içindeki maintainer adresini kullanın: `ferdi@ferdiunal.com`.
- Etkilenen sürümü, dağıtım modunu, tekrar üretme adımlarını, gizli bilgi içermeyen logları ve beklenen etkiyi ekleyin.

## Son Güvenlik Sıkılaştırmaları

Aşağıdaki maddeler 2026-05-19 tarihli Codex Security incelemesinde raporlanabilir bulunan başlıkları ve bunları kapatan commitleri özetler.

| Bulgu | Seviye | Düzelten commit | Durum |
| --- | --- | --- | --- |
| WebSocket uygulama politikası atlama: dağıtılan `/app/{app_key}` rotası disabled app, maksimum bağlantı ve user-authentication kontrollerini atlayabiliyordu. | High / P1 | `c29b3b5` `fix: enforce websocket app policies` | Giderildi |
| HTTP API gövde bütünlüğü boşluğu: imzalı POST isteklerinde request body imzaya bağlanmıyordu. | Medium / P2 | `172fb47` `fix: bind api signatures to request bodies` | Giderildi |
| WebSocket kapanış temizliği boşluğu: kopan socketler adapter içinde eski state bırakabiliyordu. | Medium / P2 | `04472e2` `fix: clean websocket state on all close paths` | Giderildi |
| HTTP API gövde limiti boşluğu: event handlerları aşırı büyük body'leri bufferlayabiliyor ve canlı config yerine varsayılan limitlerle doğrulama yapabiliyordu. | Medium / P2 | `31d98fa` `fix: enforce configured api body limits` | Giderildi |

Takip edilmesi gereken ek sıkılaştırma başlıkları dağıtım bağlamına bağlıdır:

- Düşük güven seviyesindeki kullanıcılar webhook URL'lerini yönetebiliyorsa webhook egress ve SSRF kontrolleri.
- Kuyruğa alınan webhook işleri app secret içerebildiği için Redis/SQS kuyruk güven sınırları.
- CI içinde Rust advisory kontrollerini de kapsayan dependency advisory taraması.

## Son Performans Sıkılaştırmaları

Son performans commitleri public protokolü değiştirmeden runtime maliyetini azaltır:

- `d64b3ed` local adapter send yolunu hızlandırır.
- `2390870` client event hot-path maliyetini azaltır.
- `c3bd38a` local uçtan uca gecikme benchmark'ını stabil hale getirir.
