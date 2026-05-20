import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptsDir, "../..");
const profilesDir = path.join(root, "scripts", "network", "profiles");
const validStrategies = new Set(["reset", "docker", "toxiproxy", "app-fault"]);
const validPlatforms = new Set(["windows", "linux", "macos"]);
const targetPattern = /^(argument|[a-z][a-z0-9-]*)$/;
const strategyActionTypes = {
  "reset": new Set(["reset"]),
  "docker": new Set(["disconnect", "partition"]),
  "toxiproxy": new Set(["latency", "packet_loss"]),
  "app-fault": new Set(["app_delay", "app_drop", "app_disconnect_after"]),
};

function fail(message) {
  throw new Error(`[network-profiles] ${message}`);
}

function resolveProfilePath(spec) {
  if (!spec || spec.trim() === "") {
    fail("network profile must not be empty");
  }

  const direct = path.resolve(spec);
  if (fs.existsSync(direct)) {
    return direct;
  }

  const name = spec.endsWith(".json") ? spec : `${spec}.json`;
  const bundled = path.join(profilesDir, name);
  if (fs.existsSync(bundled)) {
    return bundled;
  }

  fail(`network profile '${spec}' was not found`);
}

function readProfile(spec) {
  const profilePath = resolveProfilePath(spec);
  const raw = fs.readFileSync(profilePath, "utf8");
  let profile;
  try {
    profile = JSON.parse(raw);
  } catch (error) {
    fail(`${profilePath} is not valid JSON: ${error.message}`);
  }

  validateProfile(profile, profilePath);
  return {
    name: profile.name,
    description: profile.description,
    strategy: profile.strategy,
    supportedPlatforms: profile.supportedPlatforms,
    requiresTarget: profile.actions.some(
      (action) => action.target === "argument" || action.source === "argument",
    ),
    profilePath,
    actions: profile.actions,
  };
}

function validateProfile(profile, profilePath) {
  const label = path.relative(process.cwd(), profilePath) || profilePath;
  if (!profile || typeof profile !== "object" || Array.isArray(profile)) {
    fail(`${label} must contain a JSON object`);
  }
  if (typeof profile.name !== "string" || !/^[a-z][a-z0-9-]*$/.test(profile.name)) {
    fail(`${label} requires name matching /^[a-z][a-z0-9-]*$/`);
  }
  if (typeof profile.description !== "string" || profile.description.trim() === "") {
    fail(`${label} requires a non-empty description`);
  }
  if (!validStrategies.has(profile.strategy)) {
    fail(`${label} has unsupported strategy '${profile.strategy}'`);
  }
  if (!Array.isArray(profile.supportedPlatforms) || profile.supportedPlatforms.length === 0) {
    fail(`${label} requires at least one supported platform`);
  }
  for (const platform of profile.supportedPlatforms) {
    if (!validPlatforms.has(platform)) {
      fail(`${label} has unsupported platform '${platform}'`);
    }
  }
  if (!Array.isArray(profile.actions) || profile.actions.length === 0) {
    fail(`${label} requires at least one action`);
  }

  for (const [index, action] of profile.actions.entries()) {
    validateAction(action, `${label} actions[${index}]`);
    if (!strategyActionTypes[profile.strategy].has(action.type)) {
      fail(`${label} strategy '${profile.strategy}' does not support action '${action.type}'`);
    }
  }
}

function validateAction(action, label) {
  if (!action || typeof action !== "object" || Array.isArray(action)) {
    fail(`${label} must be an object`);
  }
  if (typeof action.type !== "string" || action.type.trim() === "") {
    fail(`${label}.type must be a non-empty string`);
  }

  switch (action.type) {
    case "reset":
      return;
    case "disconnect":
      requireTarget(action.target, `${label}.target`);
      return;
    case "partition":
      requireTarget(action.source, `${label}.source`);
      requireTarget(action.target, `${label}.target`);
      if (action.source === action.target) {
        fail(`${label} source and target must differ`);
      }
      return;
    case "latency":
      requireTarget(action.target, `${label}.target`);
      requireInteger(action.latencyMs, `${label}.latencyMs`, 1, 600000);
      if (action.jitterMs !== undefined) {
        requireInteger(action.jitterMs, `${label}.jitterMs`, 0, 600000);
      }
      return;
    case "packet_loss":
      requireTarget(action.target, `${label}.target`);
      requireNumber(action.lossPercent, `${label}.lossPercent`, 0.01, 100);
      return;
    case "app_delay":
      requireTarget(action.target, `${label}.target`);
      requireInteger(action.delayMs, `${label}.delayMs`, 1, 600000);
      return;
    case "app_drop":
      requireTarget(action.target, `${label}.target`);
      requireNumber(action.dropRate, `${label}.dropRate`, 0.0001, 1);
      return;
    case "app_disconnect_after":
      requireTarget(action.target, `${label}.target`);
      requireInteger(action.seconds, `${label}.seconds`, 1, 86400);
      return;
    default:
      fail(`${label} has unsupported action type '${action.type}'`);
  }
}

function requireTarget(value, label) {
  if (typeof value !== "string" || !targetPattern.test(value)) {
    fail(`${label} must be 'argument' or match /^[a-z][a-z0-9-]*$/`);
  }
}

function requireInteger(value, label, min, max) {
  if (!Number.isInteger(value) || value < min || value > max) {
    fail(`${label} must be an integer from ${min} to ${max}`);
  }
}

function requireNumber(value, label, min, max) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < min || value > max) {
    fail(`${label} must be a number from ${min} to ${max}`);
  }
}

function bundledProfileSpecs() {
  return fs
    .readdirSync(profilesDir)
    .filter((name) => name.endsWith(".json"))
    .sort()
    .map((name) => path.join(profilesDir, name));
}

function main() {
  const args = process.argv.slice(2);
  if (args[0] === "--print") {
    const profile = readProfile(args[1] ?? "normal");
    process.stdout.write(`${JSON.stringify(profile, null, 2)}\n`);
    return;
  }

  const specs = args.length > 0 ? args : bundledProfileSpecs();
  for (const spec of specs) {
    const profile = readProfile(spec);
    console.log(
      `[network-profiles] ${profile.name}: ${profile.strategy}, ${profile.actions.length} action(s)`,
    );
  }
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
