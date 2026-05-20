import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { runCapture } from "../lib/exec.mjs";
import { rootDir } from "../lib/paths.mjs";

const runId = process.env.GITHUB_RUN_ID || `local-${new Date().toISOString().replace(/[-:]/g, "").slice(0, 15)}`;
const evidenceDir = path.join(rootDir, "evidence", "ci", runId);
fs.mkdirSync(evidenceDir, { recursive: true });

const copies = [
  ["api.log", "api.log"],
  ["realtime.log", "realtime.log"],
  ["smoke-e2e.log", "smoke-e2e.log"],
  ["health-checks.log", "health-checks.log"],
  ["apps/web/coverage/coverage-summary.json", "web-coverage-summary.json"],
];

for (const [source, target] of copies) {
  const sourcePath = path.join(rootDir, source);
  if (fs.existsSync(sourcePath)) {
    fs.copyFileSync(sourcePath, path.join(evidenceDir, target));
  }
}

const gitResult = runCapture("git", ["rev-parse", "HEAD"]);
const gitSha = gitResult.status === 0 ? gitResult.stdout.trim() : "unknown";
const collectedAt = new Date().toISOString();

function presence(file) {
  return fs.existsSync(path.join(evidenceDir, file)) ? "present" : "missing";
}

fs.writeFileSync(
  path.join(evidenceDir, "manifest.txt"),
  [
    `run_id=${runId}`,
    `collected_at=${collectedAt}`,
    `api_log=${presence("api.log")}`,
    `realtime_log=${presence("realtime.log")}`,
    `smoke_log=${presence("smoke-e2e.log")}`,
    `health_checks=${presence("health-checks.log")}`,
    `web_coverage_summary=${presence("web-coverage-summary.json")}`,
    "",
  ].join("\n"),
);

const artifacts = [];
for (const file of ["api.log", "realtime.log", "smoke-e2e.log", "health-checks.log", "web-coverage-summary.json"]) {
  const filePath = path.join(evidenceDir, file);
  if (!fs.existsSync(filePath)) {
    continue;
  }
  const content = fs.readFileSync(filePath);
  artifacts.push({
    file,
    sha256: crypto.createHash("sha256").update(content).digest("hex"),
    bytes: content.byteLength,
  });
}

fs.writeFileSync(
  path.join(evidenceDir, "summary.json"),
  `${JSON.stringify({ run_id: runId, git_sha: gitSha, collected_at: collectedAt, artifacts }, null, 2)}\n`,
);

console.log(`Collected CI evidence in evidence/ci/${runId}`);
