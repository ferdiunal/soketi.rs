# Cloudflare Containers Deployment

This guide explains how to run the published `funal/soketi-rs` Docker image on Cloudflare Containers with a Worker in front of it.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Files](#files)
- [Install](#install)
- [Secrets](#secrets)
- [Image Selection](#image-selection)
- [App Configuration](#app-configuration)
- [Topology](#topology)
- [Metrics](#metrics)
- [Deploy](#deploy)
- [Smoke Checks](#smoke-checks)
- [Operational Notes](#operational-notes)
- [References](#references)

## Overview

The Cloudflare deployment lives in `deployment/cloudflare`. It uses Wrangler to deploy a Worker and a container-enabled Durable Object. The Worker handles public HTTP and WebSocket traffic, starts the `soketi-rs` container, and proxies traffic to port `6001`.

Cloudflare Containers are backed by Durable Objects, so each container instance is controlled through a Durable Object binding. This is required by the Cloudflare Containers model and is represented in `wrangler.jsonc`.

## Prerequisites

- A Cloudflare account with Workers and Containers access.
- Node.js and npm for the `deployment/cloudflare` package.
- Wrangler authentication with permission to deploy Workers, Durable Objects, and Containers.
- A published Docker image tag such as `docker.io/funal/soketi-rs:v1.2.6`.
- App credentials for at least one Soketi.rs app.

## Files

- `deployment/cloudflare/wrangler.jsonc` defines the Worker, Container image, Durable Object bindings, migrations, and default non-secret variables.
- `deployment/cloudflare/src/index.ts` proxies HTTP and WebSocket requests to the container and implements optional admin endpoints.
- `deployment/cloudflare/scripts/render-wrangler.mjs` renders `wrangler.generated.jsonc` from deploy-time environment variables.
- `deployment/cloudflare/.dev.vars.example` documents local development values.

## Install

```bash
cd deployment/cloudflare
npm install
```

For local development, copy the example variables:

```bash
cp .dev.vars.example .dev.vars
npm run dev
```

## Secrets

Do not store production secrets in `wrangler.jsonc`. Use Wrangler secrets for app secrets, admin tokens, Redis passwords, database credentials, AWS credentials, and metrics tokens.

```bash
npx wrangler secret put PUSHER_DEFAULT_APP_SECRET
npx wrangler secret put SOKETI_ADMIN_BEARER_TOKEN
```

Add any optional backend secrets the selected Soketi.rs drivers need:

```bash
npx wrangler secret put PUSHER_ADAPTER_REDIS_PASSWORD
npx wrangler secret put PUSHER_CACHE_REDIS_PASSWORD
npx wrangler secret put AWS_ACCESS_KEY_ID
npx wrangler secret put AWS_SECRET_ACCESS_KEY
```

## Image Selection

The default image is:

```text
docker.io/funal/soketi-rs:latest
```

Cloudflare reads the container image from Wrangler deploy-time configuration. To deploy a specific tag without editing `wrangler.jsonc`, render a generated config:

```bash
SOKETI_IMAGE_TAG=v1.2.6 npm run deploy:env
```

You can also provide a full image reference:

```bash
SOKETI_IMAGE=docker.io/funal/soketi-rs:main npm run deploy:env
```

Cloudflare Containers support pre-built images from Docker Hub, so this flow uses the already-built public image instead of building inside Wrangler.

## App Configuration

The Cloudflare wrapper supports two app configuration drivers. Both modes ultimately start the native `soketi-rs` binary with its existing `array` app manager.

### Array Mode

`array` is the default and is the simplest production path when app credentials are deployed through environment variables.

```bash
SOKETI_APP_CONFIG_DRIVER=array
```

Provide one default app:

```bash
PUSHER_DEFAULT_APP_ID=app-id
PUSHER_DEFAULT_APP_KEY=app-key
npx wrangler secret put PUSHER_DEFAULT_APP_SECRET
```

Or provide multiple apps as JSON:

```bash
npx wrangler secret put PUSHER_APP_MANAGER_ARRAY_APPS
```

### Durable Object Mode

`durable-object` stores the app list in a Cloudflare Durable Object and injects it into the container as `PUSHER_APP_MANAGER_ARRAY_APPS` when the container starts.

```bash
SOKETI_APP_CONFIG_DRIVER=durable-object
SOKETI_APP_CONFIG_INSTANCE_NAME=default
```

The storage is seeded from `PUSHER_APP_MANAGER_ARRAY_APPS` or `PUSHER_DEFAULT_APP_*` when it is empty.

Admin endpoints require `SOKETI_ADMIN_BEARER_TOKEN`:

```bash
curl -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/app-config/apps
```

Replace the app list and restart the container:

```bash
curl -X PUT \
  -H "Authorization: Bearer <admin-token>" \
  -H "Content-Type: application/json" \
  --data '{"apps":[{"id":"app-id","key":"app-key","secret":"app-secret","enabled":true}]}' \
  "https://<worker-host>/__cf/soketi/app-config/apps?restart=true"
```

Clear Durable Object app config:

```bash
curl -X DELETE \
  -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/app-config/apps
```

Restart the configured container explicitly:

```bash
curl -X POST \
  -H "Authorization: Bearer <admin-token>" \
  https://<worker-host>/__cf/soketi/container-restart
```

## Topology

The default topology is one named container instance:

```bash
SOKETI_CONTAINER_ROUTING=single
SOKETI_CONTAINER_INSTANCE_NAME=soketi-rs
SOKETI_CONTAINER_INSTANCE_COUNT=1
```

For multiple container instances, use a distributed adapter before enabling random routing:

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

`random` routing is blocked with the local adapter by default because isolated containers do not share WebSocket channel state.

Configure Redis or NATS host and port values in `wrangler.jsonc` `vars`, and store passwords or provider credentials with `wrangler secret put`. The env renderer only changes the deploy-time image, container limits, routing mode, app-config driver, and selected high-level Soketi.rs drivers.

## Metrics

`/metrics` is hidden by default.

```bash
SOKETI_EXPOSE_METRICS=true npm run deploy:env
npx wrangler secret put SOKETI_METRICS_BEARER_TOKEN
```

Read metrics with a bearer token:

```bash
curl -H "Authorization: Bearer <token>" https://<worker-host>/metrics
```

## Deploy

Deploy the checked-in configuration:

```bash
npm run deploy
```

Deploy with generated environment overrides:

```bash
SOKETI_IMAGE_TAG=v1.2.6 npm run deploy:env
```

Inspect container status:

```bash
npm run containers:list
npm run containers:images
```

## Smoke Checks

Check the Worker and container health endpoints:

```bash
curl https://<worker-host>/
curl https://<worker-host>/ready
```

WebSocket clients should connect with TLS:

```text
wss://<worker-host>/app/<app-key>
```

For Pusher.js:

```javascript
const pusher = new Pusher("app-key", {
  wsHost: "<worker-host>",
  wssPort: 443,
  forceTLS: true,
  enabledTransports: ["ws", "wss"]
});
```

## Operational Notes

- Cloudflare KV is not a Redis replacement for Soketi.rs realtime state. KV is useful for slow-changing metadata, but Redis and NATS provide the low-latency pub/sub and shared state behavior needed for multi-instance realtime fan-out.
- Durable Object app config is a deployment wrapper feature. The native binary still uses the `array` app manager inside the container.
- Use `array` mode when app config changes through deployment. Use `durable-object` mode when operators need runtime app-list updates through guarded admin endpoints.
- Keep `SOKETI_ADMIN_BEARER_TOKEN` and `SOKETI_METRICS_BEARER_TOKEN` as secrets.
- For production multi-instance deployments, configure Redis or NATS before increasing `SOKETI_CONTAINER_INSTANCE_COUNT`.

## References

- [Cloudflare Containers getting started](https://developers.cloudflare.com/containers/get-started/)
- [Cloudflare Containers image management](https://developers.cloudflare.com/containers/image-management/)
- [Wrangler configuration for Containers and Durable Objects](https://developers.cloudflare.com/workers/wrangler/configuration/)
- [Cloudflare Durable Objects overview](https://developers.cloudflare.com/durable-objects/)
