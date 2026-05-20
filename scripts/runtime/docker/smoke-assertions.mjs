import { instances, networkName } from "./config.mjs";

function requireSmokeEvent(result, description, predicate) {
  const event = (result.events ?? []).find(predicate);
  if (!event) {
    throw new Error(`network smoke assertion failed: ${description}`);
  }
  return event;
}

function requireChangedSmokeEvent(result, description, predicate) {
  const event = requireSmokeEvent(result, description, predicate);
  if (event.changed !== true) {
    throw new Error(`network smoke assertion failed: ${description} did not change Docker state`);
  }
  return event;
}

export function assertOfflineSmoke(applyResult, resetResult) {
  const [alice] = instances;
  requireChangedSmokeEvent(
    applyResult,
    "offline-alice disconnects alice-server from the simulation network",
    (event) => event.type === "disconnect"
      && event.target === alice.id
      && event.containerName === alice.containerName
      && event.networkName === networkName
      && event.disconnected === true,
  );
  requireChangedSmokeEvent(
    resetResult,
    "offline-alice reset reconnects alice-server to the simulation network",
    (event) => event.type === "connect"
      && event.containerName === alice.containerName
      && event.networkName === networkName,
  );
}

export function assertPartitionSmoke(applyResult, resetResult) {
  const createEvents = (applyResult.events ?? []).filter(
    (event) => event.type === "create-network"
      && event.changed === true
      && String(event.networkName ?? "").startsWith("hexrelay_partition_"),
  );
  if (createEvents.length !== instances.length) {
    throw new Error(`network smoke assertion failed: partition created ${createEvents.length} network(s), expected ${instances.length}`);
  }

  for (const instance of instances) {
    requireSmokeEvent(
      applyResult,
      `partition disconnects ${instance.id} from the simulation network and connects it to a partition network`,
      (event) => event.type === "partition"
        && event.target === instance.id
        && event.containerName === instance.containerName
        && event.networkName === networkName
        && event.connectedPartition === true
        && event.disconnectedBase === true,
    );
    requireChangedSmokeEvent(
      resetResult,
      `partition reset reconnects ${instance.id} to the simulation network`,
      (event) => event.type === "connect"
        && event.containerName === instance.containerName
        && event.networkName === networkName,
    );
  }

  const partitionDisconnects = (resetResult.events ?? []).filter(
    (event) => event.type === "disconnect"
      && event.changed === true
      && String(event.networkName ?? "").startsWith("hexrelay_partition_"),
  );
  if (partitionDisconnects.length !== instances.length) {
    throw new Error(`network smoke assertion failed: partition reset disconnected ${partitionDisconnects.length} partition network attachment(s), expected ${instances.length}`);
  }

  const removedNetworks = (resetResult.events ?? []).filter(
    (event) => event.type === "remove-network"
      && event.changed === true
      && String(event.networkName ?? "").startsWith("hexrelay_partition_"),
  );
  if (removedNetworks.length !== instances.length) {
    throw new Error(`network smoke assertion failed: partition reset removed ${removedNetworks.length} partition network(s), expected ${instances.length}`);
  }
}

export function assertToxiproxyLatencySmoke(applyResult, resetResult) {
  requireChangedSmokeEvent(
    applyResult,
    "high-latency applies Toxiproxy latency to alice-server API peer link",
    (event) => event.type === "toxiproxy"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-api"
      && event.toxicType === "latency"
      && event.attributes?.latency === 250,
  );
  requireChangedSmokeEvent(
    applyResult,
    "high-latency applies Toxiproxy latency to alice-server realtime peer link",
    (event) => event.type === "toxiproxy"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-realtime"
      && event.toxicType === "latency"
      && event.attributes?.latency === 250,
  );
  requireChangedSmokeEvent(
    resetResult,
    "high-latency reset clears Toxiproxy latency from alice-server API peer link",
    (event) => event.type === "toxiproxy-reset"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-api",
  );
  requireChangedSmokeEvent(
    resetResult,
    "high-latency reset clears Toxiproxy latency from alice-server realtime peer link",
    (event) => event.type === "toxiproxy-reset"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-realtime",
  );
}

export function assertToxiproxyTimeoutSmoke(applyResult, resetResult) {
  requireChangedSmokeEvent(
    applyResult,
    "packet-loss applies Toxiproxy timeout toxicity to alice-server API peer link",
    (event) => event.type === "toxiproxy"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-api"
      && event.toxicType === "timeout"
      && event.toxicity === 1,
  );
  requireChangedSmokeEvent(
    applyResult,
    "packet-loss applies Toxiproxy timeout toxicity to alice-server realtime peer link",
    (event) => event.type === "toxiproxy"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-realtime"
      && event.toxicType === "timeout"
      && event.toxicity === 1,
  );
  requireChangedSmokeEvent(
    resetResult,
    "packet-loss reset clears Toxiproxy timeout from alice-server API peer link",
    (event) => event.type === "toxiproxy-reset"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-api",
  );
  requireChangedSmokeEvent(
    resetResult,
    "packet-loss reset clears Toxiproxy timeout from alice-server realtime peer link",
    (event) => event.type === "toxiproxy-reset"
      && event.target === "alice-server"
      && event.proxyName === "alice-to-bob-realtime",
  );
}

export function assertAppFaultSmoke(applyResult, resetResult) {
  requireChangedSmokeEvent(
    applyResult,
    "flaky-mobile applies realtime app faults to alice-server",
    (event) => event.type === "app-fault"
      && event.target === "alice-server"
      && event.config?.delay_ms === 200
      && event.config?.drop_rate === 0.05
      && event.config?.disconnect_after_seconds === 45
      && event.appliedFaults?.enabled === true,
  );
  requireChangedSmokeEvent(
    resetResult,
    "flaky-mobile reset restores realtime app faults on alice-server",
    (event) => event.type === "app-fault-reset"
      && event.target === "alice-server",
  );
}
