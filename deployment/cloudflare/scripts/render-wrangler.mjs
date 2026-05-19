import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const configPath = path.join(root, "wrangler.jsonc");
const generatedPath = path.join(root, "wrangler.generated.jsonc");

const config = JSON.parse(await readFile(configPath, "utf8"));
const container = config.containers?.[0];

if (!container) {
  throw new Error("wrangler.jsonc must define at least one container");
}

const image =
  process.env.SOKETI_IMAGE ??
  (process.env.SOKETI_IMAGE_TAG
    ? `docker.io/funal/soketi-rs:${process.env.SOKETI_IMAGE_TAG}`
    : undefined);

if (image) {
  container.image = image;
}

if (process.env.SOKETI_WORKER_NAME) {
  config.name = process.env.SOKETI_WORKER_NAME;
}

if (process.env.SOKETI_CONTAINER_MAX_INSTANCES) {
  const maxInstances = Number.parseInt(process.env.SOKETI_CONTAINER_MAX_INSTANCES, 10);

  if (!Number.isInteger(maxInstances) || maxInstances < 1) {
    throw new Error("SOKETI_CONTAINER_MAX_INSTANCES must be a positive integer");
  }

  container.max_instances = maxInstances;
}

for (const key of [
  "SOKETI_CONTAINER_ROUTING",
  "SOKETI_CONTAINER_INSTANCE_NAME",
  "SOKETI_CONTAINER_INSTANCE_COUNT",
  "SOKETI_APP_CONFIG_DRIVER",
  "SOKETI_APP_CONFIG_INSTANCE_NAME",
  "SOKETI_EXPOSE_METRICS",
  "SOKETI_ALLOW_LOCAL_RANDOM_ROUTING",
  "PUSHER_ADAPTER_DRIVER",
  "PUSHER_CACHE_DRIVER",
  "PUSHER_RATE_LIMITER_DRIVER",
  "PUSHER_QUEUE_DRIVER",
  "PUSHER_METRICS_ENABLED",
  "PUSHER_CORS_ORIGINS"
]) {
  if (process.env[key]) {
    config.vars ??= {};
    config.vars[key] = process.env[key];
  }
}

await writeFile(generatedPath, `${JSON.stringify(config, null, 2)}\n`);

console.log(`Rendered ${path.relative(root, generatedPath)} with image ${container.image}`);
