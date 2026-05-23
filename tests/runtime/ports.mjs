import process from "node:process";
import { getFreePort } from "../../scripts/lib/http.mjs";

const runtimeDockerPorts = [
  ["HEXRELAY_RUNTIME_TOXIPROXY_PORT", 18474],
  ["HEXRELAY_RUNTIME_ALICE_API_PORT", 18080],
  ["HEXRELAY_RUNTIME_ALICE_REALTIME_PORT", 18081],
  ["HEXRELAY_RUNTIME_ALICE_WEB_PORT", 3002],
  ["HEXRELAY_RUNTIME_BOB_API_PORT", 18180],
  ["HEXRELAY_RUNTIME_BOB_REALTIME_PORT", 18181],
  ["HEXRELAY_RUNTIME_BOB_WEB_PORT", 3012],
];

export async function runtimeDockerEnv() {
  const env = { ...process.env };
  const reservedPorts = new Set();

  for (const [name, preferredPort] of runtimeDockerPorts) {
    const configuredPort = Number(env[name]);
    if (Number.isInteger(configuredPort) && configuredPort > 0) {
      reservedPorts.add(configuredPort);
      continue;
    }
    env[name] = String(await getFreePort(preferredPort, reservedPorts));
  }

  return env;
}
