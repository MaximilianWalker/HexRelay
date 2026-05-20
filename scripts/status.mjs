import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const result = spawnSync(process.execPath, [path.join(scriptsDir, "runtime", "local.mjs"), "status", ...process.argv.slice(2)], {
  cwd: path.resolve(scriptsDir, ".."),
  stdio: "inherit",
  shell: false,
});

if (result.error) {
  console.error(`[status] Failed to start local runtime manager: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
