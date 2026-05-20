import { spawnSync } from "node:child_process";
import process from "node:process";
import { rootDir } from "./paths.mjs";

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

export function runInherited(command, args = [], options = {}) {
  const result = spawnSync(nativeCommand(command), args, {
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
  const result = spawnSync(nativeCommand(command), args, {
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
  const result = spawnSync(nativeCommand(command), args, {
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
