import path from "node:path";
import { changedFiles, resolveBaseSha } from "../lib/git.mjs";

const baseSha = resolveBaseSha(process.argv[2] ?? "");
const headSha = process.argv[3] ?? "HEAD";

const migrationFiles = changedFiles(baseSha, headSha, ["services/api-rs/migrations/*.sql"]);
if (migrationFiles.length === 0) {
  console.log("[migration-evidence] No migration SQL changes detected");
  process.exit(0);
}

let missing = false;
for (const migrationFile of migrationFiles) {
  const migrationName = path.basename(migrationFile, ".sql");
  const evidenceFile = `evidence/migrations/${migrationName}.md`;
  const evidenceChanges = changedFiles(baseSha, headSha, [evidenceFile]);
  if (!evidenceChanges.includes(evidenceFile)) {
    console.error(`::error::Migration ${migrationFile} changed but missing evidence artifact update at ${evidenceFile}.`);
    missing = true;
  }
}

if (missing) {
  console.error("[migration-evidence] Copy docs/operations/migration-validation-template.md into evidence/migrations/<migration>.md and fill it in.");
  process.exit(1);
}

console.log("[migration-evidence] Migration evidence artifacts are present for all changed migrations");
