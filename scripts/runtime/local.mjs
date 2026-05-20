import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import { readEnvFile } from "../lib/env.mjs";
import { httpOk, getFreePort, webReady } from "../lib/http.mjs";
import { readJsonIfExists, writeJson } from "../lib/json.mjs";
import {
  delay,
  isProcessAlive,
  isWindows,
  killProcessTree,
  listenerPid,
  processCommandLine,
  processName,
  processParentPid,
  uniquePids,
  withCargoOnPath,
} from "../lib/process.mjs";
import { rootDir as root, runDir } from "../lib/paths.mjs";
import { parseStartArgs, parseStatusArgs, parseStopArgs, usage } from "./local/args.mjs";
import {
  processMatchesLauncher,
  serviceProcessMatches,
  startManagedProcess,
  waitFor,
  writeStartupLogTail,
} from "./local/managed.mjs";
import { prepareEnvFiles, readRuntimeProfile, runSeed, startInfrastructure } from "./local/setup.mjs";
import { removeManagedWebDistDir, startWebWithRetry } from "./local/web.mjs";

const statePath = path.join(runDir, "runtime-state.json");
const stopRequestPath = path.join(runDir, "runtime-stop-request.json");
const defaultRealtimeInternalToken = "hexrelay-dev-channel-dispatch-token-change-me";
const builtInRuntimeProfiles = ["single", "dual", "triple"];

function stateHasLiveProcesses(state) {
  if (!state?.instances) {
    return false;
  }
  for (const instance of state.instances) {
    for (const pid of [instance.apiPid, instance.realtimePid, instance.webPid]) {
      if (isProcessAlive(pid)) {
        return true;
      }
    }
  }
  return false;
}

async function startRuntimeInstance({ instance, baseApiEnv, baseRealtimeEnv, reservedPorts, startedProcesses }) {
  const instanceId = instance.id;
  const apiPort = await getFreePort(instance.apiPort, reservedPorts);
  const realtimePort = await getFreePort(instance.realtimePort, reservedPorts);
  const webPort = await getFreePort(instance.webPort, reservedPorts);
  if (apiPort !== instance.apiPort || realtimePort !== instance.realtimePort || webPort !== instance.webPort) {
    console.log(`[local-runtime] ${instanceId} requested ports were unavailable; using api=${apiPort} realtime=${realtimePort} web=${webPort}`);
  }

  const apiUrl = `http://127.0.0.1:${apiPort}`;
  const realtimeUrl = `http://127.0.0.1:${realtimePort}`;
  const realtimeWsUrl = `ws://127.0.0.1:${realtimePort}/ws`;
  const allowedOrigins = `http://localhost:${webPort},http://127.0.0.1:${webPort}`;
  const logDir = path.join(runDir, instanceId);
  const targetRoot = path.join(runDir, "targets", `${instanceId}-${process.pid}`);
  const apiTargetDir = path.join(targetRoot, "api");
  const realtimeTargetDir = path.join(targetRoot, "realtime");
  fs.mkdirSync(logDir, { recursive: true });

  const apiEnv = {
    ...baseApiEnv,
    CARGO_TARGET_DIR: apiTargetDir,
    API_BIND: `127.0.0.1:${apiPort}`,
    API_REALTIME_BASE_URL: realtimeUrl,
    API_ALLOWED_ORIGINS: allowedOrigins,
  };
  const realtimeEnv = {
    ...baseRealtimeEnv,
    CARGO_TARGET_DIR: realtimeTargetDir,
    REALTIME_BIND: `127.0.0.1:${realtimePort}`,
    REALTIME_API_BASE_URL: apiUrl,
    REALTIME_ALLOWED_ORIGINS: allowedOrigins,
    REALTIME_ENABLE_DEV_FAULTS: "true",
  };

  console.log(`[local-runtime] Starting ${instanceId} API service`);
  const apiProcess = startManagedProcess({
    name: "api-rs",
    cwd: root,
    env: apiEnv,
    command: isWindows ? "cargo.exe" : "cargo",
    args: ["run", "-p", "api-rs", "--bin", "api-rs"],
    logDir,
  });
  apiProcess.targetDir = apiTargetDir;
  startedProcesses.push(apiProcess);
  await waitFor(`${instanceId} api`, () => httpOk(`${apiUrl}/health`), {
    attempts: 300,
    failureProbe: () => !isProcessAlive(apiProcess.child.pid),
    onFailure: () => writeStartupLogTail(`${instanceId} API`, apiProcess.stdoutPath, apiProcess.stderrPath),
  });
  const apiPid = listenerPid(apiPort) ?? apiProcess.child.pid;
  apiProcess.servicePid = apiPid;

  console.log(`[local-runtime] Starting ${instanceId} realtime service`);
  const realtimeProcess = startManagedProcess({
    name: "realtime-rs",
    cwd: root,
    env: realtimeEnv,
    command: isWindows ? "cargo.exe" : "cargo",
    args: ["run", "-p", "realtime-rs"],
    logDir,
  });
  realtimeProcess.targetDir = realtimeTargetDir;
  startedProcesses.push(realtimeProcess);
  await waitFor(`${instanceId} realtime`, () => httpOk(`${realtimeUrl}/health`), {
    attempts: 300,
    failureProbe: () => !isProcessAlive(realtimeProcess.child.pid),
    onFailure: () => writeStartupLogTail(`${instanceId} realtime`, realtimeProcess.stdoutPath, realtimeProcess.stderrPath),
  });
  const realtimePid = listenerPid(realtimePort) ?? realtimeProcess.child.pid;
  realtimeProcess.servicePid = realtimePid;

  console.log(`[local-runtime] Starting ${instanceId} web dev server`);
  const webDistId = `${instanceId}-${process.pid}`.replace(/[^a-zA-Z0-9_-]/g, "-");
  const { processInfo: webProcess, webUrl } = await startWebWithRetry({
    instanceId,
    webPort,
    webEnv: {
      HEXRELAY_RUNTIME_INSTANCE: instanceId,
      HEXRELAY_RUNTIME_DIST_ID: webDistId,
      NEXT_PUBLIC_API_BASE_URL: apiUrl,
      NEXT_PUBLIC_REALTIME_WS_URL: realtimeWsUrl,
    },
    webDistId,
    logDir,
    startedProcesses,
  });
  const webPid = listenerPid(webPort) ?? webProcess.child.pid;
  webProcess.servicePid = webPid;

  return {
    id: instanceId,
    seedPersona: instance.seedPersona,
    apiPort,
    realtimePort,
    webPort,
    apiPid,
    realtimePid,
    webPid,
    apiLauncherPid: apiProcess.child.pid,
    realtimeLauncherPid: realtimeProcess.child.pid,
    webLauncherPid: webProcess.child.pid,
    apiLauncher: apiProcess.launcherPath,
    realtimeLauncher: realtimeProcess.launcherPath,
    webLauncher: webProcess.launcherPath,
    webDistDir: `.next-${webDistId}`,
    targetDirs: {
      api: apiTargetDir,
      realtime: realtimeTargetDir,
    },
    apiUrl,
    realtimeUrl,
    realtimeWsUrl,
    webUrl,
    logDir,
    realtimeInternalToken: realtimeEnv.REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN || defaultRealtimeInternalToken,
  };
}

function removeRuntimeTargetDir(targetDir) {
  if (!targetDir) {
    return;
  }
  const targetRoot = path.resolve(runDir, "targets");
  const resolved = path.resolve(targetDir);
  if (!resolved.startsWith(`${targetRoot}${path.sep}`)) {
    return;
  }
  try {
    fs.rmSync(resolved, { recursive: true, force: true });
  } catch {
  }
}

async function stopStartedProcesses(startedProcesses) {
  for (const processInfo of [...startedProcesses].reverse()) {
    if (processInfo?.servicePid && processInfo.servicePid !== processInfo.child?.pid) {
      await killProcessTree(processInfo.servicePid);
    }
    if (processInfo?.child?.pid) {
      await killProcessTree(processInfo.child.pid);
    }
    removeRuntimeTargetDir(processInfo?.targetDir);
    removeManagedWebDistDir(processInfo?.webDistDir);
  }
}

function runtimeProfilesForOrphanScan(primaryProfile = null) {
  const profiles = new Map();
  if (primaryProfile) {
    profiles.set(primaryProfile.profilePath || primaryProfile.name, primaryProfile);
  }
  for (const profileName of builtInRuntimeProfiles) {
    const profile = readRuntimeProfile(profileName);
    profiles.set(profile.profilePath || profile.name, profile);
  }
  return [...profiles.values()];
}

function listenerBelongsToLocalRuntime(service, pid) {
  const commandLine = processCommandLine(pid);
  if (commandLine.includes(root)) {
    return true;
  }
  if (service === "api" || service === "realtime") {
    return serviceProcessMatches(service, pid);
  }
  return false;
}

function cargoAncestorPids(pid) {
  const ancestors = [];
  let currentPid = pid;
  for (let depth = 0; depth < 4; depth += 1) {
    const parentPid = processParentPid(currentPid);
    if (!parentPid) {
      break;
    }
    const parentName = processName(parentPid).toLowerCase();
    if (parentName !== "cargo.exe" && parentName !== "cargo") {
      break;
    }
    ancestors.push(parentPid);
    currentPid = parentPid;
  }
  return ancestors;
}

async function stopUntrackedRuntimeListeners(profiles, reason, options = {}) {
  const stopped = [];
  const seenPids = new Set();
  const entries = [];
  const log = options.json ? console.error : console.log;

  for (const profile of profiles) {
    for (const instance of profile.instances ?? []) {
      entries.push({ instanceId: instance.id, service: "api", port: instance.apiPort });
      entries.push({ instanceId: instance.id, service: "realtime", port: instance.realtimePort });
      entries.push({ instanceId: instance.id, service: "web", port: instance.webPort });
    }
  }

  for (const entry of entries) {
    const pid = listenerPid(entry.port);
    if (!pid || seenPids.has(pid) || !listenerBelongsToLocalRuntime(entry.service, pid)) {
      continue;
    }
    seenPids.add(pid);
    log(
      `[local-runtime] Stopping untracked ${entry.service} listener on port ${entry.port} (pid=${pid}) before ${reason}.`,
    );
    const stoppedPids = [];
    for (const candidatePid of uniquePids([pid, ...cargoAncestorPids(pid)])) {
      if (await killProcessTree(candidatePid)) {
        stoppedPids.push(candidatePid);
      }
    }
    const stoppedEntry = { ...entry, pid, stopped: !isProcessAlive(pid), stoppedPids };
    if (!stoppedEntry.stopped) {
      const recovery = reason === "startup"
        ? "startup will avoid its port and use isolated build output"
        : "stop it from its owning shell or an elevated terminal if needed";
      log(`[local-runtime] Could not stop ${entry.service} listener pid=${pid}; ${recovery}.`);
    }
    stopped.push(stoppedEntry);
  }

  return stopped;
}

export async function startCommand(rawArgs) {
  const options = parseStartArgs(rawArgs);
  if (options.help) {
    console.log(usage("start"));
    return;
  }

  process.chdir(root);
  prepareEnvFiles();
  fs.mkdirSync(runDir, { recursive: true });
  const infraEnv = readEnvFile(path.join(root, "infra", ".env"));
  const apiEnv = readEnvFile(path.join(root, "services", "api-rs", ".env"));
  const realtimeEnv = readEnvFile(path.join(root, "services", "realtime-rs", ".env"));
  Object.assign(process.env, withCargoOnPath({ ...process.env, ...infraEnv, ...apiEnv }));
  const profile = readRuntimeProfile(options.runtimeProfile);
  const existingState = readJsonIfExists(statePath);
  if (existingState?.runtimeKind === "docker-test") {
    throw new Error("Docker runtime test stack is active. Use 'npm run runtime:docker -- down' before starting a host-process runtime.");
  }
  if (stateHasLiveProcesses(existingState)) {
    throw new Error("A tracked local runtime is already active. Run npm run status or npm run stop before starting another profile.");
  }
  if (existingState) {
    fs.rmSync(statePath, { force: true });
  }
  fs.rmSync(stopRequestPath, { force: true });
  await stopUntrackedRuntimeListeners(runtimeProfilesForOrphanScan(profile), "startup");

  await startInfrastructure(infraEnv);
  runSeed(options.seedProfile, { ...infraEnv, ...apiEnv });

  const startedProcesses = [];
  const state = {
    profile: profile.name,
    profilePath: profile.profilePath,
    seedProfile: options.seedProfile.trim() ? options.seedProfile : null,
    infraMode: profile.infraMode,
    supervisorPid: process.pid,
    startedAt: new Date().toISOString(),
    root,
    instances: [],
  };
  let cleanupStarted = false;
  const cleanup = async () => {
    if (cleanupStarted) {
      return;
    }
    cleanupStarted = true;
    await stopStartedProcesses(startedProcesses);
    for (const instance of state.instances) {
      removeManagedWebDistDir(instance.webDistDir);
      removeRuntimeTargetDir(instance.targetDirs?.api);
      removeRuntimeTargetDir(instance.targetDirs?.realtime);
    }
    fs.rmSync(statePath, { force: true });
    fs.rmSync(stopRequestPath, { force: true });
  };
  const signalHandler = (signal) => {
    cleanup().finally(() => process.exit(signal === "SIGINT" ? 130 : 143));
  };
  process.once("SIGINT", signalHandler);
  process.once("SIGTERM", signalHandler);

  try {
    const reservedPorts = new Set();
    for (const instance of profile.instances) {
      const instanceState = await startRuntimeInstance({
        instance,
        baseApiEnv: apiEnv,
        baseRealtimeEnv: realtimeEnv,
        reservedPorts,
        startedProcesses,
      });
      state.instances.push(instanceState);
      writeJson(statePath, state);
    }

    console.log("");
    console.log(`[local-runtime] Local runtime profile '${profile.name}' is ready`);
    for (const instance of state.instances) {
      console.log(`  [${instance.id}] API:      ${instance.apiUrl}`);
      console.log(`  [${instance.id}] Realtime: ${instance.realtimeUrl}`);
      console.log(`  [${instance.id}] WS:       ${instance.realtimeWsUrl}`);
      console.log(`  [${instance.id}] Web:      ${instance.webUrl}`);
      console.log(`  [${instance.id}] Logs:     ${instance.logDir}`);
    }
    console.log("");
    console.log("[local-runtime] Use npm run status from another shell to inspect health.");
    console.log("[local-runtime] Press Ctrl+C or run npm run stop to stop tracked processes.");

    const failureCounts = new Map();
    for (const instance of state.instances) {
      failureCounts.set(`${instance.id}:api`, 0);
      failureCounts.set(`${instance.id}:realtime`, 0);
      failureCounts.set(`${instance.id}:web`, 0);
    }

    while (true) {
      if (fs.existsSync(stopRequestPath)) {
        console.log("[local-runtime] Stop request received.");
        return;
      }
      for (const instance of state.instances) {
        const checks = [
          [`${instance.id}:api`, `${instance.id} API`, await httpOk(`${instance.apiUrl}/health`)],
          [`${instance.id}:realtime`, `${instance.id} realtime`, await httpOk(`${instance.realtimeUrl}/health`)],
          [`${instance.id}:web`, `${instance.id} web`, await webReady(instance.webUrl)],
        ];
        for (const [key, label, ok] of checks) {
          if (ok) {
            failureCounts.set(key, 0);
          } else {
            const nextCount = (failureCounts.get(key) ?? 0) + 1;
            failureCounts.set(key, nextCount);
            if (nextCount >= 15) {
              throw new Error(`${label} health check failed after startup`);
            }
          }
        }
      }
      await delay(2000);
    }
  } finally {
    process.off("SIGINT", signalHandler);
    process.off("SIGTERM", signalHandler);
    await cleanup();
  }
}

async function buildStatusResult() {
  const state = readJsonIfExists(statePath);
  if (!state) {
    return { active: false, instances: [] };
  }
  if (state.runtimeKind !== "docker-test" && !stateHasLiveProcesses(state)) {
    return {
      active: false,
      staleState: true,
      profile: state.profile,
      startedAt: state.startedAt,
      instances: [],
    };
  }
  const instances = [];
  for (const instance of state.instances ?? []) {
    instances.push({
      ...instance,
      apiProcessAlive: isProcessAlive(instance.apiPid),
      realtimeProcessAlive: isProcessAlive(instance.realtimePid),
      webProcessAlive: isProcessAlive(instance.webPid),
      apiHealthy: await httpOk(`${instance.apiUrl}/health`, 3000),
      realtimeHealthy: await httpOk(`${instance.realtimeUrl}/health`, 3000),
      webHealthy: await webReady(instance.webUrl),
    });
  }
  return { active: true, ...state, instances };
}

export async function statusCommand(rawArgs) {
  const options = parseStatusArgs(rawArgs);
  if (options.help) {
    console.log(usage("status"));
    return;
  }
  const result = await buildStatusResult();
  if (options.json) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  if (!result.active) {
    console.log("[local-runtime] No tracked local runtime is active.");
    return;
  }
  console.log(`[local-runtime] Runtime profile: ${result.profile}`);
  if (result.seedProfile) {
    console.log(`[local-runtime] Seed profile:    ${result.seedProfile}`);
  }
  console.log(`[local-runtime] Started at:      ${result.startedAt}`);
  for (const instance of result.instances) {
    console.log("");
    console.log(`[${instance.id}]`);
    console.log(`  API:      pid=${instance.apiPid} process=${instance.apiProcessAlive} health=${instance.apiHealthy} ${instance.apiUrl}`);
    console.log(`  Realtime: pid=${instance.realtimePid} process=${instance.realtimeProcessAlive} health=${instance.realtimeHealthy} ${instance.realtimeUrl}`);
    console.log(`  Web:      pid=${instance.webPid} process=${instance.webProcessAlive} health=${instance.webHealthy} ${instance.webUrl}`);
    console.log(`  WS:       ${instance.realtimeWsUrl}`);
    console.log(`  Logs:     ${instance.logDir}`);
  }
}

function runtimeProfileMatches(profileSpec, state) {
  if (!profileSpec.trim()) {
    return true;
  }
  if (state.profile === profileSpec || state.profilePath === profileSpec) {
    return true;
  }
  try {
    const resolved = readRuntimeProfile(profileSpec);
    return state.profile === resolved.name || state.profilePath === resolved.profilePath;
  } catch {
    return false;
  }
}

export async function stopCommand(rawArgs) {
  const options = parseStopArgs(rawArgs);
  if (options.help) {
    console.log(usage("stop"));
    return;
  }
  const state = readJsonIfExists(statePath);
  if (!state) {
    const profiles = options.runtimeProfile.trim()
      ? [readRuntimeProfile(options.runtimeProfile)]
      : runtimeProfilesForOrphanScan();
    const stopped = await stopUntrackedRuntimeListeners(profiles, "stop", { json: options.json });
    const stoppedCount = stopped.filter((entry) => entry.stopped).length;
    const message = stopped.length === 0
      ? "no tracked local runtime is active"
      : stoppedCount === stopped.length
        ? "stopped untracked local runtime listeners"
        : "found untracked local runtime listeners but could not stop all of them";
    const result = { stopped, message };
    fs.rmSync(stopRequestPath, { force: true });
    if (options.json) {
      console.log(JSON.stringify(result, null, 2));
    } else if (stopped.length > 0) {
      console.log(
        stoppedCount === stopped.length
          ? "[local-runtime] Stopped untracked local runtime listeners."
          : "[local-runtime] Found untracked local runtime listeners but could not stop all of them.",
      );
      for (const entry of stopped) {
        console.log(`  [${entry.instanceId}] ${entry.service} port=${entry.port} pid=${entry.pid} stopped=${entry.stopped}`);
      }
    } else {
      console.log("[local-runtime] No tracked local runtime is active.");
    }
    return;
  }
  if (state.runtimeKind === "docker-test") {
    const message = "Docker runtime test stack is active. Use 'npm run runtime:docker -- down' instead of 'npm run stop'.";
    if (options.json) {
      console.log(JSON.stringify({ stopped: [], runtimeKind: state.runtimeKind, message }, null, 2));
    } else {
      console.error(`[local-runtime] ERROR: ${message}`);
    }
    process.exitCode = 1;
    return;
  }
  if (!runtimeProfileMatches(options.runtimeProfile, state)) {
    throw new Error(`Active runtime profile is '${state.profile}', not '${options.runtimeProfile}'.`);
  }

  writeJson(stopRequestPath, { requestedAt: new Date().toISOString(), requestedByPid: process.pid });
  const stopped = [];
  for (const instance of state.instances ?? []) {
    for (const entry of [
      {
        service: "api",
        pid: instance.apiPid,
        launcherPid: instance.apiLauncherPid ?? instance.apiPid,
        launcher: instance.apiLauncher,
      },
      {
        service: "realtime",
        pid: instance.realtimePid,
        launcherPid: instance.realtimeLauncherPid ?? instance.realtimePid,
        launcher: instance.realtimeLauncher,
      },
      {
        service: "web",
        pid: instance.webPid,
        launcherPid: instance.webLauncherPid ?? instance.webPid,
        launcher: instance.webLauncher,
      },
    ]) {
      const { service, pid, launcherPid, launcher } = entry;
      let wasStopped = false;
      const stoppedPids = [];
      for (const candidatePid of uniquePids([launcherPid, pid])) {
        if (!isProcessAlive(candidatePid)) {
          continue;
        }
        const isLauncher = candidatePid === Number(launcherPid);
        const commandLine = processCommandLine(candidatePid);
        const belongsToWorkspace = commandLine.includes(root);
        const shouldStop = isLauncher
          ? processMatchesLauncher(candidatePid, launcher)
          : serviceProcessMatches(service, candidatePid) || belongsToWorkspace;
        if (!shouldStop) {
          continue;
        }
        if (await killProcessTree(candidatePid)) {
          wasStopped = true;
          stoppedPids.push(candidatePid);
        }
      }
      stopped.push({ instanceId: instance.id, service, pid: Number(pid), stopped: wasStopped, stoppedPids });
    }
  }

  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (stopped.every((entry) => !isProcessAlive(entry.pid))) {
      break;
    }
    await delay(500);
  }
  for (const entry of stopped) {
    if (!entry.stopped && !isProcessAlive(entry.pid)) {
      entry.stopped = true;
      entry.stoppedBySupervisor = true;
    }
  }

  fs.rmSync(statePath, { force: true });
  for (const instance of state.instances ?? []) {
    removeManagedWebDistDir(instance.webDistDir);
    removeRuntimeTargetDir(instance.targetDirs?.api);
    removeRuntimeTargetDir(instance.targetDirs?.realtime);
  }
  if (options.json) {
    console.log(JSON.stringify({ profile: state.profile, stopped }, null, 2));
    return;
  }
  console.log(`[local-runtime] Stopped tracked local runtime profile '${state.profile}'.`);
  for (const entry of stopped) {
    console.log(`  [${entry.instanceId}] ${entry.service} pid=${entry.pid} stopped=${entry.stopped}`);
  }
}

async function main() {
  const [command, ...args] = process.argv.slice(2);
  if (!command || command === "--help" || command === "-h") {
    console.log(usage());
    return;
  }
  if (command === "start") {
    await startCommand(args);
    return;
  }
  if (command === "status") {
    await statusCommand(args);
    return;
  }
  if (command === "stop") {
    await stopCommand(args);
    return;
  }
  throw new Error(`unknown local runtime command: ${command}\n${usage()}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    console.error(`[local-runtime] ERROR: ${error.message}`);
    process.exit(1);
  });
}
