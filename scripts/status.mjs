import { spawnSync } from "node:child_process";
import process from "node:process";

const isWindows = process.platform === "win32";

function normalizePowerShellArgs(args) {
  return args.map((arg) => {
    switch (arg) {
      case "--json":
        return "-Json";
      case "--help":
        return "-Help";
      default:
        return arg;
    }
  });
}

const command = isWindows ? "powershell.exe" : "bash";
const args = isWindows
  ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/status.ps1", ...normalizePowerShellArgs(process.argv.slice(2))]
  : ["scripts/status.sh", ...process.argv.slice(2)];

const result = spawnSync(command, args, { stdio: "inherit", shell: false });

if (result.error) {
  console.error(`[status] Failed to start ${command}: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
