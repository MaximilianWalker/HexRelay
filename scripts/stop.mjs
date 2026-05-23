import process from "node:process";
import { stopCommand } from "./runtime/local.mjs";

stopCommand(process.argv.slice(2)).catch((error) => {
  console.error(`[local-runtime] ERROR: ${error.message}`);
  process.exit(1);
});
