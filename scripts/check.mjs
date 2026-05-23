import process from "node:process";
import { exitFromError, runInherited } from "./lib/exec.mjs";
import { resolveBaseSha } from "./lib/git.mjs";

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
        throw new Error(`unknown check option: ${arg}`);
    }
  }

  return options;
}

function printHelp() {
  console.log("Usage: node scripts/check.mjs [--skip-service-backed-tests]");
}

function runNode(args) {
  runInherited("node", args);
}

try {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    printHelp();
    process.exit(0);
  }

  const baseSha = resolveBaseSha(process.env.BASE_SHA);
  const headSha = process.env.HEAD_SHA || "HEAD";

  console.log("[check] Security");
  runNode(["scripts/validators/cargo-audit-ignore.mjs"]);
  runNode(["scripts/security.mjs"]);

  console.log("[check] Documentation and contract validators");
  runNode(["scripts/validators/migration-evidence.mjs", baseSha, headSha]);
  runNode(["scripts/validators/evidence-provenance.mjs", baseSha, headSha]);
  runNode(["scripts/validators/docs-index-freshness.mjs", baseSha, headSha]);
  runNode(["scripts/validators/contract-parity.mjs", baseSha, headSha]);
  runNode(["tests/contract-parity/run.mjs"]);
  runNode(["scripts/validators/dm-transport-policy.mjs"]);

  console.log("[check] Profile validators");
  runInherited("npm", ["run", "validate:runtime-profiles"]);
  runInherited("npm", ["run", "validate:network-profiles"]);

  console.log("[check] Test suite");
  runNode(["tests/run.mjs", ...(options.skipServiceBackedTests ? ["--skip-service-backed-tests"] : [])]);
} catch (error) {
  exitFromError(error);
}
