import { spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const child = spawn(process.execPath, [path.join(scriptsDir, "runtime", "local.mjs"), "start", ...process.argv.slice(2)], {
  cwd: path.resolve(scriptsDir, ".."),
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
