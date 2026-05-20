import path from "node:path";
import process from "node:process";
import { rootDir as root, runDir } from "../../lib/paths.mjs";

export { root, runDir };

export const composeFile = path.join(root, "infra", "docker-compose.runtime-test.yml");
export const runtimeStatePath = path.join(runDir, "runtime-state.json");
export const networkScript = path.join(root, "scripts", "network.mjs");
export const projectName = "hexrelay-runtime";
export const networkName = "hexrelay-runtime_simulation";
export const realtimeInternalToken = "hexrelay-runtime-channel-dispatch-token-change-me";
export const nextEnvPath = path.join(root, "apps", "web", "next-env.d.ts");
export const stableNextEnv = `/// <reference types="next" />
/// <reference types="next/image-types/global" />

// NOTE: This file should not be edited
// see https://nextjs.org/docs/app/api-reference/config/typescript for more information.
`;

export const runtimeDataVolumes = [
  "hexrelay-runtime_runtime-alice-postgres-data",
  "hexrelay-runtime_runtime-alice-redis-data",
  "hexrelay-runtime_runtime-alice-minio-data",
  "hexrelay-runtime_runtime-bob-postgres-data",
  "hexrelay-runtime_runtime-bob-redis-data",
  "hexrelay-runtime_runtime-bob-minio-data",
];

function envPort(name, fallback) {
  const raw = process.env[name];
  if (raw === undefined || raw === "") {
    return fallback;
  }
  const port = Number(raw);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`${name} must be a TCP port between 1 and 65535.`);
  }
  return port;
}

export const ports = {
  toxiproxy: envPort("HEXRELAY_RUNTIME_TOXIPROXY_PORT", 18474),
  aliceApi: envPort("HEXRELAY_RUNTIME_ALICE_API_PORT", 18080),
  aliceRealtime: envPort("HEXRELAY_RUNTIME_ALICE_REALTIME_PORT", 18081),
  aliceWeb: envPort("HEXRELAY_RUNTIME_ALICE_WEB_PORT", 3002),
  bobApi: envPort("HEXRELAY_RUNTIME_BOB_API_PORT", 18180),
  bobRealtime: envPort("HEXRELAY_RUNTIME_BOB_REALTIME_PORT", 18181),
  bobWeb: envPort("HEXRELAY_RUNTIME_BOB_WEB_PORT", 3012),
};

export const toxiproxyUrl = `http://127.0.0.1:${ports.toxiproxy}`;

export const instances = [
  {
    id: "alice-server",
    seedPersona: "alice.primary",
    containerName: "hexrelay-runtime-alice-server",
    apiContainerName: "hexrelay-runtime-alice-api",
    realtimeContainerName: "hexrelay-runtime-alice-realtime",
    webContainerName: "hexrelay-runtime-alice-web",
    apiPort: ports.aliceApi,
    realtimePort: ports.aliceRealtime,
    webPort: ports.aliceWeb,
    apiUrl: `http://127.0.0.1:${ports.aliceApi}`,
    realtimeUrl: `http://127.0.0.1:${ports.aliceRealtime}`,
    realtimeWsUrl: `ws://127.0.0.1:${ports.aliceRealtime}/ws`,
    webUrl: `http://127.0.0.1:${ports.aliceWeb}`,
  },
  {
    id: "bob-server",
    seedPersona: "bob.primary",
    containerName: "hexrelay-runtime-bob-server",
    apiContainerName: "hexrelay-runtime-bob-api",
    realtimeContainerName: "hexrelay-runtime-bob-realtime",
    webContainerName: "hexrelay-runtime-bob-web",
    apiPort: ports.bobApi,
    realtimePort: ports.bobRealtime,
    webPort: ports.bobWeb,
    apiUrl: `http://127.0.0.1:${ports.bobApi}`,
    realtimeUrl: `http://127.0.0.1:${ports.bobRealtime}`,
    realtimeWsUrl: `ws://127.0.0.1:${ports.bobRealtime}/ws`,
    webUrl: `http://127.0.0.1:${ports.bobWeb}`,
  },
];

export const toxiproxyProxies = [
  {
    name: "alice-to-bob-api",
    sourceId: "alice-server",
    targetId: "bob-server",
    kind: "api",
    listen: "0.0.0.0:28080",
    upstream: "bob-server:8080",
    url: "http://toxiproxy:28080",
  },
  {
    name: "alice-to-bob-realtime",
    sourceId: "alice-server",
    targetId: "bob-server",
    kind: "realtime",
    listen: "0.0.0.0:28081",
    upstream: "bob-server:8081",
    url: "http://toxiproxy:28081",
  },
  {
    name: "bob-to-alice-api",
    sourceId: "bob-server",
    targetId: "alice-server",
    kind: "api",
    listen: "0.0.0.0:28180",
    upstream: "alice-server:8080",
    url: "http://toxiproxy:28180",
  },
  {
    name: "bob-to-alice-realtime",
    sourceId: "bob-server",
    targetId: "alice-server",
    kind: "realtime",
    listen: "0.0.0.0:28181",
    upstream: "alice-server:8081",
    url: "http://toxiproxy:28181",
  },
];
