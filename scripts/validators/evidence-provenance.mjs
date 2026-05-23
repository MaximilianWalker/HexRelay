import fs from "node:fs";
import path from "node:path";
import { rootDir } from "../lib/paths.mjs";
import { changedFiles, resolveBaseSha } from "../lib/git.mjs";

const baseSha = resolveBaseSha(process.argv[2] ?? "");
const headSha = process.argv[3] ?? "HEAD";
const changedEvidenceFiles = changedFiles(baseSha, headSha, ["evidence/iteration-*/**", "evidence/operations/**"]);

if (changedEvidenceFiles.length === 0) {
  console.log("[evidence-provenance] No iteration/operations evidence changes detected");
  process.exit(0);
}

function artifactDir(filePath) {
  const normalized = filePath.replaceAll("\\", "/");
  const outputsIndex = normalized.indexOf("/outputs/");
  if (outputsIndex !== -1) {
    return normalized.slice(0, outputsIndex);
  }
  return path.posix.dirname(normalized);
}

let missing = false;
const checkedDirs = new Set();

for (const changedFile of changedEvidenceFiles) {
  const dir = artifactDir(changedFile);
  if (!dir || checkedDirs.has(dir)) {
    continue;
  }
  checkedDirs.add(dir);

  const provenanceFile = path.join(rootDir, dir, "provenance.json");
  if (!fs.existsSync(provenanceFile)) {
    console.error(`::error::Missing required provenance file at ${dir}/provenance.json.`);
    missing = true;
    continue;
  }

  let provenance;
  try {
    provenance = JSON.parse(fs.readFileSync(provenanceFile, "utf8"));
  } catch (error) {
    console.error(`::error::${dir}/provenance.json is not valid JSON: ${error.message}`);
    missing = true;
    continue;
  }

  if (!provenance.commit_sha) {
    console.error(`::error::${dir}/provenance.json missing required field commit_sha.`);
    missing = true;
  }
  if (!provenance.generated_at_utc) {
    console.error(`::error::${dir}/provenance.json missing required field generated_at_utc.`);
    missing = true;
  }
  if (!provenance.pr_number && !provenance.run_id) {
    console.error(`::error::${dir}/provenance.json must include pr_number or run_id.`);
    missing = true;
  }
}

if (missing) {
  console.error("[evidence-provenance] Add provenance.json with commit_sha, generated_at_utc, and pr_number/run_id to each changed evidence artifact directory.");
  process.exit(1);
}

console.log("[evidence-provenance] Provenance files validated for changed evidence artifacts");
