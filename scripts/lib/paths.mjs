import path from "node:path";
import { fileURLToPath } from "node:url";

export const scriptsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const rootDir = path.resolve(scriptsDir, "..");
export const fixturesDir = path.join(rootDir, "fixtures");
export const runDir = path.join(rootDir, ".local-run");

export function fromRoot(...parts) {
  return path.join(rootDir, ...parts);
}

export function relativeFromRoot(filePath) {
  return path.relative(rootDir, filePath).replaceAll(path.sep, "/");
}
