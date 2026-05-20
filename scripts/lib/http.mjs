import fs from "node:fs";
import net from "node:net";
import { listenerPid } from "./process.mjs";

export async function portInUseOnHost(port, host) {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", (error) => {
      if (error.code === "EADDRINUSE" || error.code === "EACCES") {
        resolve(true);
      } else if (error.code === "EADDRNOTAVAIL") {
        resolve(false);
      } else {
        reject(error);
      }
    });
    server.once("listening", () => {
      server.close(() => resolve(false));
    });
    server.listen(port, host);
  });
}

export async function portInUse(port) {
  if (listenerPid(port)) {
    return true;
  }
  return (await portInUseOnHost(port, "127.0.0.1")) || (await portInUseOnHost(port, "::1"));
}

export async function getFreePort(preferredPort, reservedPorts) {
  let port = preferredPort;
  while ((await portInUse(port)) || reservedPorts.has(port)) {
    if (port >= 65535) {
      throw new Error(`no available TCP port at or above ${preferredPort}`);
    }
    port += 1;
  }
  reservedPorts.add(port);
  return port;
}

export async function httpOk(url, timeoutMs = 5000) {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(timeoutMs) });
    return response.status === 200;
  } catch {
    return false;
  }
}

export async function webReady(url) {
  return (await httpOk(url)) || (await httpOk(`${url.replace(/\/$/, "")}/onboarding/identity`));
}

export function logTail(filePath, lines = 40) {
  if (!fs.existsSync(filePath)) {
    return "";
  }
  const raw = fs.readFileSync(filePath, "utf8");
  return raw.split(/\r?\n/).slice(-lines).join("\n").trim();
}
