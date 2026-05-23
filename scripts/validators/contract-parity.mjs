import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { rootDir } from "../lib/paths.mjs";

const candidates =
  process.platform === "win32"
    ? [
        ["py", ["-3"]],
        ["python", []],
      ]
    : [
        ["python3", []],
        ["python", []],
      ];

function findPython() {
  for (const [command, prefixArgs] of candidates) {
    const result = spawnSync(command, [...prefixArgs, "--version"], {
      cwd: rootDir,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      shell: false,
    });
    if (!result.error && result.status === 0) {
      return { command, prefixArgs };
    }
  }

  throw new Error("python3, python, or py -3 is required for contract parity validation.");
}

try {
  const python = findPython();
  const validatorPath = path.join(rootDir, "scripts", "validators", "contract_parity", "validator.py");
  const result = spawnSync(python.command, [...python.prefixArgs, validatorPath, ...process.argv.slice(2)], {
    cwd: process.cwd(),
    stdio: "inherit",
    shell: false,
  });
  if (result.error) {
    throw result.error;
  }
  process.exit(result.status ?? 1);
} catch (error) {
  console.error(`::error::${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}
