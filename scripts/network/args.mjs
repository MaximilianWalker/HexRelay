export function usage() {
  return "Usage: npm run network -- [--profile normal|offline-alice|partition-alice-bob|path] [--target instance-id|container] [--reset] [--json] [--force]";
}

export function parseArgs(args) {
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
