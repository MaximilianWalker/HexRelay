import { spawn } from "node:child_process";

const isWindows = process.platform === "win32";

function normalizePowerShellArgs(args) {
  return args.map((arg) => {
    switch (arg) {
      case "--runtime-profile":
        return "-RuntimeProfile";
      case "--seed-profile":
        return "-SeedProfile";
      case "--help":
        return "-Help";
      default:
        return arg;
    }
  });
}

const command = isWindows ? "powershell.exe" : "bash";
const args = isWindows
  ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/run.ps1", ...normalizePowerShellArgs(process.argv.slice(2))]
  : ["scripts/run.sh", ...process.argv.slice(2)];

const child = spawn(command, args, {
  stdio: "inherit",
  shell: false,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
