import { exitFromError, runInherited } from "./lib/exec.mjs";
import { ensureCargoBinOnPath, ensureFileFromExample } from "./lib/env.mjs";
import { ensureCargoAudit } from "./security/cargo-audit.mjs";

try {
  ensureCargoBinOnPath();
  ensureFileFromExample("infra/.env", "infra/.env.example", "setup");

  console.log("[setup] Installing web dependencies");
  runInherited("npm", ["ci", "--prefix", "apps/web"]);

  console.log("[setup] Fetching Rust dependencies");
  runInherited("cargo", ["fetch", "--manifest-path", "services/api-rs/Cargo.toml"]);
  runInherited("cargo", ["fetch", "--manifest-path", "services/realtime-rs/Cargo.toml"]);

  console.log("[setup] Installing pinned security tooling");
  ensureCargoAudit();

  console.log("[setup] Complete");
} catch (error) {
  exitFromError(error);
}
