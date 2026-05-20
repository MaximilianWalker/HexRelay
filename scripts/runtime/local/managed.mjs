import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { logTail } from "../../lib/http.mjs";
import { delay, isWindows, processCommandLine, processName, withCargoOnPath } from "../../lib/process.mjs";

export function writeStartupLogTail(label, stdoutPath, stderrPath) {
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

export async function waitFor(label, probe, options = {}) {
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

export function startManagedProcess({ name, cwd, env, command, args, logDir }) {
  const launcher = writeLauncher({ name, cwd, env, command, args, logDir });
  const child = spawnLauncher({ ...launcher, cwd, env: withCargoOnPath({ ...process.env }) });
  return { child, ...launcher };
}

export function processMatchesLauncher(pid, launcherPath) {
  if (!launcherPath) {
    return true;
  }
  const commandLine = processCommandLine(pid);
  if (!commandLine) {
    return false;
  }
  return commandLine.includes(launcherPath) || commandLine.includes(path.basename(launcherPath));
}

export function serviceProcessMatches(service, pid) {
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
