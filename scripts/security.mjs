import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";

const isWindows = process.platform === "win32";

function prependCargoBinOnWindows() {
  if (!isWindows || !process.env.USERPROFILE) {
    return;
  }

  const pathKey =
    Object.keys(process.env).find((key) => key.toLowerCase() === "path") ?? "PATH";
  const cargoBin = path.join(process.env.USERPROFILE, ".cargo", "bin");
  const entries = (process.env[pathKey] ?? "").split(path.delimiter);

  if (!entries.some((entry) => entry.toLowerCase() === cargoBin.toLowerCase())) {
    process.env[pathKey] = [cargoBin, ...entries].filter(Boolean).join(path.delimiter);
  }
}

function run(label, command, args) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: false,
    env: process.env,
  });

  if (result.error) {
    console.error(`[security] Failed to start ${label}: ${result.error.message}`);
    process.exit(1);
  }

  if (result.signal) {
    process.kill(process.pid, result.signal);
    return;
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

prependCargoBinOnWindows();

const ensureCommand = isWindows ? "powershell.exe" : "bash";
const ensureArgs = isWindows
  ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/ensure-cargo-audit.ps1"]
  : ["scripts/ensure-cargo-audit.sh"];

run("cargo-audit installer", ensureCommand, ensureArgs);
run("cargo audit", "cargo", [
  "audit",
  "--deny",
  "warnings",
  "--ignore",
  "RUSTSEC-2023-0071",
]);
