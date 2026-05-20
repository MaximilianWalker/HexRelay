import process from "node:process";
import { exitFromError, runInherited } from "../scripts/lib/exec.mjs";

function parseArgs(args) {
  const options = {
    skipServiceBackedTests: false,
    help: false,
  };

  for (const arg of args) {
    switch (arg) {
      case "--skip-service-backed-tests":
        options.skipServiceBackedTests = true;
        break;
      case "--help":
      case "-h":
        options.help = true;
        break;
      default:
        throw new Error(`unknown test option: ${arg}`);
    }
  }

  return options;
}

try {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    console.log("Usage: node tests/run.mjs [--skip-service-backed-tests]");
    process.exit(0);
  }

  const previousSkip = process.env.HEXRELAY_SKIP_SERVICE_BACKED_TESTS;
  if (options.skipServiceBackedTests) {
    console.log("[test] Skipping external service-backed Rust tests");
    process.env.HEXRELAY_SKIP_SERVICE_BACKED_TESTS = "1";
  }

  console.log("[test] Rust fmt/clippy/test");
  runInherited("cargo", ["fmt", "--all", "--", "--check"]);
  runInherited("cargo", ["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"]);
  runInherited("cargo", ["test", "--all-features"]);

  console.log("[test] Web lint/test/build");
  runInherited("npm", ["run", "lint", "--prefix", "apps/web"]);
  runInherited("npm", ["run", "test:coverage", "--prefix", "apps/web"]);
  runInherited("npm", ["run", "build", "--prefix", "apps/web"]);

  if (options.skipServiceBackedTests) {
    if (previousSkip === undefined) {
      delete process.env.HEXRELAY_SKIP_SERVICE_BACKED_TESTS;
    } else {
      process.env.HEXRELAY_SKIP_SERVICE_BACKED_TESTS = previousSkip;
    }
  }

  console.log("[test] Complete");
} catch (error) {
  exitFromError(error);
}
