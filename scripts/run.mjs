import { spawn } from "node:child_process";

const isWindows = process.platform === "win32";

const command = isWindows ? "powershell.exe" : "bash";
const args = isWindows
  ? ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "scripts/run.ps1"]
  : ["scripts/run.sh"];

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
