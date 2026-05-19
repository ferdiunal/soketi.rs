import { Container, getContainer, getRandom, switchPort } from "@cloudflare/containers";
import { DurableObject } from "cloudflare:workers";

const SOKETI_PORT = 6001;
const METRICS_PORT = 9601;
const DEFAULT_CONTAINER_NAME = "soketi-rs";
const DEFAULT_APP_CONFIG_NAME = "default";
const APP_CONFIG_STORAGE_KEY = "apps";

type RoutingMode = "single" | "named" | "random";
type AppConfigDriver = "array" | "durable-object";

type SoketiApp = {
  id: string;
  key: string;
  secret: string;
  enabled?: boolean;
  enable_client_messages?: boolean;
  max_connections?: number;
  max_backend_events_per_second?: number;
  max_client_events_per_second?: number;
  max_read_requests_per_second?: number;
  [key: string]: unknown;
};

type SoketiContainerStub = DurableObjectStub & {
  startAndWaitForPorts(options?: {
    ports?: number | number[];
    startOptions?: {
      envVars?: Record<string, string>;
      enableInternet?: boolean;
    };
    cancellationOptions?: {
      portReadyTimeoutMS?: number;
    };
  }): Promise<void>;
  getState?(): Promise<unknown>;
  destroy?(): Promise<void>;
};

export interface Env {
  SOKETI_CONTAINER: DurableObjectNamespace<Container<Env>>;
  SOKETI_APP_CONFIG: DurableObjectNamespace<SoketiAppConfig>;
  SOKETI_CONTAINER_ROUTING?: string;
  SOKETI_CONTAINER_INSTANCE_NAME?: string;
  SOKETI_CONTAINER_INSTANCE_COUNT?: string;
  SOKETI_APP_CONFIG_DRIVER?: string;
  SOKETI_APP_CONFIG_INSTANCE_NAME?: string;
  SOKETI_ADMIN_BEARER_TOKEN?: string;
  SOKETI_EXPOSE_METRICS?: string;
  SOKETI_ALLOW_LOCAL_RANDOM_ROUTING?: string;
  SOKETI_METRICS_BEARER_TOKEN?: string;
  [key: string]: unknown;
}

export class SoketiContainer extends Container<Env> {
  defaultPort = SOKETI_PORT;
  requiredPorts = [SOKETI_PORT];
  sleepAfter = "30m";
  enableInternet = true;
  pingEndpoint = "localhost/ready";

  onStart() {
    console.log("soketi-rs container started");
  }

  onStop() {
    console.log("soketi-rs container stopped");
  }

  onError(error: unknown) {
    console.error("soketi-rs container error", error);
  }
}

export class SoketiAppConfig extends DurableObject<Env> {
  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname !== "/apps") {
      return jsonResponse({ error: "not found" }, 404);
    }

    if (request.method === "GET") {
      return jsonResponse({ apps: await this.getApps() });
    }

    if (request.method === "PUT") {
      const apps = await parseAppsRequest(request);
      await this.ctx.storage.put(APP_CONFIG_STORAGE_KEY, apps);
      return jsonResponse({ apps });
    }

    if (request.method === "DELETE") {
      await this.ctx.storage.delete(APP_CONFIG_STORAGE_KEY);
      return jsonResponse({ apps: await this.getApps() });
    }

    return jsonResponse({ error: "method not allowed" }, 405);
  }

  private async getApps(): Promise<SoketiApp[]> {
    const storedApps = await this.ctx.storage.get<SoketiApp[]>(APP_CONFIG_STORAGE_KEY);
    if (storedApps && storedApps.length > 0) {
      return storedApps;
    }

    const seededApps = appsFromEnv(this.env);
    if (seededApps.length > 0) {
      await this.ctx.storage.put(APP_CONFIG_STORAGE_KEY, seededApps);
    }

    return seededApps;
  }
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    try {
      if (url.pathname === "/__cf/soketi/container-state") {
        return await containerStateResponse(env);
      }

      if (url.pathname === "/__cf/soketi/container-restart") {
        const authResponse = validateAdminAccess(request, env);
        if (authResponse) {
          return authResponse;
        }

        return await restartContainerResponse(env);
      }

      if (url.pathname === "/__cf/soketi/app-config/apps") {
        const authResponse = validateAdminAccess(request, env);
        if (authResponse) {
          return authResponse;
        }

        return await appConfigResponse(request, env);
      }

      if (url.pathname === "/metrics") {
        const metricsResponse = validateMetricsAccess(request, env);
        if (metricsResponse) {
          return metricsResponse;
        }
      }

      const container = await routeContainer(env);
      await ensureContainerStarted(container, env);

      if (url.pathname === "/metrics") {
        return container.fetch(switchPort(request, METRICS_PORT));
      }

      return container.fetch(request);
    } catch (error) {
      console.error("soketi-rs container proxy error", error);

      return jsonResponse(
        {
          error: "soketi-rs container is unavailable",
          detail: error instanceof Error ? error.message : String(error)
        },
        503
      );
    }
  }
};

async function routeContainer(env: Env): Promise<SoketiContainerStub> {
  const routingMode = parseRoutingMode(env.SOKETI_CONTAINER_ROUTING);

  if (routingMode === "random") {
    assertRandomRoutingAllowed(env);
    return (await getRandom(
      env.SOKETI_CONTAINER,
      parseInstanceCount(env.SOKETI_CONTAINER_INSTANCE_COUNT)
    )) as SoketiContainerStub;
  }

  const instanceName = stringEnv(env.SOKETI_CONTAINER_INSTANCE_NAME) || DEFAULT_CONTAINER_NAME;
  return getContainer(env.SOKETI_CONTAINER, instanceName) as SoketiContainerStub;
}

async function ensureContainerStarted(container: SoketiContainerStub, env: Env): Promise<void> {
  await container.startAndWaitForPorts({
    ports: [SOKETI_PORT],
    startOptions: {
      envVars: await containerEnv(env),
      enableInternet: true
    },
    cancellationOptions: {
      portReadyTimeoutMS: 30_000
    }
  });
}

async function containerEnv(env: Env): Promise<Record<string, string>> {
  const values: Record<string, string> = {
    PUSHER_HOST: "0.0.0.0",
    PUSHER_PORT: String(SOKETI_PORT),
    PUSHER_SSL_ENABLED: "false",
    PUSHER_METRICS_ENABLED: "true",
    PUSHER_METRICS_PORT: String(METRICS_PORT)
  };

  for (const [key, value] of Object.entries(env)) {
    if (!shouldForwardEnv(key) || typeof value !== "string") {
      continue;
    }

    values[key] = value;
  }

  if (parseAppConfigDriver(env.SOKETI_APP_CONFIG_DRIVER) === "durable-object") {
    const apps = await loadDurableObjectApps(env);
    values.PUSHER_APP_MANAGER_DRIVER = "array";
    values.PUSHER_APP_MANAGER_ARRAY_APPS = JSON.stringify(apps);
  }

  return values;
}

function shouldForwardEnv(key: string): boolean {
  return (
    key.startsWith("PUSHER_") ||
    key.startsWith("AWS_") ||
    key === "RUST_LOG" ||
    key === "RUST_BACKTRACE"
  );
}

function validateAdminAccess(request: Request, env: Env): Response | null {
  const token = stringEnv(env.SOKETI_ADMIN_BEARER_TOKEN);
  if (!token) {
    return new Response("Not found", { status: 404 });
  }

  const authorization = request.headers.get("authorization");
  if (authorization === `Bearer ${token}`) {
    return null;
  }

  return new Response("Unauthorized", {
    status: 401,
    headers: {
      "www-authenticate": "Bearer"
    }
  });
}

function validateMetricsAccess(request: Request, env: Env): Response | null {
  if (!isTruthy(env.SOKETI_EXPOSE_METRICS)) {
    return new Response("Not found", { status: 404 });
  }

  const token = stringEnv(env.SOKETI_METRICS_BEARER_TOKEN);
  if (!token) {
    return null;
  }

  const authorization = request.headers.get("authorization");
  if (authorization === `Bearer ${token}`) {
    return null;
  }

  return new Response("Unauthorized", {
    status: 401,
    headers: {
      "www-authenticate": "Bearer"
    }
  });
}

async function appConfigResponse(request: Request, env: Env): Promise<Response> {
  const configRequest = new Request("https://soketi-app-config.internal/apps", request);
  const response = await appConfigStub(env).fetch(configRequest);

  if (request.method === "PUT" && new URL(request.url).searchParams.get("restart") === "true") {
    await restartConfiguredContainer(env);
  }

  return response;
}

async function restartContainerResponse(env: Env): Promise<Response> {
  await restartConfiguredContainer(env);
  return jsonResponse({ restarted: true });
}

async function restartConfiguredContainer(env: Env): Promise<void> {
  const container = await routeContainer(env);

  if (typeof container.destroy !== "function") {
    throw new Error("Container destroy method is unavailable");
  }

  await container.destroy();
}

async function containerStateResponse(env: Env): Promise<Response> {
  const container = await routeContainer(env);

  if (typeof container.getState !== "function") {
    return jsonResponse({ state: "unknown" });
  }

  return jsonResponse({ state: await container.getState() });
}

async function loadDurableObjectApps(env: Env): Promise<SoketiApp[]> {
  const response = await appConfigStub(env).fetch("https://soketi-app-config.internal/apps");

  if (!response.ok) {
    throw new Error(`Failed to load Durable Object app config: HTTP ${response.status}`);
  }

  const payload = await response.json<{ apps?: unknown }>();
  return validateApps(payload.apps);
}

function appConfigStub(env: Env): DurableObjectStub<SoketiAppConfig> {
  const instanceName = stringEnv(env.SOKETI_APP_CONFIG_INSTANCE_NAME) || DEFAULT_APP_CONFIG_NAME;
  return env.SOKETI_APP_CONFIG.getByName(instanceName);
}

async function parseAppsRequest(request: Request): Promise<SoketiApp[]> {
  const payload = await request.json<unknown>();
  const apps = Array.isArray(payload)
    ? payload
    : isRecord(payload) && Array.isArray(payload.apps)
      ? payload.apps
      : undefined;

  return validateApps(apps);
}

function appsFromEnv(env: Env): SoketiApp[] {
  const appsJson = stringEnv(env.PUSHER_APP_MANAGER_ARRAY_APPS);
  if (appsJson) {
    try {
      return validateApps(JSON.parse(appsJson));
    } catch (error) {
      throw new Error(
        `Invalid PUSHER_APP_MANAGER_ARRAY_APPS: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  const id = stringEnv(env.PUSHER_DEFAULT_APP_ID);
  const key = stringEnv(env.PUSHER_DEFAULT_APP_KEY);
  const secret = stringEnv(env.PUSHER_DEFAULT_APP_SECRET);

  if (!id || !key || !secret) {
    return [];
  }

  return [
    {
      id,
      key,
      secret,
      enabled: true,
      enable_client_messages: false
    }
  ];
}

function validateApps(value: unknown): SoketiApp[] {
  if (!Array.isArray(value)) {
    throw new Error("apps must be an array");
  }

  return value.map((item, index) => {
    if (!isRecord(item)) {
      throw new Error(`apps[${index}] must be an object`);
    }

    const id = stringEnv(item.id);
    const key = stringEnv(item.key);
    const secret = stringEnv(item.secret);

    if (!id || !key || !secret) {
      throw new Error(`apps[${index}] must include id, key, and secret`);
    }

    return {
      ...item,
      id,
      key,
      secret
    } as SoketiApp;
  });
}

function parseRoutingMode(value: unknown): RoutingMode {
  const mode = stringEnv(value).toLowerCase();

  if (mode === "random" || mode === "named" || mode === "single") {
    return mode;
  }

  return "single";
}

function parseAppConfigDriver(value: unknown): AppConfigDriver {
  const driver = stringEnv(value).toLowerCase();

  if (driver === "durable-object") {
    return "durable-object";
  }

  return "array";
}

function parseInstanceCount(value: unknown): number {
  const parsed = Number.parseInt(stringEnv(value) || "1", 10);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : 1;
}

function assertRandomRoutingAllowed(env: Env): void {
  const adapterDriver = stringEnv(env.PUSHER_ADAPTER_DRIVER).toLowerCase();

  if ((adapterDriver === "" || adapterDriver === "local") && !isTruthy(env.SOKETI_ALLOW_LOCAL_RANDOM_ROUTING)) {
    throw new Error(
      "SOKETI_CONTAINER_ROUTING=random requires PUSHER_ADAPTER_DRIVER=redis or nats. " +
        "Set SOKETI_ALLOW_LOCAL_RANDOM_ROUTING=true only for explicit testing."
    );
  }
}

function stringEnv(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isTruthy(value: unknown): boolean {
  return ["1", "true", "yes", "on"].includes(stringEnv(value).toLowerCase());
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8"
    }
  });
}
