import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";
import {
  composeFile,
  instances,
  networkScript,
  projectName,
  root,
  toxiproxyProxies,
  toxiproxyUrl,
} from "./config.mjs";

let jsonOutputMode = false;

export function setJsonOutputMode(value) {
  jsonOutputMode = value;
}

export function logInfo(message) {
  if (jsonOutputMode) {
    console.error(message);
    return;
  }
  console.log(message);
}

export function docker(args, options = {}) {
  const result = spawnSync("docker", args, {
    cwd: root,
    encoding: "utf8",
    stdio: options.capture || jsonOutputMode ? "pipe" : "inherit",
    shell: false,
  });

  if (result.error) {
    throw new Error(`failed to start docker: ${result.error.message}`);
  }
  if (result.status !== 0 && !options.allowFailure) {
    const output = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(output || `docker ${args.join(" ")} failed`);
  }
  return result;
}

export function compose(args, options = {}) {
  return docker(["compose", "-p", projectName, "-f", composeFile, ...args], options);
}

export function composeWithProfiles(profiles, args, options = {}) {
  const profileArgs = profiles.flatMap((profile) => ["--profile", profile]);
  return docker(["compose", "-p", projectName, "-f", composeFile, ...profileArgs, ...args], options);
}

export function runNetwork(args, options = {}) {
  const result = spawnSync(process.execPath, [networkScript, ...args], {
    cwd: root,
    encoding: "utf8",
    stdio: options.capture || jsonOutputMode ? "pipe" : "inherit",
    shell: false,
  });
  if (result.status !== 0 && !options.allowFailure) {
    const output = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
    throw new Error(output || `node ${path.relative(root, networkScript)} ${args.join(" ")} failed`);
  }
  return result;
}

export function runNetworkJson(args) {
  const result = runNetwork([...args, "--json"], { capture: true });
  if (!jsonOutputMode && result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`failed to parse network JSON for '${args.join(" ")}': ${error.message}`);
  }
}

function dockerExec(args, options = {}) {
  return docker(["exec", ...args], options);
}

function timedAppHealthFromContainer(containerName, url) {
  const started = Date.now();
  const result = dockerExec([containerName, "wget", "-q", "-T", "3", "-O", "-", url], {
    allowFailure: true,
    capture: true,
  });
  return {
    ok: result.status === 0 && (result.stdout || "").includes("ok"),
    elapsedMs: Date.now() - started,
  };
}

function appHealthFromContainer(containerName, url) {
  return timedAppHealthFromContainer(containerName, url).ok;
}

function instanceById(id) {
  const instance = instances.find((candidate) => candidate.id === id);
  if (!instance) {
    throw new Error(`unknown runtime instance '${id}'`);
  }
  return instance;
}

async function toxiproxyRequest(method, apiPath, body) {
  const response = await fetch(`${toxiproxyUrl}${apiPath}`, {
    method,
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`Toxiproxy request failed: HTTP ${response.status} ${text}`);
  }
  return text ? JSON.parse(text) : null;
}

export async function populateToxiproxy() {
  await toxiproxyRequest("POST", "/reset");
  await toxiproxyRequest(
    "POST",
    "/populate",
    toxiproxyProxies.map((proxy) => ({
      name: proxy.name,
      listen: proxy.listen,
      upstream: proxy.upstream,
      enabled: true,
    })),
  );
}

export function assertPeerReachability(expectedReachable, label) {
  const [alice, bob] = instances;
  const checks = [
    {
      from: alice,
      to: bob,
      ok: appHealthFromContainer(alice.containerName, "http://bob-server:8080/health"),
    },
    {
      from: bob,
      to: alice,
      ok: appHealthFromContainer(bob.containerName, "http://alice-server:8080/health"),
    },
  ];

  for (const check of checks) {
    if (check.ok !== expectedReachable) {
      throw new Error(
        `peer reachability assertion failed during ${label}: ${check.from.id} -> ${check.to.id} expected ${expectedReachable ? "reachable" : "unreachable"}`,
      );
    }
  }
  logInfo(`[runtime-docker] peer reachability ${label}: ${expectedReachable ? "reachable" : "unreachable"}`);
}

export function assertToxiproxyPeerReachability(expectedReachable, label) {
  for (const proxy of toxiproxyProxies) {
    const source = instanceById(proxy.sourceId);
    const ok = appHealthFromContainer(source.containerName, `${proxy.url}/health`);
    if (ok !== expectedReachable) {
      throw new Error(
        `toxiproxy reachability assertion failed during ${label}: ${proxy.sourceId} -> ${proxy.targetId} expected ${expectedReachable ? "reachable" : "unreachable"}`,
      );
    }
  }
  logInfo(`[runtime-docker] toxiproxy peer reachability ${label}: ${expectedReachable ? "reachable" : "unreachable"}`);
}

export function assertToxiproxyLatency(targetId, minimumMs) {
  const proxies = toxiproxyProxies.filter((candidate) => candidate.sourceId === targetId);
  if (proxies.length === 0) {
    throw new Error(`missing Toxiproxy proxies for '${targetId}'`);
  }
  for (const proxy of proxies) {
    const source = instanceById(proxy.sourceId);
    const result = timedAppHealthFromContainer(source.containerName, `${proxy.url}/health`);
    if (!result.ok) {
      throw new Error(`toxiproxy latency assertion failed: ${proxy.name} health probe failed`);
    }
    if (result.elapsedMs < minimumMs) {
      throw new Error(`toxiproxy latency assertion failed: ${proxy.name} took ${result.elapsedMs}ms, expected at least ${minimumMs}ms`);
    }
    logInfo(`[runtime-docker] toxiproxy latency ${proxy.name}: ${result.elapsedMs}ms`);
  }
}

export function assertToxiproxyBlocked(targetId, label) {
  const proxies = toxiproxyProxies.filter((candidate) => candidate.sourceId === targetId);
  if (proxies.length === 0) {
    throw new Error(`missing Toxiproxy proxies for '${targetId}'`);
  }
  for (const proxy of proxies) {
    const source = instanceById(proxy.sourceId);
    const result = timedAppHealthFromContainer(source.containerName, `${proxy.url}/health`);
    if (result.ok) {
      throw new Error(`toxiproxy blocked assertion failed during ${label}: ${proxy.name} was reachable`);
    }
    logInfo(`[runtime-docker] toxiproxy blocked ${proxy.name}: ${result.elapsedMs}ms`);
  }
}

export async function assertToxiproxyNoToxics(targetId, label) {
  const proxies = toxiproxyProxies.filter((candidate) => candidate.sourceId === targetId);
  if (proxies.length === 0) {
    throw new Error(`missing Toxiproxy proxies for '${targetId}'`);
  }
  for (const proxy of proxies) {
    const proxyState = await toxiproxyRequest("GET", `/proxies/${encodeURIComponent(proxy.name)}`);
    const toxics = Array.isArray(proxyState?.toxics)
      ? proxyState.toxics
      : Object.values(proxyState?.toxics ?? {});
    if (toxics.length > 0) {
      throw new Error(`toxiproxy reset assertion failed during ${label}: ${proxy.name} still has active toxics`);
    }
  }
  logInfo(`[runtime-docker] toxiproxy toxics cleared ${label}`);
}

export async function httpOk(url) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 3000);
  try {
    const response = await fetch(url, { signal: controller.signal });
    return response.status === 200;
  } catch {
    return false;
  } finally {
    clearTimeout(timeout);
  }
}

async function waitFor(label, probe, attempts = 600) {
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    if (await probe()) {
      logInfo(`[runtime-docker] ${label} is ready`);
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(`${label} did not become ready after ${attempts} seconds`);
}

export async function waitForStack() {
  await waitFor("toxiproxy", () => httpOk(`${toxiproxyUrl}/version`));
  for (const instance of instances) {
    await waitFor(`${instance.id} api`, () => httpOk(`${instance.apiUrl}/health`));
    await waitFor(`${instance.id} realtime`, () => httpOk(`${instance.realtimeUrl}/health`));
    await waitFor(`${instance.id} web`, async () => {
      return (await httpOk(instance.webUrl)) || (await httpOk(`${instance.webUrl}/onboarding/identity`));
    });
  }
}
