import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptsDir, "..");
const runDir = path.join(root, ".local-run");
const runtimeStatePath = path.join(runDir, "runtime-state.json");
const networkStatePath = path.join(runDir, "network-state.json");
const defaultNetworkName = process.env.HEXRELAY_DOCKER_NETWORK || "hexrelay_default";
const defaultRealtimeInternalToken = process.env.HEXRELAY_REALTIME_INTERNAL_TOKEN || "hexrelay-dev-channel-dispatch-token-change-me";

function usage() {
  return "Usage: npm run network -- [--profile normal|offline-alice|partition-alice-bob|path] [--target instance-id|container] [--reset] [--json] [--force]";
}

function parseArgs(args) {
  const options = {
    profile: "normal",
    target: "",
    reset: false,
    json: false,
    force: false,
    help: false,
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    switch (arg) {
      case "--profile":
      case "-Profile":
      case "-p":
        options.profile = requireValue(args, ++index, arg);
        break;
      case "--target":
      case "-Target":
      case "-t":
        options.target = requireValue(args, ++index, arg);
        break;
      case "--reset":
      case "-Reset":
        options.reset = true;
        break;
      case "--json":
      case "-Json":
        options.json = true;
        break;
      case "--force":
      case "-Force":
        options.force = true;
        break;
      case "--help":
      case "-Help":
      case "-h":
        options.help = true;
        break;
      default:
        if (arg.startsWith("-")) {
          throw new Error(`unknown network option: ${arg}\n${usage()}`);
        }
        throw new Error(`unexpected positional argument: ${arg}\n${usage()}`);
    }
  }

  return options;
}

function requireValue(args, index, flag) {
  const value = args[index];
  if (!value || value.startsWith("-")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function readNetworkProfile(profileSpec) {
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

function readJsonIfExists(filePath) {
  if (!fs.existsSync(filePath)) {
    return null;
  }
  return JSON.parse(stripBom(fs.readFileSync(filePath, "utf8")));
}

function stripBom(value) {
  return value.replace(/^\uFEFF/, "");
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function docker(args, options = {}) {
  const result = spawnSync("docker", args, {
    cwd: root,
    encoding: "utf8",
    shell: false,
  });

  if (options.allowFailure) {
    return result;
  }

  if (result.error) {
    throw new Error(`failed to start docker: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error((result.stderr || result.stdout).trim() || `docker ${args.join(" ")} failed`);
  }

  return result;
}

function dockerObjectExists(kind, name) {
  const result = docker([kind, "inspect", name], { allowFailure: true });
  if (result.error) {
    throw new Error(`failed to start docker: ${result.error.message}`);
  }
  if (result.status === 0) {
    return true;
  }

  const output = (result.stderr || result.stdout || "").trim();
  const lower = output.toLowerCase();
  if (lower.includes("no such") || lower.includes("not found")) {
    return false;
  }

  throw new Error(output || `docker ${kind} inspect ${name} failed`);
}

function containerNetworkMap(containerName) {
  const result = docker(["inspect", "--format", "{{json .NetworkSettings.Networks}}", containerName]);
  const raw = result.stdout.trim();
  if (!raw || raw === "null") {
    return {};
  }
  return JSON.parse(raw);
}

function containerNetworks(containerName) {
  return new Set(Object.keys(containerNetworkMap(containerName)));
}

function containerNetworkAliases(containerName, networkName) {
  const aliases = containerNetworkMap(containerName)[networkName]?.Aliases ?? [];
  return aliases.filter((alias) => typeof alias === "string" && alias.trim());
}

function ensureNetwork(networkName) {
  if (!dockerObjectExists("network", networkName)) {
    docker(["network", "create", networkName]);
    return true;
  }
  return false;
}

function connectIfNeeded(networkName, containerName, aliases = []) {
  if (!dockerObjectExists("container", containerName)) {
    throw new Error(`Docker container '${containerName}' was not found`);
  }
  if (!dockerObjectExists("network", networkName)) {
    throw new Error(`Docker network '${networkName}' was not found`);
  }
  if (!containerNetworks(containerName).has(networkName)) {
    const aliasArgs = aliases.flatMap((alias) => ["--alias", alias]);
    docker(["network", "connect", ...aliasArgs, networkName, containerName]);
    return true;
  }
  return false;
}

function disconnectIfConnected(networkName, containerName) {
  if (!dockerObjectExists("container", containerName)) {
    throw new Error(`Docker container '${containerName}' was not found`);
  }
  if (!dockerObjectExists("network", networkName)) {
    throw new Error(`Docker network '${networkName}' was not found`);
  }
  if (containerNetworks(containerName).has(networkName)) {
    docker(["network", "disconnect", networkName, containerName]);
    return true;
  }
  return false;
}

function instanceContainerNames(instance) {
  return [
    instance.containerName,
    instance.container,
    instance.dockerContainer,
    instance.apiContainerName,
    instance.realtimeContainerName,
    instance.webContainerName,
  ].filter((value) => typeof value === "string" && value.trim());
}

function resolveDirectContainerNetworkName(containerName, preferredNetworkName) {
  if (!dockerObjectExists("container", containerName)) {
    throw new Error(`Docker container '${containerName}' was not found`);
  }

  const networks = containerNetworks(containerName);
  if (preferredNetworkName && networks.has(preferredNetworkName)) {
    return preferredNetworkName;
  }
  if (networks.size === 1) {
    return [...networks][0];
  }
  if (networks.has(defaultNetworkName)) {
    return defaultNetworkName;
  }

  throw new Error(
    `Docker container '${containerName}' is attached to multiple networks; use a runtime instance target or set HEXRELAY_DOCKER_NETWORK`,
  );
}

function resolveRuntimeInstance(target, runtimeState) {
  return (runtimeState?.instances ?? []).find((candidate) => {
    return candidate.id === target || instanceContainerNames(candidate).includes(target);
  });
}

function resolveTarget(target, runtimeState) {
  const instance = resolveRuntimeInstance(target, runtimeState);
  if (!instance) {
    return {
      instanceId: null,
      target,
      containerName: target,
      networkName: resolveDirectContainerNetworkName(target, runtimeState?.networkName),
    };
  }

  const containerName = instance.id === target
    ? instance.containerName || instance.container || instance.dockerContainer
    : target;
  if (!containerName) {
    throw new Error(
      `Runtime instance '${target}' is tracked as host processes, not a Docker container. Docker network simulation requires containerName metadata or a direct Docker container target.`,
    );
  }

  return {
    instanceId: instance.id,
    target,
    containerName,
    networkName: instance.networkName || runtimeState.networkName || defaultNetworkName,
    realtimeUrl: instance.realtimeUrl,
    realtimeInternalToken: instance.realtimeInternalToken || runtimeState.realtimeInternalToken || defaultRealtimeInternalToken,
  };
}

function normalizeActionTarget(value, argumentTarget, fieldName) {
  if (value === "argument") {
    if (!argumentTarget) {
      throw new Error(`network profile requires --target for ${fieldName}`);
    }
    return argumentTarget;
  }
  return value;
}

function partitionNetworkName(baseNetworkName, profileName, target) {
  const safe = `${baseNetworkName}-${profileName}-${target}`.replace(/[^a-zA-Z0-9_.-]/g, "-").slice(0, 80);
  return `hexrelay_partition_${safe}`;
}

function createPartitionNetwork(networkName) {
  if (dockerObjectExists("network", networkName)) {
    throw new Error(`Docker partition network '${networkName}' already exists. Remove the stale network before applying this profile.`);
  }
  docker(["network", "create", networkName]);
}

function toxiproxyTargetState(runtimeState) {
  const toxiproxy = runtimeState?.toxiproxy;
  if (!toxiproxy?.url || !Array.isArray(toxiproxy.proxies)) {
    throw new Error("Toxiproxy profiles require Docker runtime state with toxiproxy metadata");
  }
  return toxiproxy;
}

function resolveToxiproxyTarget(target, runtimeState) {
  const toxiproxy = toxiproxyTargetState(runtimeState);
  const instance = resolveRuntimeInstance(target, runtimeState);
  const sourceId = instance?.id ?? target;
  const proxies = toxiproxy.proxies.filter((proxy) => proxy.sourceId === sourceId);
  if (proxies.length === 0) {
    throw new Error(`Toxiproxy profile target '${target}' was not found in runtime state`);
  }
  return {
    target,
    instanceId: sourceId,
    toxiproxyUrl: toxiproxy.url,
    proxies,
  };
}

function toxiproxyToxic(profileName, action, index) {
  if (action.type === "latency") {
    return {
      name: `hexrelay-${profileName}-latency-${index}`,
      type: "latency",
      stream: "downstream",
      toxicity: 1.0,
      attributes: {
        latency: action.latencyMs,
        jitter: action.jitterMs ?? 0,
      },
    };
  }

  if (action.type === "packet_loss") {
    return {
      name: `hexrelay-${profileName}-timeout-${index}`,
      type: "timeout",
      stream: "downstream",
      toxicity: action.lossPercent / 100,
      attributes: {
        timeout: 0,
      },
    };
  }

  throw new Error(`network action '${action.type}' is not implemented by the Toxiproxy wrapper`);
}

function combineToxiproxyActions(profile, options) {
  const configs = new Map();
  for (const [index, action] of profile.actions.entries()) {
    const target = normalizeActionTarget(action.target, options.target, "target");
    const existing = configs.get(target) ?? { target, toxics: [] };
    existing.toxics.push(toxiproxyToxic(profile.name, action, index));
    configs.set(target, existing);
  }
  return [...configs.values()];
}

async function toxiproxyRequest(baseUrl, method, apiPath, body, options = {}) {
  const response = await fetch(`${baseUrl.replace(/\/$/, "")}${apiPath}`, {
    method,
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  if (options.allowNotFound && response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(`Toxiproxy request failed: HTTP ${response.status} ${text}`);
  }
  return text ? JSON.parse(text) : null;
}

async function readToxiproxyProxy(baseUrl, proxyName) {
  return toxiproxyRequest(baseUrl, "GET", `/proxies/${encodeURIComponent(proxyName)}`);
}

async function addToxiproxyToxic(baseUrl, proxyName, toxic) {
  return toxiproxyRequest(
    baseUrl,
    "POST",
    `/proxies/${encodeURIComponent(proxyName)}/toxics`,
    toxic,
  );
}

async function deleteToxiproxyToxic(baseUrl, proxyName, toxicName) {
  return toxiproxyRequest(
    baseUrl,
    "DELETE",
    `/proxies/${encodeURIComponent(proxyName)}/toxics/${encodeURIComponent(toxicName)}`,
    undefined,
    { allowNotFound: true },
  );
}

async function readRealtimeFaults(realtimeUrl, internalToken) {
  return realtimeFaultRequest("GET", realtimeUrl, internalToken);
}

async function writeRealtimeFaults(realtimeUrl, internalToken, config) {
  return realtimeFaultRequest("POST", realtimeUrl, internalToken, config);
}

async function realtimeFaultRequest(method, realtimeUrl, internalToken, body) {
  if (!realtimeUrl) {
    throw new Error("app-fault profiles require runtime state with realtimeUrl metadata");
  }
  const response = await fetch(`${realtimeUrl.replace(/\/$/, "")}/internal/dev/faults`, {
    method,
    headers: {
      "content-type": "application/json",
      "x-hexrelay-internal-token": internalToken,
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  const payload = text ? JSON.parse(text) : {};
  if (!response.ok) {
    throw new Error(`realtime dev fault request failed for ${realtimeUrl}: HTTP ${response.status} ${text}`);
  }
  return payload;
}

function combineAppFaultActions(profile, options) {
  const configs = new Map();
  for (const action of profile.actions) {
    const target = normalizeActionTarget(action.target, options.target, "target");
    const existing = configs.get(target) ?? { target, delay_ms: 0, drop_rate: 0, disconnect_after_seconds: null };
    if (action.type === "app_delay") {
      existing.delay_ms = action.delayMs;
    } else if (action.type === "app_drop") {
      existing.drop_rate = action.dropRate;
    } else if (action.type === "app_disconnect_after") {
      existing.disconnect_after_seconds = action.seconds;
    } else {
      throw new Error(`network action '${action.type}' is not implemented by the app-fault wrapper`);
    }
    configs.set(target, existing);
  }
  return [...configs.values()];
}

async function resetNetworkState(options = {}) {
  const state = readJsonIfExists(networkStatePath);
  const events = [];
  if (!state) {
    return { reset: true, message: "no network profile is active", events };
  }

  const reversed = [...(state.operations ?? [])].reverse();
  const resetErrors = [];
  for (const operation of reversed) {
    try {
      const requiresContainer = operation.type === "disconnect"
        || operation.type === "partition";
      if (requiresContainer && !dockerObjectExists("container", operation.containerName)) {
        events.push({ type: "container-missing", containerName: operation.containerName, changed: false });
        continue;
      }
      if (operation.type === "disconnect") {
        if (operation.disconnected) {
          ensureNetwork(operation.networkName);
          const connected = connectIfNeeded(operation.networkName, operation.containerName, operation.networkAliases);
          events.push({ type: "connect", containerName: operation.containerName, networkName: operation.networkName, changed: connected });
        }
      }
      if (operation.type === "partition") {
        let restoredBase = !operation.disconnectedBase;
        if (operation.disconnectedBase) {
          try {
            ensureNetwork(operation.networkName);
            const connected = connectIfNeeded(operation.networkName, operation.containerName, operation.networkAliases);
            events.push({ type: "connect", containerName: operation.containerName, networkName: operation.networkName, changed: connected });
            restoredBase = true;
          } catch (error) {
            resetErrors.push(error.message);
          }
        }
        if (restoredBase && operation.connectedPartition && operation.partitionNetworkName && dockerObjectExists("network", operation.partitionNetworkName)) {
          try {
            const disconnected = disconnectIfConnected(operation.partitionNetworkName, operation.containerName);
            events.push({ type: "disconnect", containerName: operation.containerName, networkName: operation.partitionNetworkName, changed: disconnected });
          } catch (error) {
            resetErrors.push(error.message);
          }
        }
      }
      if (operation.type === "toxiproxy") {
        for (const toxicName of operation.toxicNames ?? []) {
          await deleteToxiproxyToxic(operation.toxiproxyUrl, operation.proxyName, toxicName);
        }
        events.push({
          type: "toxiproxy-reset",
          target: operation.target,
          instanceId: operation.instanceId,
          toxiproxyUrl: operation.toxiproxyUrl,
          proxyName: operation.proxyName,
          toxicNames: operation.toxicNames,
          changed: true,
        });
      }
      if (operation.type === "app-fault") {
        await writeRealtimeFaults(operation.realtimeUrl, operation.internalToken || defaultRealtimeInternalToken, operation.previousFaults);
        events.push({
          type: "app-fault-reset",
          target: operation.target,
          instanceId: operation.instanceId,
          realtimeUrl: operation.realtimeUrl,
          changed: true,
        });
      }
    } catch (error) {
      resetErrors.push(error.message);
    }
  }

  for (const networkName of [...(state.createdNetworks ?? [])].reverse()) {
    try {
      if (!dockerObjectExists("network", networkName)) {
        state.createdNetworks = (state.createdNetworks ?? []).filter((value) => value !== networkName);
        writeJson(networkStatePath, state);
        events.push({ type: "remove-network", networkName, changed: false, missing: true });
        continue;
      }
      const result = docker(["network", "rm", networkName], { allowFailure: true });
      if (result.error) {
        resetErrors.push(`failed to start docker while removing '${networkName}': ${result.error.message}`);
        continue;
      }
      if (result.status !== 0) {
        resetErrors.push((result.stderr || result.stdout).trim() || `failed to remove Docker network '${networkName}'`);
        events.push({ type: "remove-network", networkName, changed: false });
        continue;
      }
      state.createdNetworks = (state.createdNetworks ?? []).filter((value) => value !== networkName);
      writeJson(networkStatePath, state);
      events.push({ type: "remove-network", networkName, changed: true });
    } catch (error) {
      resetErrors.push(error.message);
    }
  }

  if (resetErrors.length > 0 && !options.forceStateRemoval) {
    throw new Error(`network reset was incomplete: ${resetErrors.join("; ")}`);
  }

  fs.rmSync(networkStatePath, { force: true });
  return { reset: true, profile: state.profile, events, errors: resetErrors };
}

function applyDockerProfile(profile, options) {
  if (fs.existsSync(networkStatePath)) {
    throw new Error("A network profile is already active. Run npm run network -- --reset before applying another profile.");
  }

  const runtimeState = readJsonIfExists(runtimeStatePath);
  const operations = [];
  const createdNetworks = [];
  const events = [];
  const state = {
    profile: profile.name,
    profilePath: profile.profilePath,
    runtimeProfile: runtimeState?.profile ?? null,
    appliedAt: new Date().toISOString(),
    operations,
    createdNetworks,
  };

  try {
    for (const action of profile.actions) {
      if (action.type === "disconnect") {
        const target = normalizeActionTarget(action.target, options.target, "target");
        const resolved = resolveTarget(target, runtimeState);
        const networkAliases = containerNetworkAliases(resolved.containerName, resolved.networkName);
        const disconnected = disconnectIfConnected(resolved.networkName, resolved.containerName);
        const operation = {
          type: "disconnect",
          target,
          instanceId: resolved.instanceId,
          containerName: resolved.containerName,
          networkName: resolved.networkName,
          networkAliases,
          disconnected,
        };
        operations.push(operation);
        events.push({ ...operation, changed: disconnected });
        writeJson(networkStatePath, state);
        continue;
      }

      if (action.type === "partition") {
        for (const fieldName of ["source", "target"]) {
          const target = normalizeActionTarget(action[fieldName], options.target, fieldName);
          const resolved = resolveTarget(target, runtimeState);
          const partitionName = partitionNetworkName(resolved.networkName, profile.name, target);
          createPartitionNetwork(partitionName);
          createdNetworks.push(partitionName);
          events.push({ type: "create-network", networkName: partitionName, changed: true });
          writeJson(networkStatePath, state);
          const operation = {
            type: "partition",
            target,
            role: fieldName,
            instanceId: resolved.instanceId,
            containerName: resolved.containerName,
            networkName: resolved.networkName,
            networkAliases: containerNetworkAliases(resolved.containerName, resolved.networkName),
            partitionNetworkName: partitionName,
            connectedPartition: false,
            disconnectedBase: false,
          };
          operations.push(operation);
          writeJson(networkStatePath, state);
          operation.connectedPartition = connectIfNeeded(partitionName, resolved.containerName);
          writeJson(networkStatePath, state);
          operation.disconnectedBase = disconnectIfConnected(resolved.networkName, resolved.containerName);
          writeJson(networkStatePath, state);
          events.push({ ...operation });
        }
        continue;
      }

      throw new Error(`network action '${action.type}' is not implemented by the Docker wrapper slice`);
    }
  } catch (error) {
    if (operations.length > 0 || createdNetworks.length > 0) {
      writeJson(networkStatePath, state);
    }
    throw error;
  }

  return { applied: true, profile: profile.name, events };
}

async function applyToxiproxyProfile(profile, options) {
  if (fs.existsSync(networkStatePath)) {
    throw new Error("A network profile is already active. Run npm run network -- --reset before applying another profile.");
  }

  const runtimeState = readJsonIfExists(runtimeStatePath);
  const operations = [];
  const events = [];
  const state = {
    profile: profile.name,
    profilePath: profile.profilePath,
    runtimeProfile: runtimeState?.profile ?? null,
    appliedAt: new Date().toISOString(),
    operations,
    createdNetworks: [],
  };

  for (const config of combineToxiproxyActions(profile, options)) {
    const resolved = resolveToxiproxyTarget(config.target, runtimeState);
    for (const proxy of resolved.proxies) {
      const previousProxy = await readToxiproxyProxy(resolved.toxiproxyUrl, proxy.name);
      const operation = {
        type: "toxiproxy",
        target: config.target,
        instanceId: resolved.instanceId,
        toxiproxyUrl: resolved.toxiproxyUrl,
        proxyName: proxy.name,
        proxy,
        toxicNames: config.toxics.map((toxic) => toxic.name),
        previousProxy,
      };
      operations.push(operation);
      writeJson(networkStatePath, state);

      for (const toxic of config.toxics) {
        await deleteToxiproxyToxic(resolved.toxiproxyUrl, proxy.name, toxic.name);
        const appliedToxic = await addToxiproxyToxic(resolved.toxiproxyUrl, proxy.name, toxic);
        events.push({
          type: "toxiproxy",
          target: config.target,
          instanceId: resolved.instanceId,
          toxiproxyUrl: resolved.toxiproxyUrl,
          proxyName: proxy.name,
          toxicName: toxic.name,
          toxicType: toxic.type,
          stream: toxic.stream,
          toxicity: toxic.toxicity,
          attributes: toxic.attributes,
          appliedToxic,
          changed: true,
        });
      }
      writeJson(networkStatePath, state);
    }
  }

  return { applied: true, profile: profile.name, events };
}

async function applyAppFaultProfile(profile, options) {
  if (fs.existsSync(networkStatePath)) {
    throw new Error("A network profile is already active. Run npm run network -- --reset before applying another profile.");
  }

  const runtimeState = readJsonIfExists(runtimeStatePath);
  const operations = [];
  const events = [];
  const state = {
    profile: profile.name,
    profilePath: profile.profilePath,
    runtimeProfile: runtimeState?.profile ?? null,
    appliedAt: new Date().toISOString(),
    operations,
    createdNetworks: [],
  };

  for (const config of combineAppFaultActions(profile, options)) {
    const resolved = resolveTarget(config.target, runtimeState);
    if (!resolved.realtimeUrl) {
      throw new Error(`Runtime instance '${config.target}' does not expose realtimeUrl metadata for app-fault profiles`);
    }
    const previousFaults = await readRealtimeFaults(resolved.realtimeUrl, resolved.realtimeInternalToken);
    const operation = {
      type: "app-fault",
      target: config.target,
      instanceId: resolved.instanceId,
      realtimeUrl: resolved.realtimeUrl,
      internalToken: resolved.realtimeInternalToken,
      previousFaults,
      config,
    };
    operations.push(operation);
    writeJson(networkStatePath, state);
    const appliedFaults = await writeRealtimeFaults(resolved.realtimeUrl, resolved.realtimeInternalToken, config);
    events.push({
      type: "app-fault",
      target: config.target,
      instanceId: resolved.instanceId,
      realtimeUrl: resolved.realtimeUrl,
      config,
      appliedFaults,
      changed: true,
    });
    writeJson(networkStatePath, state);
  }

  return { applied: true, profile: profile.name, events };
}

async function applyProfile(profile, options) {
  if (profile.strategy === "reset") {
    return resetNetworkState({ forceStateRemoval: options.force });
  }
  if (profile.strategy === "docker") {
    return applyDockerProfile(profile, options);
  }
  if (profile.strategy === "toxiproxy") {
    return applyToxiproxyProfile(profile, options);
  }
  if (profile.strategy === "app-fault") {
    return applyAppFaultProfile(profile, options);
  }
  throw new Error(`network strategy '${profile.strategy}' is validated but not implemented in this wrapper slice`);
}

function writeResult(result, json) {
  if (json) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  if (result.applied) {
    console.log(`[network] Applied network profile '${result.profile}'.`);
  } else if (result.reset) {
    console.log(`[network] Reset network simulation state.${result.profile ? ` Previous profile: ${result.profile}.` : ""}`);
  }
  for (const event of result.events ?? []) {
    console.log(`[network] ${event.type} ${event.containerName ?? event.networkName ?? event.proxyName ?? ""}`.trim());
  }
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    console.log(usage());
    return;
  }

  if (options.reset) {
    writeResult(await resetNetworkState({ forceStateRemoval: options.force }), options.json);
    return;
  }

  const profile = readNetworkProfile(options.profile);
  if (profile.requiresTarget && !options.target) {
    throw new Error(`network profile '${profile.name}' requires --target`);
  }

  writeResult(await applyProfile(profile, options), options.json);
}

try {
  await main();
} catch (error) {
  console.error(`[network] ERROR: ${error.message}`);
  process.exit(1);
}
