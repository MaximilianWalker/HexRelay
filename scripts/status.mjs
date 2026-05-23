import process from "node:process";
import { statusCommand } from "./runtime/local.mjs";

statusCommand(process.argv.slice(2)).catch((error) => {
  console.error(`[local-runtime] ERROR: ${error.message}`);
  process.exit(1);
});
