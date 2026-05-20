import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { ensureFileFromExample } from "../../lib/env.mjs";
import { runChecked } from "../../lib/exec.mjs";
import { httpOk, logTail } from "../../lib/http.mjs";
import { isWindows, withCargoOnPath } from "../../lib/process.mjs";
import { rootDir as root, runDir } from "../../lib/paths.mjs";
import { waitFor } from "./managed.mjs";

export function prepareEnvFiles() {
  ensureFileFromExample(path.join(root, "infra", ".env"), path.join(root, "infra", ".env.example"));
  ensureFileFromExample(
    path.join(root, "services", "api-rs", ".env"),
    path.join(root, "services", "api-rs", ".env.example"),
  );
  ensureFileFromExample(
    path.join(root, "services", "realtime-rs", ".env"),
    path.join(root, "services", "realtime-rs", ".env.example"),
  );
}

export function readRuntimeProfile(spec) {
  const result = runChecked(process.execPath, [path.join(root, "scripts", "validators", "runtime-profiles.mjs"), "--print", spec]);
  return JSON.parse(result.stdout);
}

function dockerCompose(args, options = {}) {
  return runChecked("docker", ["compose", "--env-file", "infra/.env", "-f", "infra/docker-compose.yml", ...args], {
    stdio: options.stdio ?? "pipe",
  });
}

export async function startInfrastructure(infraEnv) {
  console.log("[local-runtime] Starting local infrastructure");
  dockerCompose(["up", "-d", "postgres", "redis", "minio"]);
  const postgresUser = infraEnv.POSTGRES_USER || "hexrelay";
  const postgresDb = infraEnv.POSTGRES_DB || "hexrelay";
  await waitFor("postgres", () => {
    const result = spawnSync(
      "docker",
      [
        "compose",
        "--env-file",
        "infra/.env",
        "-f",
        "infra/docker-compose.yml",
        "exec",
        "-T",
        "postgres",
        "pg_isready",
        "-U",
        postgresUser,
        "-d",
        postgresDb,
      ],
      { cwd: root, stdio: "ignore", shell: false },
    );
    return result.status === 0;
  });
  await waitFor("redis", () => {
    const result = spawnSync(
      "docker",
      ["compose", "--env-file", "infra/.env", "-f", "infra/docker-compose.yml", "exec", "-T", "redis", "redis-cli", "--raw", "ping"],
      { cwd: root, encoding: "utf8", shell: false },
    );
    return result.status === 0 && result.stdout.includes("PONG");
  });
  await waitFor("minio", () => httpOk("http://localhost:9000/minio/health/live"));
}

export function runSeed(seedProfile, env) {
  if (!seedProfile.trim()) {
    return;
  }
  console.log(`[local-runtime] Seeding local database with '${seedProfile}'`);
  fs.mkdirSync(runDir, { recursive: true });
  const targetDir = path.join(runDir, "targets", `seed-${process.pid}`);
  const stdoutPath = path.join(runDir, "seed.stdout.json");
  const stderrPath = path.join(runDir, "seed.stderr.log");
  const stdout = fs.openSync(stdoutPath, "w");
  const stderr = fs.openSync(stderrPath, "w");
  const result = spawnSync(
    isWindows ? "cargo.exe" : "cargo",
    ["run", "-p", "api-rs", "--bin", "seed_dev", "--", "--profile", seedProfile, "--json"],
    {
      cwd: root,
      env: withCargoOnPath({ ...process.env, ...env, CARGO_TARGET_DIR: targetDir }),
      stdio: ["ignore", stdout, stderr],
      shell: false,
    },
  );
  fs.closeSync(stdout);
  fs.closeSync(stderr);
  if (result.error) {
    fs.rmSync(targetDir, { recursive: true, force: true });
    throw new Error(`failed to start seed process: ${result.error.message}`);
  }
  if (result.status !== 0) {
    const tail = logTail(stderrPath);
    fs.rmSync(targetDir, { recursive: true, force: true });
    throw new Error(`seed profile '${seedProfile}' failed${tail ? `\n${tail}` : ""}`);
  }
  fs.rmSync(targetDir, { recursive: true, force: true });
  console.log(`[local-runtime] Seed output written to ${stdoutPath}`);
}
