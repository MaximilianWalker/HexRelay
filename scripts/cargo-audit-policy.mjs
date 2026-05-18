#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

const policyScript = "node scripts/cargo-audit-policy.mjs";
const advisoryIgnores = [
  { id: "RUSTSEC-2023-0071", expiresUtc: "2026-06-30" },
  { id: "RUSTSEC-2026-0049", expiresUtc: "2026-09-30" },
  { id: "RUSTSEC-2026-0097", expiresUtc: "2026-08-31" },
];

function usage() {
  console.error(
    [
      "Usage: node scripts/cargo-audit-policy.mjs <command>",
      "",
      "Commands:",
      "  audit       Install the pinned cargo-audit version, then run the canonical audit command.",
      "  validate    Fail if any temporary advisory ignore has expired.",
      "  check       Fail if CI, npm, docs, or wrapper surfaces bypass this policy.",
      "  list        Print active temporary advisory ignores and expiry dates.",
    ].join("\n"),
  );
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: false,
    ...options,
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  if (result.signal) {
    console.error(`${command} terminated with signal ${result.signal}`);
    process.exit(1);
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function ensureCargoBinOnPath() {
  const cargoHome =
    process.env.CARGO_HOME ||
    (process.env.USERPROFILE
      ? path.join(process.env.USERPROFILE, ".cargo")
      : path.join(process.env.HOME || "", ".cargo"));
  const cargoBin = path.join(cargoHome, "bin");
  const delimiter = process.platform === "win32" ? ";" : ":";
  const currentPath = process.env.PATH || "";
  const paths = currentPath.split(delimiter).filter(Boolean);

  if (!paths.some((entry) => path.resolve(entry) === path.resolve(cargoBin))) {
    process.env.PATH = `${cargoBin}${delimiter}${currentPath}`;
  }
}

function cargoAuditArgs() {
  return [
    "audit",
    "--deny",
    "warnings",
    ...advisoryIgnores.flatMap(({ id }) => ["--ignore", id]),
  ];
}

function ensureCargoAudit() {
  ensureCargoBinOnPath();

  if (process.platform === "win32") {
    run("powershell.exe", [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      "scripts/ensure-cargo-audit.ps1",
    ]);
    return;
  }

  run("bash", ["scripts/ensure-cargo-audit.sh"]);
}

function todayUtcFromArgs(args) {
  const todayIndex = args.indexOf("--today");
  if (todayIndex === -1) {
    return new Date().toISOString().slice(0, 10);
  }

  const value = args[todayIndex + 1];
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value || "")) {
    console.error("Expected --today in YYYY-MM-DD format.");
    process.exit(1);
  }
  return value;
}

function validate(args) {
  const todayUtc = todayUtcFromArgs(args);
  let failed = false;

  for (const { id, expiresUtc } of advisoryIgnores) {
    if (todayUtc > expiresUtc) {
      console.error(`::error::cargo-audit ignore ${id} expired on ${expiresUtc}.`);
      console.error("Remove the ignore or renew with explicit rationale in the same PR.");
      failed = true;
    } else {
      console.log(`[security] cargo-audit ignore ${id} valid until ${expiresUtc}`);
    }
  }

  process.exit(failed ? 1 : 0);
}

function audit() {
  ensureCargoAudit();
  ensureCargoBinOnPath();

  const args = cargoAuditArgs();
  console.log(`[security] Running cargo ${args.join(" ")}`);
  run(process.platform === "win32" ? "cargo.exe" : "cargo", args);
}

function list() {
  for (const { id, expiresUtc } of advisoryIgnores) {
    console.log(`${id} expires ${expiresUtc}`);
  }
}

function readText(filePath) {
  return readFileSync(filePath, "utf8");
}

function check() {
  const errors = [];
  const rootPackage = JSON.parse(readText("package.json"));

  if (rootPackage.scripts?.security !== `${policyScript} audit`) {
    errors.push(`package.json scripts.security must be "${policyScript} audit".`);
  }

  const ci = readText(".github/workflows/ci.yml");
  const wrapper = readText("scripts/validate-cargo-audit-ignore.sh");
  const runbook = readText("docs/operations/01-mvp-runbook.md");
  const contributorGuide = readText("docs/operations/contributor-guide.md");

  for (const command of [
    `${policyScript} check`,
    `${policyScript} validate`,
    `${policyScript} audit`,
  ]) {
    if (!ci.includes(command)) {
      errors.push(`.github/workflows/ci.yml must run "${command}".`);
    }
  }

  if (!wrapper.includes("scripts/cargo-audit-policy.mjs validate")) {
    errors.push("scripts/validate-cargo-audit-ignore.sh must delegate to scripts/cargo-audit-policy.mjs validate.");
  }

  const canonicalDocsText = `${runbook}\n${contributorGuide}`;
  for (const command of [
    `${policyScript} audit`,
    `${policyScript} validate`,
    `${policyScript} list`,
  ]) {
    if (!canonicalDocsText.includes(command)) {
      errors.push(`canonical docs must mention "${command}".`);
    }
  }

  const driftSurfaces = {
    "package.json": JSON.stringify(rootPackage, null, 2),
    ".github/workflows/ci.yml": ci,
    "scripts/validate-cargo-audit-ignore.sh": wrapper,
    "docs/operations/01-mvp-runbook.md": runbook,
    "docs/operations/contributor-guide.md": contributorGuide,
  };
  for (const [filePath, text] of Object.entries(driftSurfaces)) {
    if (/RUSTSEC-\d{4}-\d{4}/.test(text)) {
      errors.push(`${filePath} must not hardcode cargo-audit advisory IDs; use scripts/cargo-audit-policy.mjs.`);
    }
    if (/cargo audit\s+--deny\s+warnings\s+--ignore/.test(text)) {
      errors.push(`${filePath} must not hardcode cargo-audit ignore arguments; use scripts/cargo-audit-policy.mjs.`);
    }
  }

  if (errors.length > 0) {
    for (const error of errors) {
      console.error(`::error::${error}`);
    }
    process.exit(1);
  }

  console.log("[security] cargo-audit policy surfaces route through scripts/cargo-audit-policy.mjs");
}

const [command, ...args] = process.argv.slice(2);

switch (command) {
  case "audit":
    audit();
    break;
  case "validate":
    validate(args);
    break;
  case "check":
    check();
    break;
  case "list":
    list();
    break;
  default:
    usage();
    process.exit(1);
}
