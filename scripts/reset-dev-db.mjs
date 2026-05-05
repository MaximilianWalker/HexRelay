import { spawnSync } from "node:child_process";
import process from "node:process";

const isWindows = process.platform === "win32";
const command = isWindows ? "powershell" : "bash";
const args = isWindows
  ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/reset-dev-db.ps1", ...process.argv.slice(2)]
  : ["scripts/reset-dev-db.sh", ...process.argv.slice(2)];

const result = spawnSync(command, args, { stdio: "inherit", shell: false });

if (result.error) {
  console.error(`[reset-dev-db] Failed to start ${command}: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
