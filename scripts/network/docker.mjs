import { spawnSync } from "node:child_process";
import process from "node:process";
import { rootDir as root } from "../lib/paths.mjs";

export const defaultNetworkName = process.env.HEXRELAY_DOCKER_NETWORK || "hexrelay_default";
export const defaultRealtimeInternalToken = process.env.HEXRELAY_REALTIME_INTERNAL_TOKEN || "hexrelay-dev-channel-dispatch-token-change-me";

export function docker(args, options = {}) {
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

export function dockerObjectExists(kind, name) {
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

export function containerNetworkAliases(containerName, networkName) {
  const aliases = containerNetworkMap(containerName)[networkName]?.Aliases ?? [];
  return aliases.filter((alias) => typeof alias === "string" && alias.trim());
}

export function ensureNetwork(networkName) {
  if (!dockerObjectExists("network", networkName)) {
    docker(["network", "create", networkName]);
    return true;
  }
  return false;
}

export function connectIfNeeded(networkName, containerName, aliases = []) {
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

export function disconnectIfConnected(networkName, containerName) {
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

export function resolveRuntimeInstance(target, runtimeState) {
  return (runtimeState?.instances ?? []).find((candidate) => {
    return candidate.id === target || instanceContainerNames(candidate).includes(target);
  });
}

export function resolveTarget(target, runtimeState) {
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

export function normalizeActionTarget(value, argumentTarget, fieldName) {
  if (value === "argument") {
    if (!argumentTarget) {
      throw new Error(`network profile requires --target for ${fieldName}`);
    }
    return argumentTarget;
  }
  return value;
}

export function partitionNetworkName(baseNetworkName, profileName, target) {
  const safe = `${baseNetworkName}-${profileName}-${target}`.replace(/[^a-zA-Z0-9_.-]/g, "-").slice(0, 80);
  return `hexrelay_partition_${safe}`;
}

export function createPartitionNetwork(networkName) {
  if (dockerObjectExists("network", networkName)) {
    throw new Error(`Docker partition network '${networkName}' already exists. Remove the stale network before applying this profile.`);
  }
  docker(["network", "create", networkName]);
}
