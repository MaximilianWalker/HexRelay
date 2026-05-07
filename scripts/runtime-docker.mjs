import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptsDir, "..");
const composeFile = path.join(root, "infra", "docker-compose.runtime-test.yml");
const runDir = path.join(root, ".local-run");
const runtimeStatePath = path.join(runDir, "runtime-state.json");
const networkScript = path.join(scriptsDir, "network.mjs");
const projectName = "hexrelay-runtime";
const networkName = "hexrelay-runtime_simulation";
const nextEnvPath = path.join(root, "apps", "web", "next-env.d.ts");
const stableNextEnv = `/// <reference types="next" />
/// <reference types="next/image-types/global" />

// NOTE: This file should not be edited
// see https://nextjs.org/docs/app/api-reference/config/typescript for more information.
`;
const runtimeDataVolumes = [
  "hexrelay-runtime_runtime-alice-postgres-data",
  "hexrelay-runtime_runtime-alice-redis-data",
  "hexrelay-runtime_runtime-alice-minio-data",
  "hexrelay-runtime_runtime-bob-postgres-data",
  "hexrelay-runtime_runtime-bob-redis-data",
  "hexrelay-runtime_runtime-bob-minio-data",
];

const instances = [
  {
    id: "alice-node",
    seedPersona: "alice.primary",
    containerName: "hexrelay-runtime-alice-node",
    apiContainerName: "hexrelay-runtime-alice-api",
    realtimeContainerName: "hexrelay-runtime-alice-realtime",
    webContainerName: "hexrelay-runtime-alice-web",
    apiPort: 18080,
    realtimePort: 18081,
    webPort: 3002,
    apiUrl: "http://127.0.0.1:18080",
    realtimeUrl: "http://127.0.0.1:18081",
    realtimeWsUrl: "ws://127.0.0.1:18081/ws",
    webUrl: "http://127.0.0.1:3002",
  },
  {
    id: "bob-node",
    seedPersona: "bob.primary",
    containerName: "hexrelay-runtime-bob-node",
    apiContainerName: "hexrelay-runtime-bob-api",
    realtimeContainerName: "hexrelay-runtime-bob-realtime",
    webContainerName: "hexrelay-runtime-bob-web",
    apiPort: 18180,
    realtimePort: 18181,
    webPort: 3012,
    apiUrl: "http://127.0.0.1:18180",
    realtimeUrl: "http://127.0.0.1:18181",
    realtimeWsUrl: "ws://127.0.0.1:18181/ws",
    webUrl: "http://127.0.0.1:3012",
  },
];

function usage() {
  return "Usage: runtime-docker.mjs up|down|status|smoke [--seed-profile dm-basic] [--json] [--force]";
}

function parseArgs(args) {
  const options = {
    command: "status",
    seedProfile: "",
    json: false,
    force: false,
    help: false,
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    switch (arg) {
      case "up":
      case "down":
      case "status":
      case "smoke":
        options.command = arg;
        break;
      case "--seed-profile":
      case "-SeedProfile":
        options.seedProfile = requireValue(args, ++index, arg);
        break;
      case "--json":
      case "-Json":
        options.json = true;
        break;
      case "--force":
      case "-Force":
        options.force = true;
        break;
      case "--help":
      case "-Help":
      case "-h":
        options.help = true;
        break;
      default:
        throw new Error(`unknown runtime-docker option: ${arg}\n${usage()}`);
    }
  }

  return options;
}

function requireValue(args, index, flag) {
  const value = args[index];
  if (!value || value.startsWith("-")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function readJsonIfExists(filePath) {
  if (!fs.existsSync(filePath)) {
    return null;
  }
  return JSON.parse(fs.readFileSync(filePath, "utf8").replace(/^\uFEFF/, ""));
}

function hasLiveHostProcesses(state) {
  for (const instance of state?.instances ?? []) {
    for (const pid of [instance.apiPid, instance.realtimePid, instance.webPid]) {
      if (Number.isInteger(pid) && pid > 0) {
        try {
          process.kill(pid, 0);
          return true;
        } catch {
        }
      }
    }
  }
  return false;
}

function ensureCanWriteRuntimeState() {
  const existing = readJsonIfExists(runtimeStatePath);
  if (!existing) {
    return;
  }
  if (existing.runtimeKind === "docker-test") {
    return;
  }
  if (hasLiveHostProcesses(existing)) {
    throw new Error("A host-process runtime is active. Stop it before starting the Docker runtime test stack.");
  }
  fs.rmSync(runtimeStatePath, { force: true });
}

function docker(args, options = {}) {
  const result = spawnSync("docker", args, {
    cwd: root,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
    shell: false,
  });

  if (result.error) {
    throw new Error(`failed to start docker: ${result.error.message}`);
  }
  if (result.status !== 0 && !options.allowFailure) {
    const output = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(output || `docker ${args.join(" ")} failed`);
  }
  return result;
}

function compose(args, options = {}) {
  return docker(["compose", "-p", projectName, "-f", composeFile, ...args], options);
}

function composeWithProfiles(profiles, args, options = {}) {
  const profileArgs = profiles.flatMap((profile) => ["--profile", profile]);
  return docker(["compose", "-p", projectName, "-f", composeFile, ...profileArgs, ...args], options);
}

function runNetwork(args, options = {}) {
  const result = spawnSync(process.execPath, [networkScript, ...args], {
    cwd: root,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
    shell: false,
  });
  if (result.status !== 0 && !options.allowFailure) {
    const output = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(output || `node ${path.relative(root, networkScript)} ${args.join(" ")} failed`);
  }
  return result;
}

function runNetworkJson(args) {
  const result = runNetwork([...args, "--json"], { capture: true });
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`failed to parse network JSON for '${args.join(" ")}': ${error.message}`);
  }
}

function requireSmokeEvent(result, description, predicate) {
  const event = (result.events ?? []).find(predicate);
  if (!event) {
    throw new Error(`network smoke assertion failed: ${description}`);
  }
  return event;
}

function requireChangedSmokeEvent(result, description, predicate) {
  const event = requireSmokeEvent(result, description, predicate);
  if (event.changed !== true) {
    throw new Error(`network smoke assertion failed: ${description} did not change Docker state`);
  }
  return event;
}

function assertOfflineSmoke(applyResult, resetResult) {
  const [alice] = instances;
  requireChangedSmokeEvent(
    applyResult,
    "offline-alice disconnects alice-node from the simulation network",
    (event) => event.type === "disconnect"
      && event.target === alice.id
      && event.containerName === alice.containerName
      && event.networkName === networkName
      && event.disconnected === true,
  );
  requireChangedSmokeEvent(
    resetResult,
    "offline-alice reset reconnects alice-node to the simulation network",
    (event) => event.type === "connect"
      && event.containerName === alice.containerName
      && event.networkName === networkName,
  );
}

function assertPartitionSmoke(applyResult, resetResult) {
  const createEvents = (applyResult.events ?? []).filter(
    (event) => event.type === "create-network"
      && event.changed === true
      && String(event.networkName ?? "").startsWith("hexrelay_partition_"),
  );
  if (createEvents.length !== instances.length) {
    throw new Error(`network smoke assertion failed: partition created ${createEvents.length} network(s), expected ${instances.length}`);
  }

  for (const instance of instances) {
    requireSmokeEvent(
      applyResult,
      `partition disconnects ${instance.id} from the simulation network and connects it to a partition network`,
      (event) => event.type === "partition"
        && event.target === instance.id
        && event.containerName === instance.containerName
        && event.networkName === networkName
        && event.connectedPartition === true
        && event.disconnectedBase === true,
    );
    requireChangedSmokeEvent(
      resetResult,
      `partition reset reconnects ${instance.id} to the simulation network`,
      (event) => event.type === "connect"
        && event.containerName === instance.containerName
        && event.networkName === networkName,
    );
  }

  const partitionDisconnects = (resetResult.events ?? []).filter(
    (event) => event.type === "disconnect"
      && event.changed === true
      && String(event.networkName ?? "").startsWith("hexrelay_partition_"),
  );
  if (partitionDisconnects.length !== instances.length) {
    throw new Error(`network smoke assertion failed: partition reset disconnected ${partitionDisconnects.length} partition network attachment(s), expected ${instances.length}`);
  }

  const removedNetworks = (resetResult.events ?? []).filter(
    (event) => event.type === "remove-network"
      && event.changed === true
      && String(event.networkName ?? "").startsWith("hexrelay_partition_"),
  );
  if (removedNetworks.length !== instances.length) {
    throw new Error(`network smoke assertion failed: partition reset removed ${removedNetworks.length} partition network(s), expected ${instances.length}`);
  }
}

function restoreStableNextEnv() {
  fs.writeFileSync(nextEnvPath, stableNextEnv);
}

function dockerVolumeLabels(volumeName) {
  const result = docker(["volume", "inspect", volumeName, "--format", "{{json .Labels}}"], {
    allowFailure: true,
    capture: true,
  });
  if (result.status !== 0) {
    return null;
  }
  return JSON.parse((result.stdout || "null").trim() || "null");
}

function removeOwnedDataVolumes() {
  for (const volumeName of runtimeDataVolumes) {
    const labels = dockerVolumeLabels(volumeName);
    if (!labels) {
      continue;
    }
    if (labels["com.docker.compose.project"] !== projectName) {
      console.warn(`[runtime-docker] Skipping volume '${volumeName}' because it is not owned by compose project '${projectName}'.`);
      continue;
    }
    docker(["volume", "rm", "-f", volumeName], { allowFailure: true });
  }
}

async function httpOk(url) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 3000);
  try {
    const response = await fetch(url, { signal: controller.signal });
    return response.status === 200;
  } catch {
    return false;
  } finally {
    clearTimeout(timeout);
  }
}

async function waitFor(label, probe, attempts = 600) {
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    if (await probe()) {
      console.log(`[runtime-docker] ${label} is ready`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(`${label} did not become ready after ${attempts} seconds`);
}

async function waitForStack() {
  for (const instance of instances) {
    await waitFor(`${instance.id} api`, () => httpOk(`${instance.apiUrl}/health`));
    await waitFor(`${instance.id} realtime`, () => httpOk(`${instance.realtimeUrl}/health`));
    await waitFor(`${instance.id} web`, async () => {
      return (await httpOk(instance.webUrl)) || (await httpOk(`${instance.webUrl}/onboarding/identity`));
    });
  }
}

function writeRuntimeState(seedProfile) {
  fs.mkdirSync(runDir, { recursive: true });
  const state = {
    profile: "docker-dual",
    runtimeKind: "docker-test",
    seedProfile: seedProfile || null,
    infraMode: "docker-compose-runtime-test",
    networkName,
    composeProject: projectName,
    composeFile: path.relative(root, composeFile),
    startedAt: new Date().toISOString(),
    root,
    instances: instances.map((instance) => ({ ...instance, networkName })),
  };
  fs.writeFileSync(runtimeStatePath, `${JSON.stringify(state, null, 2)}\n`);
  return state;
}

function printResult(result, json) {
  if (json) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  if (result.status === "up") {
    console.log(`[runtime-docker] Docker runtime test stack is ready.`);
    for (const instance of result.instances) {
      console.log(`  [${instance.id}] API:      ${instance.apiUrl}`);
      console.log(`  [${instance.id}] Realtime: ${instance.realtimeUrl}`);
      console.log(`  [${instance.id}] WS:       ${instance.realtimeWsUrl}`);
      console.log(`  [${instance.id}] Web:      ${instance.webUrl}`);
    }
  } else if (result.status === "down") {
    console.log("[runtime-docker] Docker runtime test stack stopped.");
  } else if (result.status === "inactive") {
    console.log("[runtime-docker] Docker runtime test stack is not active.");
  } else if (result.status === "smoke-passed") {
    console.log("[runtime-docker] Runtime Docker smoke passed.");
  }
}

function seedProfile(profileName, node) {
  composeWithProfiles(["tools"], [
    "run",
    "--rm",
    `${node}-seed`,
    "cargo",
    "run",
    "-p",
    "api-rs",
    "--bin",
    "seed_dev",
    "--",
    "--profile",
    profileName,
    "--json",
  ]);
}

async function up(options) {
  ensureCanWriteRuntimeState();
  try {
    compose(["up", "-d", "--remove-orphans"]);
    await waitForStack();
  } catch (error) {
    restoreStableNextEnv();
    throw error;
  }
  restoreStableNextEnv();
  if (options.seedProfile) {
    seedProfile(options.seedProfile, "alice");
    seedProfile(options.seedProfile, "bob");
  }
  const state = writeRuntimeState(options.seedProfile);
  return { status: "up", ...state };
}

function down(options = {}) {
  const existing = readJsonIfExists(runtimeStatePath);
  let removeRuntimeState = false;
  let resetError = null;
  if (existing?.runtimeKind === "docker-test") {
    removeRuntimeState = true;
    try {
      runNetwork(["--reset", ...(options.force ? ["--force"] : [])], { allowFailure: options.force });
    } catch (error) {
      resetError = error;
    }
  }
  let composeSucceeded = false;
  try {
    compose(["down", "--remove-orphans"]);
    composeSucceeded = true;
  } finally {
    restoreStableNextEnv();
    if (composeSucceeded) {
      if (resetError) {
        runNetwork(["--reset", "--force"], { allowFailure: true });
      }
      if (removeRuntimeState) {
        fs.rmSync(runtimeStatePath, { force: true });
      }
      removeOwnedDataVolumes();
    }
  }
  if (resetError) {
    throw resetError;
  }
  return { status: "down" };
}

async function status() {
  const state = readJsonIfExists(runtimeStatePath);
  if (state?.runtimeKind !== "docker-test") {
    return { status: "inactive" };
  }
  const checks = [];
  for (const instance of state.instances ?? []) {
    checks.push({ id: instance.id, service: "api", ok: await httpOk(`${instance.apiUrl}/health`) });
    checks.push({ id: instance.id, service: "realtime", ok: await httpOk(`${instance.realtimeUrl}/health`) });
    checks.push({ id: instance.id, service: "web", ok: await httpOk(instance.webUrl) });
  }
  return { status: "up", ...state, checks };
}

async function smoke(options) {
  try {
    await up({ ...options, seedProfile: options.seedProfile || "dm-basic" });
    const offlineApply = runNetworkJson(["--profile", "offline-alice"]);
    const offlineReset = runNetworkJson(["--reset"]);
    assertOfflineSmoke(offlineApply, offlineReset);
    await waitForStack();
    const partitionApply = runNetworkJson(["--profile", "partition-alice-bob"]);
    const partitionReset = runNetworkJson(["--reset"]);
    assertPartitionSmoke(partitionApply, partitionReset);
    await waitForStack();
    return { status: "smoke-passed" };
  } catch (error) {
    compose(["logs", "--tail", "80", "alice-api", "alice-realtime", "bob-api", "bob-realtime", "alice-web", "bob-web"], { allowFailure: true });
    throw error;
  } finally {
    down({ force: true });
  }
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    console.log(usage());
    return;
  }

  if (options.command === "up") {
    printResult(await up(options), options.json);
    return;
  }
  if (options.command === "down") {
    printResult(down(options), options.json);
    return;
  }
  if (options.command === "status") {
    printResult(await status(), options.json);
    return;
  }
  if (options.command === "smoke") {
    printResult(await smoke(options), options.json);
    return;
  }
  throw new Error(`unknown runtime-docker command '${options.command}'`);
}

main().catch((error) => {
  console.error(`[runtime-docker] ERROR: ${error.message}`);
  process.exit(1);
});
