import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import process from "node:process";

export const isWindows = process.platform === "win32";

export function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function withCargoOnPath(env) {
  const cargoBin = path.join(os.homedir(), ".cargo", "bin");
  const currentPath = env.PATH ?? env.Path ?? process.env.PATH ?? "";
  const hasCargoBin = currentPath
    .split(path.delimiter)
    .some((entry) => entry.toLowerCase() === cargoBin.toLowerCase());
  if (hasCargoBin) {
    return env;
  }
  return { ...env, PATH: `${cargoBin}${path.delimiter}${currentPath}` };
}

export function isProcessAlive(pid) {
  if (!Number.isInteger(Number(pid)) || Number(pid) <= 0) {
    return false;
  }
  try {
    process.kill(Number(pid), 0);
    return true;
  } catch (error) {
    return error?.code === "EPERM";
  }
}

export async function killProcessTree(pid) {
  if (!isProcessAlive(pid)) {
    return false;
  }
  if (isWindows) {
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], { stdio: "ignore", shell: false });
    await delay(500);
    return !isProcessAlive(pid);
  }
  try {
    process.kill(-Number(pid), "SIGTERM");
  } catch {
    try {
      process.kill(Number(pid), "SIGTERM");
    } catch {
    }
  }
  await delay(1000);
  if (isProcessAlive(pid)) {
    try {
      process.kill(-Number(pid), "SIGKILL");
    } catch {
      try {
        process.kill(Number(pid), "SIGKILL");
      } catch {
      }
    }
  }
  return true;
}

export function listenerPid(port) {
  if (isWindows) {
    const command = [
      `$c = Get-NetTCPConnection -LocalPort ${Number(port)} -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1;`,
      "if ($c) { $c.OwningProcess }",
    ].join(" ");
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    const pid = Number(result.stdout?.trim());
    return result.status === 0 && Number.isInteger(pid) && pid > 0 ? pid : null;
  }

  const lsof = spawnSync("lsof", ["-nP", `-iTCP:${Number(port)}`, "-sTCP:LISTEN", "-t"], {
    encoding: "utf8",
    shell: false,
  });
  if (lsof.status === 0) {
    const pid = Number(lsof.stdout.split(/\r?\n/).find(Boolean));
    if (Number.isInteger(pid) && pid > 0) {
      return pid;
    }
  }

  const ss = spawnSync("ss", ["-ltnp", `sport = :${Number(port)}`], {
    encoding: "utf8",
    shell: false,
  });
  if (ss.status === 0) {
    const match = ss.stdout.match(/pid=(\d+)/);
    const pid = match ? Number(match[1]) : null;
    if (Number.isInteger(pid) && pid > 0) {
      return pid;
    }
  }

  return null;
}

export function processCommandLine(pid) {
  if (!isProcessAlive(pid)) {
    return "";
  }
  if (isWindows) {
    const command = `$p = Get-CimInstance Win32_Process -Filter "ProcessId = ${Number(pid)}" -ErrorAction SilentlyContinue; if ($p) { $p.CommandLine }`;
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    return result.status === 0 ? result.stdout.trim() : "";
  }
  const result = spawnSync("ps", ["-p", String(pid), "-o", "args="], {
    encoding: "utf8",
    shell: false,
  });
  return result.status === 0 ? result.stdout.trim() : "";
}

export function processParentPid(pid) {
  if (!isProcessAlive(pid)) {
    return null;
  }
  if (isWindows) {
    const command = `$p = Get-CimInstance Win32_Process -Filter "ProcessId = ${Number(pid)}" -ErrorAction SilentlyContinue; if ($p) { $p.ParentProcessId }`;
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    const parentPid = Number(result.stdout?.trim());
    return result.status === 0 && Number.isInteger(parentPid) && parentPid > 0 ? parentPid : null;
  }
  const result = spawnSync("ps", ["-p", String(pid), "-o", "ppid="], {
    encoding: "utf8",
    shell: false,
  });
  const parentPid = Number(result.stdout?.trim());
  return result.status === 0 && Number.isInteger(parentPid) && parentPid > 0 ? parentPid : null;
}

export function processName(pid) {
  if (!isProcessAlive(pid)) {
    return "";
  }
  if (isWindows) {
    const command = `$p = Get-CimInstance Win32_Process -Filter "ProcessId = ${Number(pid)}" -ErrorAction SilentlyContinue; if ($p) { $p.Name }`;
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    return result.status === 0 ? result.stdout.trim() : "";
  }
  const result = spawnSync("ps", ["-p", String(pid), "-o", "comm="], {
    encoding: "utf8",
    shell: false,
  });
  return result.status === 0 ? result.stdout.trim() : "";
}

export function uniquePids(values) {
  return [...new Set(values.map(Number).filter((pid) => Number.isInteger(pid) && pid > 0))];
}
