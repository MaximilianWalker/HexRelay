import fs from "node:fs";
import path from "node:path";
import { root } from "./config.mjs";

export function resolveEvidenceDir(evidenceDir) {
  if (!evidenceDir) {
    return "";
  }
  return path.resolve(root, evidenceDir);
}

export function prepareEvidenceDir(evidenceDir) {
  if (!evidenceDir) {
    return;
  }
  fs.mkdirSync(evidenceDir, { recursive: true });
  for (const fileName of [
    "scenario-config.json",
    "runtime-status-before.json",
    "runtime-status-after.json",
    "event-log.ndjson",
    "verdict.md",
  ]) {
    fs.rmSync(path.join(evidenceDir, fileName), { force: true, recursive: true });
  }
}

export function writeEvidenceJson(evidenceDir, fileName, value) {
  if (!evidenceDir) {
    return;
  }
  fs.mkdirSync(evidenceDir, { recursive: true });
  fs.writeFileSync(path.join(evidenceDir, fileName), `${JSON.stringify(value, null, 2)}\n`);
}

export function appendEvidenceEvent(evidenceDir, event) {
  if (!evidenceDir) {
    return;
  }
  fs.mkdirSync(evidenceDir, { recursive: true });
  fs.appendFileSync(
    path.join(evidenceDir, "event-log.ndjson"),
    `${JSON.stringify({ at: new Date().toISOString(), ...event })}\n`,
  );
}

export function writeEvidenceVerdict(evidenceDir, status, error = null) {
  if (!evidenceDir) {
    return;
  }
  const lines = [
    `# Runtime Docker Smoke Verdict`,
    "",
    `- status: ${status}`,
    `- completed_at: ${new Date().toISOString()}`,
  ];
  if (error) {
    lines.push(`- error: ${error.message}`);
  }
  fs.writeFileSync(path.join(evidenceDir, "verdict.md"), `${lines.join("\n")}\n`);
}
