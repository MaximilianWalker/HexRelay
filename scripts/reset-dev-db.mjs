import process from "node:process";
import { exitFromError, runInherited } from "./lib/exec.mjs";
import { defaultRuntimeEnvFiles, ensureAndLoadEnvFiles, ensureCargoBinOnPath } from "./lib/env.mjs";

try {
  ensureCargoBinOnPath();
  ensureAndLoadEnvFiles(defaultRuntimeEnvFiles(), "reset-dev-db");
  runInherited("cargo", ["run", "-p", "api-rs", "--bin", "reset_dev_db", "--", ...process.argv.slice(2)]);
} catch (error) {
  exitFromError(error);
}
