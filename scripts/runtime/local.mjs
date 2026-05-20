import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptsDir, "../..");
const runDir = path.join(root, ".local-run");
const statePath = path.join(runDir, "runtime-state.json");
const stopRequestPath = path.join(runDir, "runtime-stop-request.json");
const isWindows = process.platform === "win32";
const defaultRealtimeInternalToken = "hexrelay-dev-channel-dispatch-token-change-me";

function usage(command = "all") {
  const lines = {
    start: [
      "Usage: scripts/runtime/local.mjs start [--runtime-profile single|dual|triple|path] [--seed-profile dm-basic]",
      "Default startup uses the clean single profile and does not seed fixture data.",
    ],
    status: ["Usage: scripts/runtime/local.mjs status [--json]"],
    stop: ["Usage: scripts/runtime/local.mjs stop [--runtime-profile single|dual|triple|path] [--json]"],
  };
  if (command !== "all") {
    return lines[command].join("\n");
  }
  return [
    "Usage: scripts/runtime/local.mjs start|status|stop [options]",
    ...lines.start,
    ...lines.status,
    ...lines.stop,
  ].join("\n");
}

function normalizeFlag(arg) {
  switch (arg) {
    case "-RuntimeProfile":
      return "--runtime-profile";
    case "-SeedProfile":
      return "--seed-profile";
    case "-Json":
      return "--json";
    case "-Help":
      return "--help";
    default:
      return arg;
  }
}

function readValue(args, index, flag) {
  const value = args[index + 1];
  if (!value || value.startsWith("-")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function parseStartArgs(rawArgs) {
  const args = rawArgs.map(normalizeFlag);
  const options = { runtimeProfile: "single", seedProfile: "", help: false };
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--runtime-profile" || arg === "-r") {
      options.runtimeProfile = readValue(args, index, arg);
      index += 1;
    } else if (arg === "--seed-profile") {
      options.seedProfile = readValue(args, index, arg);
      index += 1;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown start option: ${arg}\n${usage("start")}`);
    }
  }
  return options;
}

function parseStatusArgs(rawArgs) {
  const args = rawArgs.map(normalizeFlag);
  const options = { json: false, help: false };
  for (const arg of args) {
    if (arg === "--json") {
      options.json = true;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown status option: ${arg}\n${usage("status")}`);
    }
  }
  return options;
}

function parseStopArgs(rawArgs) {
  const args = rawArgs.map(normalizeFlag);
  const options = { runtimeProfile: "", json: false, help: false };
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--runtime-profile" || arg === "-r") {
      options.runtimeProfile = readValue(args, index, arg);
      index += 1;
    } else if (arg === "--json") {
      options.json = true;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown stop option: ${arg}\n${usage("stop")}`);
    }
  }
  return options;
}

function ensureFileFromExample(filePath, examplePath) {
  if (!fs.existsSync(filePath)) {
    fs.copyFileSync(examplePath, filePath);
  }
}

function stripOuterQuotes(value) {
  if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1);
  }
  if (value.length >= 2 && value.startsWith("'") && value.endsWith("'")) {
    return value.slice(1, -1);
  }
  return value;
}

function readEnvFile(filePath) {
  const values = {};
  const raw = fs.readFileSync(filePath, "utf8");
  for (const line of raw.split(/\r?\n/)) {
    let trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) {
      continue;
    }
    if (trimmed.startsWith("export ")) {
      trimmed = trimmed.slice("export ".length).trim();
    }
    const separator = trimmed.indexOf("=");
    if (separator <= 0) {
      continue;
    }
    const key = trimmed.slice(0, separator).trim();
    const value = stripOuterQuotes(trimmed.slice(separator + 1).trim());
    if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) {
      values[key] = value;
    }
  }
  return values;
}

function withCargoOnPath(env) {
  if (!isWindows || !process.env.USERPROFILE) {
    return env;
  }
  const cargoBin = path.join(process.env.USERPROFILE, ".cargo", "bin");
  const currentPath = env.PATH ?? env.Path ?? process.env.PATH ?? "";
  const hasCargoBin = currentPath
    .split(path.delimiter)
    .some((entry) => entry.toLowerCase() === cargoBin.toLowerCase());
  if (hasCargoBin) {
    return env;
  }
  return { ...env, PATH: `${cargoBin}${path.delimiter}${currentPath}` };
}

function prepareEnvFiles() {
  ensureFileFromExample(path.join(root, "infra", ".env"), path.join(root, "infra", ".env.example"));
  ensureFileFromExample(
    path.join(root, "services", "api-rs", ".env"),
    path.join(root, "services", "api-rs", ".env.example"),
  );
  ensureFileFromExample(
    path.join(root, "services", "realtime-rs", ".env"),
    path.join(root, "services", "realtime-rs", ".env.example"),
  );
}

function runChecked(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
    encoding: "utf8",
    stdio: options.stdio ?? "pipe",
    shell: false,
  });
  if (result.error) {
    throw new Error(`failed to start ${command}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    const output = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(output || `${command} ${args.join(" ")} failed with exit code ${result.status}`);
  }
  return result;
}

function readRuntimeProfile(spec) {
  const result = runChecked(process.execPath, [path.join(root, "scripts", "validators", "runtime-profiles.mjs"), "--print", spec]);
  return JSON.parse(result.stdout);
}

function readJsonIfExists(filePath) {
  if (!fs.existsSync(filePath)) {
    return null;
  }
  return JSON.parse(fs.readFileSync(filePath, "utf8").replace(/^\uFEFF/, ""));
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function isProcessAlive(pid) {
  if (!Number.isInteger(Number(pid)) || Number(pid) <= 0) {
    return false;
  }
  try {
    process.kill(Number(pid), 0);
    return true;
  } catch (error) {
    return error?.code === "EPERM";
  }
}

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

function listenerPid(port) {
  if (isWindows) {
    const command = [
      `$c = Get-NetTCPConnection -LocalPort ${Number(port)} -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1;`,
      "if ($c) { $c.OwningProcess }",
    ].join(" ");
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    const pid = Number(result.stdout?.trim());
    return result.status === 0 && Number.isInteger(pid) && pid > 0 ? pid : null;
  }

  const lsof = spawnSync("lsof", ["-nP", `-iTCP:${Number(port)}`, "-sTCP:LISTEN", "-t"], {
    encoding: "utf8",
    shell: false,
  });
  if (lsof.status === 0) {
    const pid = Number(lsof.stdout.split(/\r?\n/).find(Boolean));
    if (Number.isInteger(pid) && pid > 0) {
      return pid;
    }
  }

  const ss = spawnSync("ss", ["-ltnp", `sport = :${Number(port)}`], {
    encoding: "utf8",
    shell: false,
  });
  if (ss.status === 0) {
    const match = ss.stdout.match(/pid=(\d+)/);
    const pid = match ? Number(match[1]) : null;
    if (Number.isInteger(pid) && pid > 0) {
      return pid;
    }
  }

  return null;
}

async function portInUseOnHost(port, host) {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", (error) => {
      if (error.code === "EADDRINUSE" || error.code === "EACCES") {
        resolve(true);
      } else if (error.code === "EADDRNOTAVAIL") {
        resolve(false);
      } else {
        reject(error);
      }
    });
    server.once("listening", () => {
      server.close(() => resolve(false));
    });
    server.listen(port, host);
  });
}

async function portInUse(port) {
  if (listenerPid(port)) {
    return true;
  }
  return (await portInUseOnHost(port, "127.0.0.1")) || (await portInUseOnHost(port, "::1"));
}

async function getFreePort(preferredPort, reservedPorts) {
  let port = preferredPort;
  while ((await portInUse(port)) || reservedPorts.has(port)) {
    if (port >= 65535) {
      throw new Error(`no available TCP port at or above ${preferredPort}`);
    }
    port += 1;
  }
  reservedPorts.add(port);
  return port;
}

async function httpOk(url, timeoutMs = 5000) {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(timeoutMs) });
    return response.status === 200;
  } catch {
    return false;
  }
}

async function webReady(url) {
  return (await httpOk(url)) || (await httpOk(`${url.replace(/\/$/, "")}/onboarding/identity`));
}

function logTail(filePath, lines = 40) {
  if (!fs.existsSync(filePath)) {
    return "";
  }
  const raw = fs.readFileSync(filePath, "utf8");
  return raw.split(/\r?\n/).slice(-lines).join("\n").trim();
}

function writeStartupLogTail(label, stdoutPath, stderrPath) {
  console.log(`[local-runtime] ${label} did not become ready. Recent logs:`);
  for (const [stream, filePath] of [
    ["stdout", stdoutPath],
    ["stderr", stderrPath],
  ]) {
    console.log(`[local-runtime] ${label} ${stream}: ${filePath}`);
    const tail = logTail(filePath);
    console.log(tail || "[local-runtime] (no log output)");
  }
}

async function waitFor(label, probe, options = {}) {
  const attempts = options.attempts ?? Number(process.env.WAIT_FOR_ATTEMPTS ?? 60);
  const sleepMs = options.sleepMs ?? Number(process.env.WAIT_FOR_SLEEP_MS ?? 1000);
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    if (await probe()) {
      console.log(`[local-runtime] ${label} is ready`);
      return;
    }
    if (options.failureProbe?.()) {
      options.onFailure?.();
      throw new Error(`${label} failed before becoming ready`);
    }
    await delay(sleepMs);
  }
  options.onFailure?.();
  throw new Error(`${label} did not become ready after ${attempts} attempts`);
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function quoteWindowsValue(value) {
  return String(value).replaceAll('"', '""');
}

function quoteShellValue(value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}

function commandLineFor(command, args) {
  if (isWindows) {
    return [command, ...args].join(" ");
  }
  return [command, ...args.map((arg) => quoteShellValue(arg))].join(" ");
}

function writeLauncher({ name, cwd, env, command, args, logDir }) {
  fs.mkdirSync(logDir, { recursive: true });
  const extension = isWindows ? "cmd" : "sh";
  const launcherPath = path.join(logDir, `${name}.${extension}`);
  const stdoutPath = path.join(logDir, `${name}.stdout.log`);
  const stderrPath = path.join(logDir, `${name}.stderr.log`);
  if (isWindows) {
    const lines = ["@echo off", `cd /d "${cwd}"`];
    for (const [key, value] of Object.entries(env)) {
      lines.push(`set "${key}=${quoteWindowsValue(value)}"`);
    }
    lines.push(commandLineFor(command, args));
    fs.writeFileSync(launcherPath, `${lines.join("\r\n")}\r\n`, "ascii");
  } else {
    const lines = ["#!/usr/bin/env bash", "set -euo pipefail", `cd ${quoteShellValue(cwd)}`];
    for (const [key, value] of Object.entries(env)) {
      lines.push(`export ${key}=${quoteShellValue(value)}`);
    }
    lines.push(commandLineFor(command, args));
    fs.writeFileSync(launcherPath, `${lines.join("\n")}\n`, "utf8");
    fs.chmodSync(launcherPath, 0o755);
  }
  return { launcherPath, stdoutPath, stderrPath };
}

function spawnLauncher({ launcherPath, stdoutPath, stderrPath, cwd, env }) {
  const stdout = fs.openSync(stdoutPath, "w");
  const stderr = fs.openSync(stderrPath, "w");
  const command = isWindows ? "cmd.exe" : "bash";
  const args = isWindows ? ["/d", "/s", "/c", launcherPath] : [launcherPath];
  const child = spawn(command, args, {
    cwd,
    env,
    detached: !isWindows,
    stdio: ["ignore", stdout, stderr],
    windowsHide: true,
    shell: false,
  });
  fs.closeSync(stdout);
  fs.closeSync(stderr);
  return child;
}

function startManagedProcess({ name, cwd, env, command, args, logDir }) {
  const launcher = writeLauncher({ name, cwd, env, command, args, logDir });
  const child = spawnLauncher({ ...launcher, cwd, env: withCargoOnPath({ ...process.env }) });
  return { child, ...launcher };
}

async function killProcessTree(pid) {
  if (!isProcessAlive(pid)) {
    return false;
  }
  if (isWindows) {
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], { stdio: "ignore", shell: false });
    await delay(500);
    return !isProcessAlive(pid);
  }
  try {
    process.kill(-Number(pid), "SIGTERM");
  } catch {
    try {
      process.kill(Number(pid), "SIGTERM");
    } catch {
    }
  }
  await delay(1000);
  if (isProcessAlive(pid)) {
    try {
      process.kill(-Number(pid), "SIGKILL");
    } catch {
      try {
        process.kill(Number(pid), "SIGKILL");
      } catch {
      }
    }
  }
  return true;
}

function processCommandLine(pid) {
  if (!isProcessAlive(pid)) {
    return "";
  }
  if (isWindows) {
    const command = `$p = Get-CimInstance Win32_Process -Filter "ProcessId = ${Number(pid)}" -ErrorAction SilentlyContinue; if ($p) { $p.CommandLine }`;
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    return result.status === 0 ? result.stdout.trim() : "";
  }
  const result = spawnSync("ps", ["-p", String(pid), "-o", "args="], {
    encoding: "utf8",
    shell: false,
  });
  return result.status === 0 ? result.stdout.trim() : "";
}

function processName(pid) {
  if (!isProcessAlive(pid)) {
    return "";
  }
  if (isWindows) {
    const command = `$p = Get-CimInstance Win32_Process -Filter "ProcessId = ${Number(pid)}" -ErrorAction SilentlyContinue; if ($p) { $p.Name }`;
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    return result.status === 0 ? result.stdout.trim() : "";
  }
  const result = spawnSync("ps", ["-p", String(pid), "-o", "comm="], {
    encoding: "utf8",
    shell: false,
  });
  return result.status === 0 ? result.stdout.trim() : "";
}

function processMatchesLauncher(pid, launcherPath) {
  if (!launcherPath) {
    return true;
  }
  const commandLine = processCommandLine(pid);
  if (!commandLine) {
    return false;
  }
  return commandLine.includes(launcherPath) || commandLine.includes(path.basename(launcherPath));
}

function serviceProcessMatches(service, pid) {
  const name = processName(pid).toLowerCase();
  if (service === "api") {
    return name === "api-rs.exe" || name === "api-rs";
  }
  if (service === "realtime") {
    return name === "realtime-rs.exe" || name === "realtime-rs";
  }
  if (service === "web") {
    return name === "node.exe" || name === "node";
  }
  return false;
}

function uniquePids(values) {
  return [...new Set(values.map(Number).filter((pid) => Number.isInteger(pid) && pid > 0))];
}

function dockerCompose(args, options = {}) {
  return runChecked("docker", ["compose", "--env-file", "infra/.env", "-f", "infra/docker-compose.yml", ...args], {
    stdio: options.stdio ?? "pipe",
  });
}

async function startInfrastructure(infraEnv) {
  console.log("[local-runtime] Starting local infrastructure");
  dockerCompose(["up", "-d", "postgres", "redis", "minio"]);
  const postgresUser = infraEnv.POSTGRES_USER || "hexrelay";
  const postgresDb = infraEnv.POSTGRES_DB || "hexrelay";
  await waitFor("postgres", () => {
    const result = spawnSync(
      "docker",
      [
        "compose",
        "--env-file",
        "infra/.env",
        "-f",
        "infra/docker-compose.yml",
        "exec",
        "-T",
        "postgres",
        "pg_isready",
        "-U",
        postgresUser,
        "-d",
        postgresDb,
      ],
      { cwd: root, stdio: "ignore", shell: false },
    );
    return result.status === 0;
  });
  await waitFor("redis", () => {
    const result = spawnSync(
      "docker",
      ["compose", "--env-file", "infra/.env", "-f", "infra/docker-compose.yml", "exec", "-T", "redis", "redis-cli", "--raw", "ping"],
      { cwd: root, encoding: "utf8", shell: false },
    );
    return result.status === 0 && result.stdout.includes("PONG");
  });
  await waitFor("minio", () => httpOk("http://localhost:9000/minio/health/live"));
}

function runSeed(seedProfile, env) {
  if (!seedProfile.trim()) {
    return;
  }
  console.log(`[local-runtime] Seeding local database with '${seedProfile}'`);
  fs.mkdirSync(runDir, { recursive: true });
  const stdoutPath = path.join(runDir, "seed.stdout.json");
  const stderrPath = path.join(runDir, "seed.stderr.log");
  const stdout = fs.openSync(stdoutPath, "w");
  const stderr = fs.openSync(stderrPath, "w");
  const result = spawnSync(
    isWindows ? "cargo.exe" : "cargo",
    ["run", "-p", "api-rs", "--bin", "seed_dev", "--", "--profile", seedProfile, "--json"],
    {
      cwd: root,
      env: withCargoOnPath({ ...process.env, ...env }),
      stdio: ["ignore", stdout, stderr],
      shell: false,
    },
  );
  fs.closeSync(stdout);
  fs.closeSync(stderr);
  if (result.error) {
    throw new Error(`failed to start seed process: ${result.error.message}`);
  }
  if (result.status !== 0) {
    const tail = logTail(stderrPath);
    throw new Error(`seed profile '${seedProfile}' failed${tail ? `\n${tail}` : ""}`);
  }
  console.log(`[local-runtime] Seed output written to ${stdoutPath}`);
}

function writeRuntimeTsConfig(distId) {
  const runtimeTsConfigDir = path.join(root, "apps", "web", ".runtime-tsconfig");
  fs.mkdirSync(runtimeTsConfigDir, { recursive: true });
  writeJson(path.join(runtimeTsConfigDir, `${distId}.json`), {
    extends: "../tsconfig.json",
    include: [
      "../next-env.d.ts",
      "../**/*.ts",
      "../**/*.tsx",
      `../.next-${distId}/types/**/*.ts`,
      `../.next-${distId}/dev/types/**/*.ts`,
      "../**/*.mts",
    ],
    exclude: ["../node_modules"],
  });
}

function removeManagedWebDistDir(distDir) {
  if (!distDir) {
    return;
  }
  const webDir = path.join(root, "apps", "web");
  const resolved = path.resolve(webDir, distDir);
  if (path.dirname(resolved) !== webDir || !path.basename(resolved).startsWith(".next-")) {
    return;
  }
  fs.rmSync(resolved, { recursive: true, force: true });
}

function findExistingNextPid(stderrPath) {
  const tail = logTail(stderrPath, 80);
  const match = tail.match(/PID:\s+(\d+)/);
  return match ? Number(match[1]) : null;
}

async function startWebWithRetry({ instanceId, webPort, webEnv, webDistId, logDir, startedProcesses }) {
  const webDir = path.join(root, "apps", "web");
  const webBaseUrl = `http://localhost:${webPort}`;
  let webUrl = webBaseUrl;
  let processInfo = null;
  writeRuntimeTsConfig(webDistId);

  for (let attempt = 1; attempt <= 2; attempt += 1) {
    processInfo = startManagedProcess({
      name: "web",
      cwd: webDir,
      env: webEnv,
      command: isWindows ? ".\\node_modules\\.bin\\next.cmd" : "./node_modules/.bin/next",
      args: ["dev", "--port", String(webPort)],
      logDir,
    });
    processInfo.webDistDir = `.next-${webDistId}`;
    startedProcesses.push(processInfo);
    await waitFor(
      `${instanceId} web`,
      async () => {
        if (logTail(processInfo.stdoutPath, 20).includes("Ready in")) {
          return true;
        }
        if (logTail(processInfo.stderrPath, 40).includes("Another next dev server is already running")) {
          return true;
        }
        return httpOk(webBaseUrl);
      },
      {
        failureProbe: () => {
          if (logTail(processInfo.stderrPath, 40).includes("Another next dev server is already running")) {
            return false;
          }
          return !isProcessAlive(processInfo.child.pid);
        },
        onFailure: () => writeStartupLogTail(`${instanceId} web`, processInfo.stdoutPath, processInfo.stderrPath),
      },
    );

    const stderrTail = logTail(processInfo.stderrPath, 80);
    if (stderrTail.includes("Another next dev server is already running")) {
      const existingPid = findExistingNextPid(processInfo.stderrPath);
      if (existingPid && attempt < 2) {
        console.log(`[local-runtime] Stopping stale Next dev server PID ${existingPid} and retrying ${instanceId} web startup`);
        await killProcessTree(existingPid);
        await delay(2000);
        continue;
      }
      throw new Error(
        existingPid
          ? `Another Next dev server PID ${existingPid} is still running. Stop it and rerun npm run start.`
          : "Another Next dev server is already running, but its PID could not be determined. Stop it and rerun npm run start.",
      );
    }
    break;
  }

  await waitFor(`${instanceId} web HTTP`, () => webReady(webUrl), {
    attempts: 60,
    failureProbe: () => !isProcessAlive(processInfo.child.pid),
    onFailure: () => writeStartupLogTail(`${instanceId} web HTTP`, processInfo.stdoutPath, processInfo.stderrPath),
  });

  return { processInfo, webUrl };
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
  fs.mkdirSync(logDir, { recursive: true });

  const apiEnv = {
    ...baseApiEnv,
    API_BIND: `127.0.0.1:${apiPort}`,
    API_REALTIME_BASE_URL: realtimeUrl,
    API_ALLOWED_ORIGINS: allowedOrigins,
  };
  const realtimeEnv = {
    ...baseRealtimeEnv,
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
  startedProcesses.push(apiProcess);
  await waitFor(`${instanceId} api`, () => httpOk(`${apiUrl}/health`), {
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
  startedProcesses.push(realtimeProcess);
  await waitFor(`${instanceId} realtime`, () => httpOk(`${realtimeUrl}/health`), {
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
    apiUrl,
    realtimeUrl,
    realtimeWsUrl,
    webUrl,
    logDir,
    realtimeInternalToken: realtimeEnv.REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN || defaultRealtimeInternalToken,
  };
}

async function stopStartedProcesses(startedProcesses) {
  for (const processInfo of [...startedProcesses].reverse()) {
    if (processInfo?.servicePid && processInfo.servicePid !== processInfo.child?.pid) {
      await killProcessTree(processInfo.servicePid);
    }
    if (processInfo?.child?.pid) {
      await killProcessTree(processInfo.child.pid);
    }
    removeManagedWebDistDir(processInfo?.webDistDir);
  }
}

async function startCommand(rawArgs) {
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

async function statusCommand(rawArgs) {
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

async function stopCommand(rawArgs) {
  const options = parseStopArgs(rawArgs);
  if (options.help) {
    console.log(usage("stop"));
    return;
  }
  const state = readJsonIfExists(statePath);
  if (!state) {
    const result = { stopped: [], message: "no tracked local runtime is active" };
    fs.rmSync(stopRequestPath, { force: true });
    if (options.json) {
      console.log(JSON.stringify(result, null, 2));
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
  fs.rmSync(statePath, { force: true });
  for (const instance of state.instances ?? []) {
    removeManagedWebDistDir(instance.webDistDir);
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

main().catch((error) => {
  console.error(`[local-runtime] ERROR: ${error.message}`);
  process.exit(1);
});
