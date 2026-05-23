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
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], { stdio: "ignore", shell: false, timeout: 5000 });
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

function listenerPidUnix(port) {
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

export function listenerPids(ports) {
  const uniquePorts = [...new Set(ports.map(Number).filter((port) => Number.isInteger(port) && port > 0))];
  const resultMap = new Map(uniquePorts.map((port) => [port, null]));
  if (uniquePorts.length === 0) {
    return resultMap;
  }

  if (isWindows) {
    const command = `Get-NetTCPConnection -LocalPort ${uniquePorts.join(",")} -State Listen -ErrorAction SilentlyContinue | ForEach-Object { "{0} {1}" -f $_.LocalPort, $_.OwningProcess }`;
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", command], {
      encoding: "utf8",
      shell: false,
    });
    if (result.status === 0) {
      for (const line of result.stdout.split(/\r?\n/)) {
        const match = line.trim().match(/^(\d+)\s+(\d+)$/);
        if (!match) {
          continue;
        }
        const port = Number(match[1]);
        const pid = Number(match[2]);
        if (resultMap.has(port) && Number.isInteger(pid) && pid > 0 && !resultMap.get(port)) {
          resultMap.set(port, pid);
        }
      }
    }
    return resultMap;
  }

  for (const port of uniquePorts) {
    resultMap.set(port, listenerPidUnix(port));
  }
  return resultMap;
}

export function listenerPid(port) {
  return listenerPids([port]).get(Number(port)) ?? null;
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
