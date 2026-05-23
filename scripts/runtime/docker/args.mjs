export function usage() {
  return "Usage: scripts/runtime/docker.mjs up|down|status|smoke [--seed-profile dm-basic] [--scope all|runtime|network] [--evidence-dir path] [--json] [--force]";
}

export function parseArgs(args) {
  const options = {
    command: "status",
    seedProfile: "",
    scope: "all",
    evidenceDir: "",
    json: false,
    force: false,
    help: false,
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    switch (arg) {
      case "up":
      case "down":
      case "status":
      case "smoke":
        options.command = arg;
        break;
      case "--seed-profile":
      case "-SeedProfile":
        options.seedProfile = requireValue(args, ++index, arg);
        break;
      case "--scope":
      case "-Scope":
        options.scope = requireValue(args, ++index, arg);
        if (!["all", "runtime", "network"].includes(options.scope)) {
          throw new Error(`${arg} must be one of: all, runtime, network`);
        }
        break;
      case "--evidence-dir":
      case "-EvidenceDir":
        options.evidenceDir = requireValue(args, ++index, arg);
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
        throw new Error(`unknown runtime docker option: ${arg}\n${usage()}`);
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
