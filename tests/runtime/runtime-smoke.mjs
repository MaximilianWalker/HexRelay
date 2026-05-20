import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptsDir, "../..");
const runtimeDocker = path.join(root, "scripts", "runtime", "docker.mjs");

const args = process.argv.slice(2);
const result = spawnSync(process.execPath, [runtimeDocker, "smoke", ...args], {
  cwd: root,
  stdio: "inherit",
  shell: false,
});

if (result.error) {
  console.error(`[test-runtime] Failed to start runtime smoke: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
