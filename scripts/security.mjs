import { exitFromError } from "./lib/exec.mjs";
import { runCargoAudit } from "./security/cargo-audit.mjs";

try {
  runCargoAudit();
} catch (error) {
  exitFromError(error);
}
