# Cloudflare Containers Dağıtımı

Bu kılavuz, yayımlanmış `funal/soketi-rs` Docker imajının Cloudflare Containers üzerinde, önünde bir Worker ile nasıl çalıştırılacağını açıklar.

## İçindekiler

- [Genel Bakış](#genel-bakış)
- [Ön Gereksinimler](#ön-gereksinimler)
- [Dosyalar](#dosyalar)
- [Kurulum](#kurulum)
- [Gizli Değerler](#gizli-değerler)
- [İmaj Seçimi](#imaj-seçimi)
- [Uygulama Yapılandırması](#uygulama-yapılandırması)
- [Topoloji](#topoloji)
- [Metrikler](#metrikler)
- [Dağıtım](#dağıtım)
- [Smoke Kontrolleri](#smoke-kontrolleri)
- [Operasyon Notları](#operasyon-notları)
- [Referanslar](#referanslar)

## Genel Bakış

Cloudflare dağıtımı `deployment/cloudflare` dizininde bulunur. Wrangler bir Worker ve container-enabled Durable Object dağıtır. Worker public HTTP ve WebSocket trafiğini karşılar, `soketi-rs` konteynerini başlatır ve trafiği `6001` portuna proxy eder.

Cloudflare Containers, Durable Objects tarafından desteklenir. Bu nedenle her konteyner instance'ı bir Durable Object binding üzerinden kontrol edilir. Bu zorunlu model `wrangler.jsonc` içinde tanımlanır.

## Ön Gereksinimler

- Workers ve Containers erişimi olan bir Cloudflare hesabı.
- `deployment/cloudflare` paketi için Node.js ve npm.
- Workers, Durable Objects ve Containers dağıtımı yapabilen Wrangler oturumu.
- `docker.io/funal/soketi-rs:v1.2.6` gibi yayımlanmış bir Docker imaj tag'i.
- En az bir Soketi.rs uygulaması için app credentials.

## Dosyalar

- `deployment/cloudflare/wrangler.jsonc` Worker'ı, Container imajını, Durable Object bindinglerini, migration'ları ve gizli olmayan varsayılan değişkenleri tanımlar.
- `deployment/cloudflare/src/index.ts` HTTP ve WebSocket isteklerini konteynere proxy eder ve opsiyonel admin endpointlerini uygular.
- `deployment/cloudflare/scripts/render-wrangler.mjs` deploy-time ortam değişkenlerinden `wrangler.generated.jsonc` üretir.
- `deployment/cloudflare/.dev.vars.example` lokal geliştirme değerlerini belgeler.

## Kurulum

```bash
cd deployment/cloudflare
npm install
```

Lokal geliştirme için örnek değişkenleri kopyalayın:

```bash
cp .dev.vars.example .dev.vars
npm run dev
```

## Gizli Değerler

Production gizli değerlerini `wrangler.jsonc` içine yazmayın. App secret, admin token, Redis şifresi, veritabanı kimlik bilgileri, AWS kimlik bilgileri ve metrics token için Wrangler secrets kullanın.

```bash
npx wrangler secret put PUSHER_DEFAULT_APP_SECRET
npx wrangler secret put SOKETI_ADMIN_BEARER_TOKEN
```

Seçilen Soketi.rs driver'larının ihtiyaç duyduğu ek backend secret'larını ekleyin:

```bash
npx wrangler secret put PUSHER_ADAPTER_REDIS_PASSWORD
npx wrangler secret put PUSHER_CACHE_REDIS_PASSWORD
npx wrangler secret put AWS_ACCESS_KEY_ID
npx wrangler secret put AWS_SECRET_ACCESS_KEY
```

## İmaj Seçimi

Varsayılan imaj:

```text
docker.io/funal/soketi-rs:latest
```

Cloudflare konteyner imajını Wrangler deploy-time configuration üzerinden okur. `wrangler.jsonc` düzenlemeden belirli bir tag deploy etmek için generated config üretin:

```bash
SOKETI_IMAGE_TAG=v1.2.6 npm run deploy:env
```

Tam imaj referansı da verebilirsiniz:

```bash
SOKETI_IMAGE=docker.io/funal/soketi-rs:main npm run deploy:env
```

Cloudflare Containers, Docker Hub üzerindeki pre-built imajları destekler. Bu akışta Wrangler içinde yeniden build almak yerine hazır public imaj kullanılır.

## Uygulama Yapılandırması

Cloudflare wrapper iki uygulama yapılandırma driver'ı destekler. İki mod da native `soketi-rs` binary'sini mevcut `array` app manager ile başlatır.

### Array Mode

`array`, varsayılan ve app credentials değerlerinin ortam değişkenleriyle deploy edildiği en sade production yoludur.

```bash
SOKETI_APP_CONFIG_DRIVER=array
```

Tek varsayılan app verin:

```bash
PUSHER_DEFAULT_APP_ID=app-id
PUSHER_DEFAULT_APP_KEY=app-key
npx wrangler secret put PUSHER_DEFAULT_APP_SECRET
```

Birden fazla app için JSON verin:

```bash
npx wrangler secret put PUSHER_APP_MANAGER_ARRAY_APPS
```

### Durable Object Mode

`durable-object`, app listesini Cloudflare Durable Object içinde saklar ve konteyner başlarken `PUSHER_APP_MANAGER_ARRAY_APPS` olarak enjekte eder.

```bash
SOKETI_APP_CONFIG_DRIVER=durable-object
SOKETI_APP_CONFIG_INSTANCE_NAME=default
```

Storage boş olduğunda `PUSHER_APP_MANAGER_ARRAY_APPS` veya `PUSHER_DEFAULT_APP_*` değerlerinden seed edilir.

Admin endpointleri `SOKETI_ADMIN_BEARER_TOKEN` ister:

```bash
curl -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/app-config/apps
```

App listesini değiştirip konteyneri yeniden başlatın:

```bash
curl -X PUT \
  -H "Authorization: Bearer <admin-token>" \
  -H "Content-Type: application/json" \
  --data '{"apps":[{"id":"app-id","key":"app-key","secret":"app-secret","enabled":true}]}' \
  "https://<worker-host>/__cf/soketi/app-config/apps?restart=true"
```

Durable Object app config verisini temizleyin:

```bash
curl -X DELETE \
  -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/app-config/apps
```

Yapılandırılmış konteyneri açıkça yeniden başlatın:

```bash
curl -X POST \
  -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/container-restart
```

## Topoloji

Varsayılan topoloji tek isimlendirilmiş konteyner instance'ıdır:

```bash
SOKETI_CONTAINER_ROUTING=single
SOKETI_CONTAINER_INSTANCE_NAME=soketi-rs
SOKETI_CONTAINER_INSTANCE_COUNT=1
```

Birden fazla konteyner instance'ı için random routing açmadan önce distributed adapter kullanın:

```bash
SOKETI_CONTAINER_ROUTING=random \
SOKETI_CONTAINER_INSTANCE_COUNT=3 \
SOKETI_CONTAINER_MAX_INSTANCES=3 \
PUSHER_ADAPTER_DRIVER=redis \
PUSHER_CACHE_DRIVER=redis \
PUSHER_RATE_LIMITER_DRIVER=redis \
PUSHER_QUEUE_DRIVER=redis \
npm run deploy:env
```

`random` routing, local adapter ile varsayılan olarak engellenir. Çünkü izole konteynerler WebSocket kanal state'ini paylaşmaz.

Redis veya NATS host ve port değerlerini `wrangler.jsonc` içindeki `vars` alanında yapılandırın; şifreleri veya provider kimlik bilgilerini `wrangler secret put` ile saklayın. Env renderer yalnızca deploy-time imajı, konteyner limitlerini, routing modunu, app-config driver'ını ve seçili yüksek seviye Soketi.rs driver'larını değiştirir.

## Metrikler

`/metrics` varsayılan olarak gizlidir.

```bash
SOKETI_EXPOSE_METRICS=true npm run deploy:env
npx wrangler secret put SOKETI_METRICS_BEARER_TOKEN
```

Metrikleri bearer token ile okuyun:

```bash
curl -H "Authorization: Bearer <token>" https://<worker-host>/metrics
```

## Dağıtım

Repo içindeki yapılandırmayı deploy edin:

```bash
npm run deploy
```

Generated ortam override'ları ile deploy edin:

```bash
SOKETI_IMAGE_TAG=v1.2.6 npm run deploy:env
```

Konteyner durumunu inceleyin:

```bash
npm run containers:list
npm run containers:images
```

## Smoke Kontrolleri

Worker ve konteyner health endpointlerini kontrol edin:

```bash
curl https://<worker-host>/
curl https://<worker-host>/ready
```

WebSocket istemcileri TLS ile bağlanmalıdır:

```text
wss://<worker-host>/app/<app-key>
```

Pusher.js için:

```javascript
const pusher = new Pusher("app-key", {
  wsHost: "<worker-host>",
  wssPort: 443,
  forceTLS: true,
  enabledTransports: ["ws", "wss"]
});
```

## Operasyon Notları

- Cloudflare KV, Soketi.rs realtime state için Redis yerine geçmez. KV yavaş değişen metadata için kullanışlıdır; Redis ve NATS ise multi-instance realtime fan-out için gereken düşük gecikmeli pub/sub ve shared state davranışını sağlar.
- Durable Object app config, dağıtım wrapper'ına ait bir özelliktir. Native binary konteyner içinde yine `array` app manager kullanır.
- App config deployment ile değişecekse `array` mode kullanın. Operatörler guarded admin endpointleri ile runtime app-list update istiyorsa `durable-object` mode kullanın.
- `SOKETI_ADMIN_BEARER_TOKEN` ve `SOKETI_METRICS_BEARER_TOKEN` değerlerini secret olarak saklayın.
- Production multi-instance dağıtımlarda `SOKETI_CONTAINER_INSTANCE_COUNT` artırmadan önce Redis veya NATS yapılandırın.

## Referanslar

- [Cloudflare Containers getting started](https://developers.cloudflare.com/containers/get-started/)
- [Cloudflare Containers image management](https://developers.cloudflare.com/containers/image-management/)
- [Wrangler configuration for Containers and Durable Objects](https://developers.cloudflare.com/workers/wrangler/configuration/)
- [Cloudflare Durable Objects overview](https://developers.cloudflare.com/durable-objects/)
