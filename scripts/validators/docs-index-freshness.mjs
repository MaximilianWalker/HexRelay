import { changedFiles, diffText, resolveBaseSha } from "../lib/git.mjs";

const baseSha = resolveBaseSha(process.argv[2] ?? "");
const headSha = process.argv[3] ?? "HEAD";
const indexFile = "docs/README.md";

const docsChanges = changedFiles(baseSha, headSha, [
  "docs/**/*.md",
  "docs/**/*.yaml",
  "docs/**/*.yml",
  "docs/**/*.json",
]);

const canonicalChanges = docsChanges.filter(
  (file) =>
    file !== indexFile &&
    !/^docs\/operations\/quality-audits\/[0-9][0-9]-[^/]+\.md$/.test(file),
);

if (canonicalChanges.length === 0) {
  console.log("[docs-index-freshness] No canonical docs changes detected");
  process.exit(0);
}

if (!changedFiles(baseSha, headSha, [indexFile]).includes(indexFile)) {
  console.error(`::error::Canonical docs changed but ${indexFile} was not updated.`);
  console.error("Changed canonical docs:");
  console.error(canonicalChanges.join("\n"));
  process.exit(1);
}

const indexDiff = diffText(baseSha, headSha, [indexFile]);
if (!/^[-+]- (last_updated|Latest meaningful change):/m.test(indexDiff)) {
  console.error(`::error::Canonical docs changed but ${indexFile} metadata fields were not refreshed (last_updated or Latest meaningful change).`);
  process.exit(1);
}

console.log("[docs-index-freshness] docs index updated with canonical docs changes");
