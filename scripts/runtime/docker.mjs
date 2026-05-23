import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { readJsonIfExists } from "../lib/json.mjs";
import { parseArgs, usage } from "./docker/args.mjs";
import {
  composeFile,
  instances,
  networkName,
  nextEnvPath,
  projectName,
  realtimeInternalToken,
  root,
  runDir,
  runtimeDataVolumes,
  runtimeStatePath,
  stableNextEnv,
  toxiproxyProxies,
  toxiproxyUrl,
} from "./docker/config.mjs";
import {
  appendEvidenceEvent,
  prepareEvidenceDir,
  resolveEvidenceDir,
  writeEvidenceJson,
  writeEvidenceVerdict,
} from "./docker/evidence.mjs";
import {
  assertAppFaultSmoke,
  assertOfflineSmoke,
  assertPartitionSmoke,
  assertToxiproxyLatencySmoke,
  assertToxiproxyTimeoutSmoke,
} from "./docker/smoke-assertions.mjs";
import {
  assertPeerReachability,
  assertToxiproxyBlocked,
  assertToxiproxyLatency,
  assertToxiproxyNoToxics,
  assertToxiproxyPeerReachability,
  compose,
  composeWithProfiles,
  docker,
  httpOk,
  populateToxiproxy,
  runNetwork,
  runNetworkJson,
  setJsonOutputMode,
  waitForStack,
} from "./docker/stack.mjs";

async function writeRuntimeStatusEvidence(evidenceDir, fileName) {
  if (!evidenceDir) {
    return;
  }
  writeEvidenceJson(evidenceDir, fileName, await status());
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

function restoreStableNextEnv() {
  try {
    fs.writeFileSync(nextEnvPath, stableNextEnv);
  } catch (error) {
    if (error?.code !== "EACCES" && error?.code !== "EPERM") {
      throw error;
    }
    if (!repairNextEnvOwnership()) {
      throw error;
    }
    fs.writeFileSync(nextEnvPath, stableNextEnv);
  }
}

function repairNextEnvOwnership() {
  if (process.platform === "win32" || typeof process.getuid !== "function" || typeof process.getgid !== "function") {
    return false;
  }

  const uid = process.getuid();
  const gid = process.getgid();
  const result = docker(
    [
      "run",
      "--rm",
      "-v",
      `${root}:/workspace`,
      "alpine:3.20",
      "sh",
      "-c",
      `chown ${uid}:${gid} /workspace/apps/web/next-env.d.ts 2>/dev/null || chmod a+rw /workspace/apps/web/next-env.d.ts`,
    ],
    { allowFailure: true, capture: true },
  );
  return result.status === 0;
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

function writeRuntimeState(seedProfile) {
  fs.mkdirSync(runDir, { recursive: true });
  const state = {
    profile: "docker-dual",
    runtimeKind: "docker-test",
    seedProfile: seedProfile || null,
    infraMode: "docker-compose-runtime-test",
    networkName,
    toxiproxy: {
      url: toxiproxyUrl,
      proxies: toxiproxyProxies,
    },
    composeProject: projectName,
    composeFile: path.relative(root, composeFile),
    realtimeInternalToken,
    startedAt: new Date().toISOString(),
    root,
    instances: instances.map((instance) => ({ ...instance, networkName, realtimeInternalToken })),
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
    console.log(`  [toxiproxy] API:    ${toxiproxyUrl}`);
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
  } else if (result.status === "runtime-smoke-passed") {
    console.log("[runtime-docker] Runtime health smoke passed.");
  } else if (result.status === "network-smoke-passed") {
    console.log("[runtime-docker] Runtime network smoke passed.");
  } else if (result.status === "smoke-passed") {
    console.log("[runtime-docker] Runtime Docker smoke passed.");
  }
  if (result.evidenceDir) {
    console.log(`[runtime-docker] Evidence: ${result.evidenceDir}`);
  }
}

function seedProfile(profileName, server) {
  composeWithProfiles(["tools"], [
    "run",
    "--rm",
    `${server}-seed`,
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
    await populateToxiproxy();
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
  const activeToxiproxyUrl = state.toxiproxy?.url ?? toxiproxyUrl;
  const checks = [];
  checks.push({ service: "toxiproxy", ok: await httpOk(`${activeToxiproxyUrl}/version`) });
  for (const instance of state.instances ?? []) {
    checks.push({ id: instance.id, service: "api", ok: await httpOk(`${instance.apiUrl}/health`) });
    checks.push({ id: instance.id, service: "realtime", ok: await httpOk(`${instance.realtimeUrl}/health`) });
    checks.push({ id: instance.id, service: "web", ok: await httpOk(instance.webUrl) });
  }
  return { status: "up", ...state, checks };
}

async function smoke(options) {
  const scope = options.scope || "all";
  const evidenceDir = resolveEvidenceDir(options.evidenceDir);
  prepareEvidenceDir(evidenceDir);
  writeEvidenceJson(evidenceDir, "scenario-config.json", {
    scope,
    seedProfile: options.seedProfile || "dm-basic",
    runtimeProfile: "docker-dual",
    profiles: scope === "runtime"
      ? []
      : ["offline-alice", "partition-alice-bob", "high-latency", "packet-loss", "flaky-mobile"],
    startedAt: new Date().toISOString(),
  });

  let result = null;
  let smokeError = null;
  try {
    await up({ ...options, seedProfile: options.seedProfile || "dm-basic" });
    await writeRuntimeStatusEvidence(evidenceDir, "runtime-status-before.json");
    assertPeerReachability(true, "baseline");
    assertToxiproxyPeerReachability(true, "baseline");
    appendEvidenceEvent(evidenceDir, { type: "observe", phase: "baseline", check: "peer-reachability", reachable: true });
    appendEvidenceEvent(evidenceDir, { type: "observe", phase: "baseline", check: "toxiproxy-peer-reachability", reachable: true });

    if (scope === "runtime") {
      await writeRuntimeStatusEvidence(evidenceDir, "runtime-status-after.json");
      result = {
        status: "runtime-smoke-passed",
        evidenceDir: evidenceDir ? path.relative(root, evidenceDir) : undefined,
      };
    } else {
      const offlineApply = runNetworkJson(["--profile", "offline-alice"]);
      appendEvidenceEvent(evidenceDir, { type: "network-apply", profile: "offline-alice", result: offlineApply });
      assertPeerReachability(false, "offline-alice");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "offline-alice", check: "peer-reachability", reachable: false });
      const offlineReset = runNetworkJson(["--reset"]);
      appendEvidenceEvent(evidenceDir, { type: "network-reset", profile: "offline-alice", result: offlineReset });
      assertOfflineSmoke(offlineApply, offlineReset);
      await waitForStack();
      assertPeerReachability(true, "offline reset");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "offline-alice", phase: "reset", check: "peer-reachability", reachable: true });
      const partitionApply = runNetworkJson(["--profile", "partition-alice-bob"]);
      appendEvidenceEvent(evidenceDir, { type: "network-apply", profile: "partition-alice-bob", result: partitionApply });
      assertPeerReachability(false, "partition-alice-bob");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "partition-alice-bob", check: "peer-reachability", reachable: false });
      const partitionReset = runNetworkJson(["--reset"]);
      appendEvidenceEvent(evidenceDir, { type: "network-reset", profile: "partition-alice-bob", result: partitionReset });
      assertPartitionSmoke(partitionApply, partitionReset);
      await waitForStack();
      assertPeerReachability(true, "partition reset");
      assertToxiproxyPeerReachability(true, "partition reset");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "partition-alice-bob", phase: "reset", check: "peer-reachability", reachable: true });
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "partition-alice-bob", phase: "reset", check: "toxiproxy-peer-reachability", reachable: true });
      const latencyApply = runNetworkJson(["--profile", "high-latency", "--target", "alice-server"]);
      appendEvidenceEvent(evidenceDir, { type: "network-apply", profile: "high-latency", result: latencyApply });
      assertToxiproxyLatency("alice-server", 150);
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "high-latency", check: "toxiproxy-latency", target: "alice-server", minimumMs: 150 });
      const latencyReset = runNetworkJson(["--reset"]);
      appendEvidenceEvent(evidenceDir, { type: "network-reset", profile: "high-latency", result: latencyReset });
      assertToxiproxyLatencySmoke(latencyApply, latencyReset);
      await assertToxiproxyNoToxics("alice-server", "latency reset");
      await waitForStack();
      assertToxiproxyPeerReachability(true, "latency reset");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "high-latency", phase: "reset", check: "toxiproxy-clear-and-reachability", reachable: true });
      const timeoutApply = runNetworkJson(["--profile", "packet-loss", "--target", "alice-server"]);
      appendEvidenceEvent(evidenceDir, { type: "network-apply", profile: "packet-loss", result: timeoutApply });
      assertToxiproxyBlocked("alice-server", "packet-loss");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "packet-loss", check: "toxiproxy-blocked", target: "alice-server" });
      const timeoutReset = runNetworkJson(["--reset"]);
      appendEvidenceEvent(evidenceDir, { type: "network-reset", profile: "packet-loss", result: timeoutReset });
      assertToxiproxyTimeoutSmoke(timeoutApply, timeoutReset);
      await assertToxiproxyNoToxics("alice-server", "timeout reset");
      await waitForStack();
      assertToxiproxyPeerReachability(true, "timeout reset");
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "packet-loss", phase: "reset", check: "toxiproxy-clear-and-reachability", reachable: true });
      const appFaultApply = runNetworkJson(["--profile", "flaky-mobile", "--target", "alice-server"]);
      appendEvidenceEvent(evidenceDir, { type: "network-apply", profile: "flaky-mobile", result: appFaultApply });
      const appFaultReset = runNetworkJson(["--reset"]);
      appendEvidenceEvent(evidenceDir, { type: "network-reset", profile: "flaky-mobile", result: appFaultReset });
      assertAppFaultSmoke(appFaultApply, appFaultReset);
      appendEvidenceEvent(evidenceDir, { type: "observe", profile: "flaky-mobile", phase: "reset", check: "app-fault-clear" });
      await waitForStack();
      await writeRuntimeStatusEvidence(evidenceDir, "runtime-status-after.json");
      result = {
        status: scope === "network" ? "network-smoke-passed" : "smoke-passed",
        evidenceDir: evidenceDir ? path.relative(root, evidenceDir) : undefined,
      };
    }
  } catch (error) {
    smokeError = error;
    compose(["logs", "--tail", "80", "toxiproxy", "alice-api", "alice-realtime", "bob-api", "bob-realtime", "alice-web", "bob-web"], { allowFailure: true });
  }

  let cleanupError = null;
  try {
    down({ force: true });
  } catch (error) {
    cleanupError = error;
  }

  if (smokeError || cleanupError) {
    const error = smokeError && cleanupError
      ? new Error(`${smokeError.message}; cleanup failed: ${cleanupError.message}`)
      : smokeError || cleanupError;
    appendEvidenceEvent(evidenceDir, { type: "verdict", status: "fail", error: error.message });
    writeEvidenceVerdict(evidenceDir, "fail", error);
    throw error;
  }

  appendEvidenceEvent(evidenceDir, { type: "verdict", status: "pass" });
  writeEvidenceVerdict(evidenceDir, "pass");
  return result;
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  setJsonOutputMode(options.json);
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
  throw new Error(`unknown runtime docker command '${options.command}'`);
}

main().catch((error) => {
  console.error(`[runtime-docker] ERROR: ${error.message}`);
  process.exit(1);
});
