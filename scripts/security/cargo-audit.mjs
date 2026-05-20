import { runCapture, runInherited } from "../lib/exec.mjs";
import { ensureCargoBinOnPath } from "../lib/env.mjs";
import { CARGO_AUDIT_VERSION, cargoAuditIgnoreArgs } from "./advisories.mjs";

export function installedCargoAuditVersion() {
  const result = runCapture("cargo-audit", ["--version"], { allowStartError: true });
  if (result.status !== 0) {
    return "";
  }

  const match = result.stdout.match(/([0-9]+\.[0-9]+\.[0-9]+)/);
  return match?.[1] ?? "";
}

export function ensureCargoAudit() {
  ensureCargoBinOnPath();
  const installedVersion = installedCargoAuditVersion();
  if (installedVersion !== CARGO_AUDIT_VERSION) {
    console.log(`[security] Installing cargo-audit ${CARGO_AUDIT_VERSION}`);
    runInherited("cargo", ["install", "cargo-audit", "--version", CARGO_AUDIT_VERSION, "--locked"]);
  }

  const finalVersion = installedCargoAuditVersion();
  if (finalVersion !== CARGO_AUDIT_VERSION) {
    throw new Error(`Expected cargo-audit ${CARGO_AUDIT_VERSION} but found ${finalVersion || "none"}`);
  }

  console.log(`[security] Using cargo-audit ${finalVersion}`);
}

export function runCargoAudit() {
  ensureCargoAudit();
  runInherited("cargo", ["audit", "--deny", "warnings", ...cargoAuditIgnoreArgs()]);
}
