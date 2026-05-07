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

function usage() {
  return "Usage: network.mjs [--profile normal|offline-alice|partition-alice-bob|path] [--target instance-id|container] [--reset] [--json] [--force]";
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
  const validatorPath = path.join(scriptsDir, "validate-network-profiles.mjs");
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

function resolveTarget(target, runtimeState) {
  const instance = (runtimeState?.instances ?? []).find((candidate) => candidate.id === target);
  if (!instance) {
    return {
      instanceId: null,
      target,
      containerName: target,
      networkName: defaultNetworkName,
    };
  }

  const containerName = instance.containerName || instance.container || instance.dockerContainer;
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

function resetNetworkState(options = {}) {
  const state = readJsonIfExists(networkStatePath);
  const events = [];
  if (!state) {
    return { reset: true, message: "no network profile is active", events };
  }

  const reversed = [...(state.operations ?? [])].reverse();
  const resetErrors = [];
  for (const operation of reversed) {
    try {
      if (!dockerObjectExists("container", operation.containerName)) {
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
    throw new Error("A network profile is already active. Run scripts/network.mjs --reset before applying another profile.");
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

function applyProfile(profile, options) {
  if (profile.strategy === "reset") {
    return resetNetworkState({ forceStateRemoval: options.force });
  }
  if (profile.strategy === "docker") {
    return applyDockerProfile(profile, options);
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
    console.log(`[network] ${event.type} ${event.containerName ?? event.networkName ?? ""}`.trim());
  }
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    console.log(usage());
    return;
  }

  if (options.reset) {
    writeResult(resetNetworkState({ forceStateRemoval: options.force }), options.json);
    return;
  }

  const profile = readNetworkProfile(options.profile);
  if (profile.requiresTarget && !options.target) {
    throw new Error(`network profile '${profile.name}' requires --target`);
  }

  writeResult(applyProfile(profile, options), options.json);
}

try {
  main();
} catch (error) {
  console.error(`[network] ERROR: ${error.message}`);
  process.exit(1);
}
