import { runCapture, runCheckedCapture } from "./exec.mjs";

const NULL_SHA = "0000000000000000000000000000000000000000";

export function resolveBaseSha(baseSha = "") {
  if (baseSha && baseSha !== NULL_SHA) {
    return baseSha;
  }

  for (const args of [
    ["merge-base", "HEAD", "origin/main"],
    ["merge-base", "HEAD", "origin/master"],
    ["rev-parse", "HEAD~1"],
  ]) {
    const result = runCapture("git", args);
    if (result.status === 0 && result.stdout.trim()) {
      return result.stdout.trim();
    }
  }

  throw new Error("Unable to resolve base SHA");
}

export function changedFiles(baseSha, headSha = "HEAD", paths = []) {
  const args = ["diff", "--name-only", baseSha, headSha, "--", ...paths];
  return runCheckedCapture("git", args)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

export function diffText(baseSha, headSha = "HEAD", paths = []) {
  return runCheckedCapture("git", ["diff", baseSha, headSha, "--", ...paths]);
}
