import process from "node:process";
import { exitFromError, runInherited } from "./lib/exec.mjs";
import { defaultRuntimeEnvFiles, ensureAndLoadEnvFiles, ensureCargoBinOnPath } from "./lib/env.mjs";

try {
  ensureCargoBinOnPath();
  ensureAndLoadEnvFiles(defaultRuntimeEnvFiles(), "seed");
  runInherited("cargo", ["run", "-p", "api-rs", "--bin", "seed_dev", "--", ...process.argv.slice(2)]);
} catch (error) {
  exitFromError(error);
}
