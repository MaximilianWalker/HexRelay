import fs from "node:fs";
import path from "node:path";
import { httpOk, logTail, webReady } from "../../lib/http.mjs";
import { writeJson } from "../../lib/json.mjs";
import { delay, isProcessAlive, isWindows, killProcessTree } from "../../lib/process.mjs";
import { rootDir as root } from "../../lib/paths.mjs";
import { startManagedProcess, waitFor, writeStartupLogTail } from "./managed.mjs";

function writeRuntimeTsConfig(distId) {
  const runtimeTsConfigDir = path.join(root, "apps", "web", ".runtime-tsconfig");
  fs.mkdirSync(runtimeTsConfigDir, { recursive: true });
  writeJson(path.join(runtimeTsConfigDir, `${distId}.json`), {
    extends: "../tsconfig.json",
    include: [
      "../next-env.d.ts",
      "../**/*.ts",
      "../**/*.tsx",
      `../.next-${distId}/types/**/*.ts`,
      `../.next-${distId}/dev/types/**/*.ts`,
      "../**/*.mts",
    ],
    exclude: ["../node_modules"],
  });
}

function findExistingNextPid(stderrPath) {
  const tail = logTail(stderrPath, 80);
  const match = tail.match(/PID:\s+(\d+)/);
  return match ? Number(match[1]) : null;
}

export async function startWebWithRetry({ instanceId, webPort, webEnv, webDistId, logDir, startedProcesses }) {
  const webDir = path.join(root, "apps", "web");
  const webBaseUrl = `http://localhost:${webPort}`;
  let webUrl = webBaseUrl;
  let processInfo = null;
  if (webDistId) {
    writeRuntimeTsConfig(webDistId);
  }

  for (let attempt = 1; attempt <= 2; attempt += 1) {
    processInfo = startManagedProcess({
      name: "web",
      cwd: webDir,
      env: webEnv,
      command: isWindows ? ".\\node_modules\\.bin\\next.cmd" : "./node_modules/.bin/next",
      args: ["dev", "--port", String(webPort)],
      logDir,
    });
    startedProcesses.push(processInfo);
    await waitFor(
      `${instanceId} web`,
      async () => {
        if (logTail(processInfo.stdoutPath, 20).includes("Ready in")) {
          return true;
        }
        if (logTail(processInfo.stderrPath, 40).includes("Another next dev server is already running")) {
          return true;
        }
        return httpOk(webBaseUrl);
      },
      {
        failureProbe: () => {
          if (logTail(processInfo.stderrPath, 40).includes("Another next dev server is already running")) {
            return false;
          }
          return !isProcessAlive(processInfo.child.pid);
        },
        onFailure: () => writeStartupLogTail(`${instanceId} web`, processInfo.stdoutPath, processInfo.stderrPath),
      },
    );

    const stderrTail = logTail(processInfo.stderrPath, 80);
    if (stderrTail.includes("Another next dev server is already running")) {
      const existingPid = findExistingNextPid(processInfo.stderrPath);
      if (existingPid && attempt < 2) {
        console.log(`[local-runtime] Stopping stale Next dev server PID ${existingPid} and retrying ${instanceId} web startup`);
        await killProcessTree(existingPid);
        await delay(2000);
        continue;
      }
      throw new Error(
        existingPid
          ? `Another Next dev server PID ${existingPid} is still running. Stop it and rerun npm run start.`
          : "Another Next dev server is already running, but its PID could not be determined. Stop it and rerun npm run start.",
      );
    }
    break;
  }

  await waitFor(`${instanceId} web HTTP`, () => webReady(webUrl), {
    attempts: 60,
    failureProbe: () => !isProcessAlive(processInfo.child.pid),
    onFailure: () => writeStartupLogTail(`${instanceId} web HTTP`, processInfo.stdoutPath, processInfo.stderrPath),
  });

  return { processInfo, webUrl };
}
