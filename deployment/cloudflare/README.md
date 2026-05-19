# Cloudflare Containers deployment

This deployment runs the existing Docker Hub image as a Cloudflare Container and uses a Worker as the public HTTP/WebSocket proxy.

Full guides:

- [English Cloudflare Containers deployment](../../docs/en/deployment/cloudflare-containers.md)
- [Turkish Cloudflare Containers deployment](../../docs/tr/deployment/cloudflare-containers.md)

## Files

- `wrangler.jsonc` defines the Worker, Container image, Durable Object binding, and default non-secret runtime variables.
- `src/index.ts` proxies HTTP and WebSocket traffic to the `soketi-rs` container on port `6001`.
- `scripts/render-wrangler.mjs` renders a generated Wrangler config from deploy-time environment variables.
- `.dev.vars.example` shows local development variables. Copy it to `.dev.vars` for `wrangler dev`.

## Install

```bash
cd deployment/cloudflare
npm install
```

## Configure secrets

Do not store production secrets in `wrangler.jsonc`.

```bash
npx wrangler secret put PUSHER_DEFAULT_APP_SECRET
```

Use additional `wrangler secret put` commands for Redis passwords, AWS credentials, database credentials, or `SOKETI_METRICS_BEARER_TOKEN`.

Admin endpoints for Durable Object app config require:

```bash
npx wrangler secret put SOKETI_ADMIN_BEARER_TOKEN
```

## Image tag

The default image is:

```text
docker.io/funal/soketi-rs:latest
```

Cloudflare's `containers.image` value is deploy-time config, so use the env renderer when you want to select a tag without editing `wrangler.jsonc`:

```bash
SOKETI_IMAGE_TAG=v1.2.6 npm run deploy:env
```

Or provide the full image reference:

```bash
SOKETI_IMAGE=docker.io/funal/soketi-rs:main npm run deploy:env
```

## Topology

Default routing is a single named container instance:

```bash
SOKETI_CONTAINER_ROUTING=single
SOKETI_CONTAINER_INSTANCE_NAME=soketi-rs
```

For multiple instances, set a distributed adapter first. Redis example:

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

Then configure the Redis host/port/password using `wrangler.jsonc` vars for non-secrets and `wrangler secret put` for secrets.

`random` routing is blocked when `PUSHER_ADAPTER_DRIVER` is unset or `local`, because isolated containers would not share WebSocket channel state.

## App config driver

Cloudflare deployment supports two app config sources:

```bash
SOKETI_APP_CONFIG_DRIVER=array
```

`array` is the default. The container reads app credentials from regular `PUSHER_*` vars:

- `PUSHER_DEFAULT_APP_ID`
- `PUSHER_DEFAULT_APP_KEY`
- `PUSHER_DEFAULT_APP_SECRET`
- or `PUSHER_APP_MANAGER_ARRAY_APPS`

```bash
SOKETI_APP_CONFIG_DRIVER=durable-object
```

`durable-object` stores the app list in Cloudflare Durable Object storage, then injects it into the container as `PUSHER_APP_MANAGER_ARRAY_APPS` when the container starts. The native `soketi-rs` binary still uses its existing `array` app manager internally.

Durable Object mode seeds itself from `PUSHER_APP_MANAGER_ARRAY_APPS` or `PUSHER_DEFAULT_APP_*` if storage is empty.

Manage Durable Object app config with the admin endpoint:

```bash
curl -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/app-config/apps
```

Replace the app list:

```bash
curl -X PUT \
  -H "Authorization: Bearer <admin-token>" \
  -H "Content-Type: application/json" \
  --data '{"apps":[{"id":"app-id","key":"app-key","secret":"app-secret","enabled":true}]}' \
  "https://<worker-host>/__cf/soketi/app-config/apps?restart=true"
```

The container reads app config at startup. Use `?restart=true` on update, or call:

```bash
curl -X POST -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/container-restart
```

## Cloudflare KV is not a Redis adapter

Cloudflare KV can be useful for slow-changing configuration or cache data, but it is not a drop-in replacement for Redis in this project. The Redis paths in `soketi-rs` need pub/sub, queue, and low-latency shared state behavior. For multi-instance realtime deployments, use Redis or NATS, or build a dedicated Durable Objects adapter instead of wiring KV into the Redis adapter surface.

## Metrics

`/metrics` is blocked by default. To expose the container metrics port:

```bash
SOKETI_EXPOSE_METRICS=true npm run deploy:env
npx wrangler secret put SOKETI_METRICS_BEARER_TOKEN
```

When a bearer token is configured, call metrics with:

```bash
curl -H "Authorization: Bearer <token>" https://<worker-host>/metrics
```

## Deploy

```bash
npm run deploy
```

For env-rendered deploys:

```bash
SOKETI_IMAGE_TAG=v1.2.6 npm run deploy:env
```

Check provisioning:

```bash
npm run containers:list
```

Smoke checks:

```bash
curl https://<worker-host>/
curl https://<worker-host>/ready
```

WebSocket clients should connect to:

```text
wss://<worker-host>/app/<app-key>
```
