import process from "node:process";
import { startCommand } from "./runtime/local.mjs";

startCommand(process.argv.slice(2)).catch((error) => {
  console.error(`[local-runtime] ERROR: ${error.message}`);
  process.exit(1);
});
