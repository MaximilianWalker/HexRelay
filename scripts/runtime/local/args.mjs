export function usage(command = "all") {
  const lines = {
    start: [
      "Usage: npm run start -- [--runtime-profile single|dual|triple|path] [--seed-profile dm-basic]",
      "Default startup uses the clean single profile and does not seed fixture data.",
    ],
    status: ["Usage: npm run status -- [--json]"],
    stop: ["Usage: npm run stop -- [--runtime-profile single|dual|triple|path] [--json]"],
  };
  if (command !== "all") {
    return lines[command].join("\n");
  }
  return [
    "Usage: npm run start|status|stop -- [options]",
    ...lines.start,
    ...lines.status,
    ...lines.stop,
  ].join("\n");
}

function normalizeFlag(arg) {
  switch (arg) {
    case "-RuntimeProfile":
      return "--runtime-profile";
    case "-SeedProfile":
      return "--seed-profile";
    case "-Json":
      return "--json";
    case "-Help":
      return "--help";
    default:
      return arg;
  }
}

function readValue(args, index, flag) {
  const value = args[index + 1];
  if (!value || value.startsWith("-")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function parseStartArgs(rawArgs) {
  const args = rawArgs.map(normalizeFlag);
  const options = { runtimeProfile: "single", seedProfile: "", help: false };
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--runtime-profile" || arg === "-r") {
      options.runtimeProfile = readValue(args, index, arg);
      index += 1;
    } else if (arg === "--seed-profile") {
      options.seedProfile = readValue(args, index, arg);
      index += 1;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown start option: ${arg}\n${usage("start")}`);
    }
  }
  return options;
}

export function parseStatusArgs(rawArgs) {
  const args = rawArgs.map(normalizeFlag);
  const options = { json: false, help: false };
  for (const arg of args) {
    if (arg === "--json") {
      options.json = true;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown status option: ${arg}\n${usage("status")}`);
    }
  }
  return options;
}

export function parseStopArgs(rawArgs) {
  const args = rawArgs.map(normalizeFlag);
  const options = { runtimeProfile: "", json: false, help: false };
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--runtime-profile" || arg === "-r") {
      options.runtimeProfile = readValue(args, index, arg);
      index += 1;
    } else if (arg === "--json") {
      options.json = true;
    } else if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else {
      throw new Error(`unknown stop option: ${arg}\n${usage("stop")}`);
    }
  }
  return options;
}
