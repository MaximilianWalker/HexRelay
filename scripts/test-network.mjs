import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const runtimeDocker = path.join(scriptsDir, "runtime-docker.mjs");

const args = process.argv.slice(2);
const hasScope = args.some((arg) =>
  arg === "--scope" || arg === "-Scope" || arg.startsWith("--scope=") || arg.startsWith("-Scope="),
);
if (hasScope) {
  console.error("[test-network] This wrapper always runs --scope network; remove the explicit scope argument.");
  process.exit(1);
}

const result = spawnSync(process.execPath, [runtimeDocker, "smoke", "--scope", "network", ...args], {
  cwd: path.resolve(scriptsDir, ".."),
  stdio: "inherit",
  shell: false,
});

if (result.error) {
  console.error(`[test-network] Failed to start network smoke: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
