import { cargoAuditAdvisories } from "../security/advisories.mjs";

const today = new Date().toISOString().slice(0, 10);
let failed = false;

for (const advisory of cargoAuditAdvisories) {
  if (today > advisory.expires) {
    console.error(`::error::cargo-audit ignore ${advisory.id} expired on ${advisory.expires}.`);
    console.error("Remove the ignore or renew with explicit rationale in the same PR.");
    failed = true;
  } else {
    console.log(`[security] cargo-audit ignore ${advisory.id} valid until ${advisory.expires}`);
  }
}

process.exit(failed ? 1 : 0);
