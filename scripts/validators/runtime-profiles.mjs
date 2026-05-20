import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptsDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptsDir, "../..");
const profilesDir = path.join(root, "scripts", "runtime", "profiles");

function fail(message) {
  throw new Error(`[runtime-profiles] ${message}`);
}

function resolveProfilePath(spec) {
  if (!spec || spec.trim() === "") {
    fail("runtime profile must not be empty");
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

  fail(`runtime profile '${spec}' was not found`);
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
    infraMode: profile.infraMode ?? "shared",
    profilePath,
    instances: profile.instances.map((instance) => ({
      id: instance.id,
      apiPort: instance.apiPort,
      realtimePort: instance.realtimePort,
      webPort: instance.webPort,
      seedPersona: instance.seedPersona ?? null,
    })),
  };
}

function validateProfile(profile, profilePath) {
  const label = path.relative(process.cwd(), profilePath) || profilePath;
  if (!profile || typeof profile !== "object" || Array.isArray(profile)) {
    fail(`${label} must contain a JSON object`);
  }
  if (typeof profile.name !== "string" || profile.name.trim() === "") {
    fail(`${label} requires a non-empty string name`);
  }
  if (profile.infraMode !== undefined && profile.infraMode !== "shared") {
    fail(`${label} only supports infraMode 'shared'`);
  }
  if (!Array.isArray(profile.instances) || profile.instances.length === 0) {
    fail(`${label} requires at least one instance`);
  }

  const ids = new Set();
  const ports = new Set();
  for (const [index, instance] of profile.instances.entries()) {
    const prefix = `${label} instances[${index}]`;
    if (!instance || typeof instance !== "object" || Array.isArray(instance)) {
      fail(`${prefix} must be an object`);
    }
    if (typeof instance.id !== "string" || !/^[a-z][a-z0-9-]*$/.test(instance.id)) {
      fail(`${prefix}.id must match /^[a-z][a-z0-9-]*$/`);
    }
    if (ids.has(instance.id)) {
      fail(`${label} has duplicate instance id '${instance.id}'`);
    }
    ids.add(instance.id);

    for (const key of ["apiPort", "realtimePort", "webPort"]) {
      const value = instance[key];
      if (!Number.isInteger(value) || value < 1 || value > 65535) {
        fail(`${prefix}.${key} must be an integer TCP port`);
      }
      if (ports.has(value)) {
        fail(`${label} has duplicate port ${value}`);
      }
      ports.add(value);
    }

    if (instance.seedPersona !== undefined && (typeof instance.seedPersona !== "string" || instance.seedPersona.trim() === "")) {
      fail(`${prefix}.seedPersona must be a non-empty string when present`);
    }
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
    const profile = readProfile(args[1] ?? "single");
    process.stdout.write(`${JSON.stringify(profile, null, 2)}\n`);
    return;
  }

  const specs = args.length > 0 ? args : bundledProfileSpecs();
  for (const spec of specs) {
    const profile = readProfile(spec);
    console.log(`[runtime-profiles] ${profile.name}: ${profile.instances.length} instance(s)`);
  }
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
