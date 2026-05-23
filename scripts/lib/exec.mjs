import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { rootDir } from "./paths.mjs";

function npmCliCandidates() {
  const nodeDir = path.dirname(process.execPath);
  return [
    process.env.npm_execpath,
    path.join(nodeDir, "node_modules", "npm", "bin", "npm-cli.js"),
    path.resolve(nodeDir, "..", "lib", "node_modules", "npm", "bin", "npm-cli.js"),
  ].filter(Boolean);
}

function npmInvocation(args) {
  const npmCli = npmCliCandidates().find((candidate) => fs.existsSync(candidate));
  if (!npmCli) {
    return {
      command: process.platform === "win32" ? "npm.cmd" : "npm",
      args,
    };
  }

  return {
    command: process.execPath,
    args: [npmCli, ...args],
  };
}

export function nativeCommand(command) {
  if (process.platform !== "win32") {
    return command;
  }

  if (command === "npm") {
    return "npm.cmd";
  }

  if (["cargo", "cargo-audit", "docker", "git"].includes(command)) {
    return `${command}.exe`;
  }

  return command;
}

function nativeInvocation(command, args) {
  if (command === "npm") {
    return npmInvocation(args);
  }

  return {
    command: nativeCommand(command),
    args,
  };
}

export function runInherited(command, args = [], options = {}) {
  const invocation = nativeInvocation(command, args);
  const result = spawnSync(invocation.command, invocation.args, {
    cwd: options.cwd ?? rootDir,
    env: options.env ?? process.env,
    stdio: "inherit",
    shell: false,
  });

  if (result.error) {
    if (options.allowStartError) {
      return {
        status: 1,
        stdout: "",
        stderr: result.error.message,
      };
    }
    throw new Error(`Failed to start ${command}: ${result.error.message}`);
  }

  if ((result.status ?? 1) !== 0) {
    throw new Error(`${command} ${args.join(" ")} exited with code ${result.status ?? 1}`);
  }
}

export function runChecked(command, args = [], options = {}) {
  const invocation = nativeInvocation(command, args);
  const result = spawnSync(invocation.command, invocation.args, {
    cwd: options.cwd ?? rootDir,
    env: options.env ?? process.env,
    encoding: "utf8",
    stdio: options.stdio ?? "pipe",
    shell: false,
  });

  if (result.error) {
    throw new Error(`Failed to start ${command}: ${result.error.message}`);
  }

  if ((result.status ?? 1) !== 0) {
    const output = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(output || `${command} ${args.join(" ")} failed with exit code ${result.status ?? 1}`);
  }

  return result;
}

export function runCapture(command, args = [], options = {}) {
  const invocation = nativeInvocation(command, args);
  const result = spawnSync(invocation.command, invocation.args, {
    cwd: options.cwd ?? rootDir,
    env: options.env ?? process.env,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    shell: false,
  });

  if (result.error) {
    if (options.allowStartError) {
      return {
        status: 1,
        stdout: "",
        stderr: result.error.message,
      };
    }
    throw new Error(`Failed to start ${command}: ${result.error.message}`);
  }

  return {
    status: result.status ?? 1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

export function runCheckedCapture(command, args = [], options = {}) {
  const result = runCapture(command, args, options);
  if (result.status !== 0) {
    const detail = `${result.stderr}${result.stdout}`.trim();
    throw new Error(`${command} ${args.join(" ")} exited with code ${result.status}${detail ? `\n${detail}` : ""}`);
  }
  return result.stdout;
}

export function exitFromError(error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
