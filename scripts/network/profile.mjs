import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { stripBom } from "../lib/json.mjs";
import { rootDir as root } from "../lib/paths.mjs";

export function readNetworkProfile(profileSpec) {
  const validatorPath = path.join(root, "scripts", "validators", "network-profiles.mjs");
  const result = spawnSync(process.execPath, [validatorPath, "--print", profileSpec], {
    cwd: root,
    encoding: "utf8",
    shell: false,
  });
  if (result.status !== 0) {
    throw new Error((result.stderr || result.stdout).trim() || `failed to read network profile '${profileSpec}'`);
  }
  return JSON.parse(stripBom(result.stdout));
}
