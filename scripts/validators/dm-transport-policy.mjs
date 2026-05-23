import fs from "node:fs";
import path from "node:path";
import { rootDir } from "../lib/paths.mjs";

const runtimeForbiddenPattern = /(dm[-_ ]?plain[-_ ]?text|plain[-_ ]?text[-_ ]?dm|clear[-_ ]?text[-_ ]?dm|dm[-_ ]?clear[-_ ]?text|server[-_ ]?readable[-_ ]?dm|dm[-_ ]?server[-_ ]?readable|decrypt[-_ ]?on[-_ ]?server|server[-_ ]?decrypt|server[-_ ]?side[-_ ]?decrypt(ion)?|dm[-_ ]?private[-_ ]?key|private[-_ ]?key[-_ ]?(upload|custody|escrow)|key[-_ ]?escrow|unencrypted[-_ ]?dm[-_ ]?(mailbox|relay|payload|storage)|dm[-_ ]?unencrypted[-_ ]?(mailbox|relay|payload|storage)|plain[-_ ]?text[-_ ]?relay|clear[-_ ]?text[-_ ]?relay|dm[-_ ]?plain[-_ ]?text[-_ ]?relay|dm[-_ ]?clear[-_ ]?text[-_ ]?relay)/i;
const configForbiddenPattern = /(dm[-_ ]?plain[-_ ]?text|plain[-_ ]?text[-_ ]?dm|clear[-_ ]?text[-_ ]?dm|dm[-_ ]?clear[-_ ]?text|server[-_ ]?readable[-_ ]?dm|dm[-_ ]?server[-_ ]?readable|dm[-_ ]?decrypt[-_ ]?on[-_ ]?server|dm[-_ ]?server[-_ ]?decrypt|dm[-_ ]?server[-_ ]?side[-_ ]?decrypt(ion)?|dm[-_ ]?private[-_ ]?key|dm[-_ ]?private[-_ ]?key[-_ ]?(upload|custody|escrow)|dm[-_ ]?key[-_ ]?escrow|dm[-_ ]?unencrypted[-_ ]?(mailbox|relay|payload|storage)|unencrypted[-_ ]?dm[-_ ]?(mailbox|relay|payload|storage)|plain[-_ ]?text[-_ ]?relay|clear[-_ ]?text[-_ ]?relay|dm[-_ ]?plain[-_ ]?text[-_ ]?relay|dm[-_ ]?clear[-_ ]?text[-_ ]?relay)/i;
const contractForbiddenPattern = /(dm_?plain_?text|plain_?text_?dm|clear_?text_?dm|dm_?clear_?text|server_?readable_?dm|dm_?server_?readable|dm_?decrypt_?on_?server|dm_?server_?decrypt|dm_?server_?side_?decrypt(ion)?|dm_?private_?key|dm_?private_?key_?(upload|custody|escrow)|dm_?key_?escrow|dm_?unencrypted_?(mailbox|relay|payload|storage)|unencrypted_?dm_?(mailbox|relay|payload|storage)|plain_?text_?relay|clear_?text_?relay|dm_?plain_?text_?relay|dm_?clear_?text_?relay)/i;
const directDmForbiddenPattern = /(direct[-_ ]?only|direct[-_ ]?peer|DirectPeerTransport|dm[-_ ]?lan[-_ ]?discovery|dm\.lan_discovery|pairing[-_ ]?envelope|\/dm\/connectivity|endpoint[-_ ]?cards?|DmEndpointCard|wan[-_ ]?wizard|DmWanWizard|parallel[-_ ]?dial|DmParallelDial|DmConnectivityPreflight|dm_pairing|dm_lan_presence|dm_pairing_nonces)/;
const contactQrForbiddenPattern = /(QRCodeSVG|qrcode\.react|IconQrcode|link \+ QR|QR code)/;

const allowedDirectDmFragments = [
  "0011_dm_pairing_nonces",
  "0014_dm_endpoint_cards_and_profile_devices",
  "0019_remove_dm_direct_connect_tables",
];

function exists(relativePath) {
  return fs.existsSync(path.join(rootDir, relativePath));
}

function walkFiles(relativeRoots, extensions) {
  const files = [];
  const wanted = new Set(extensions);

  function visit(absolutePath) {
    if (!fs.existsSync(absolutePath)) {
      return;
    }

    const stat = fs.statSync(absolutePath);
    if (stat.isDirectory()) {
      for (const entry of fs.readdirSync(absolutePath)) {
        visit(path.join(absolutePath, entry));
      }
      return;
    }

    if (stat.isFile() && wanted.has(path.extname(absolutePath))) {
      files.push(absolutePath);
    }
  }

  for (const relativeRoot of relativeRoots) {
    visit(path.join(rootDir, relativeRoot));
  }

  return files;
}

function scanFiles(files, pattern, filter = () => true) {
  const matches = [];
  for (const file of files) {
    const relativePath = path.relative(rootDir, file).replaceAll(path.sep, "/");
    const content = fs.readFileSync(file, "utf8");
    const lines = content.split(/\r?\n/);
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      if (pattern.test(line)) {
        const match = `${relativePath}:${index + 1}:${line}`;
        if (filter(match)) {
          matches.push(match);
        }
      }
    }
  }
  return matches;
}

function explicitFiles(relativePaths) {
  return relativePaths.filter(exists).map((file) => path.join(rootDir, file));
}

const matches = [
  ...scanFiles(walkFiles(["crates/communication-core/src"], [".rs"]), runtimeForbiddenPattern),
  ...scanFiles(walkFiles(["services/api-rs/src"], [".rs"]), runtimeForbiddenPattern),
  ...scanFiles(walkFiles(["services/realtime-rs/src"], [".rs"]), runtimeForbiddenPattern),
  ...scanFiles(
    explicitFiles([
      ".github/workflows/ci.yml",
      "docs/reference/runtime-config-reference.md",
      "services/api-rs/src/config.rs",
      "services/realtime-rs/src/config.rs",
    ]),
    configForbiddenPattern,
  ),
  ...scanFiles(walkFiles(["docs/contracts"], [".yaml", ".yml"]), contractForbiddenPattern),
  ...scanFiles(walkFiles(["services/api-rs/migrations"], [".sql"]), contractForbiddenPattern),
  ...scanFiles(walkFiles(["fixtures"], [".json", ".yaml", ".yml", ".rs", ".sql"]), contractForbiddenPattern),
  ...scanFiles(walkFiles(["evidence"], [".json", ".yaml", ".yml", ".md", ".txt"]), contractForbiddenPattern),
  ...scanFiles(
    walkFiles(["crates/communication-core/src", "services/api-rs/src", "services/realtime-rs/src"], [".rs"]),
    directDmForbiddenPattern,
    (match) => !allowedDirectDmFragments.some((fragment) => match.includes(fragment)),
  ),
  ...scanFiles(walkFiles(["apps/web/app", "apps/web/lib"], [".ts", ".tsx"]), directDmForbiddenPattern),
  ...scanFiles(walkFiles(["docs/contracts"], [".yaml", ".yml"]), directDmForbiddenPattern),
  ...scanFiles(walkFiles(["fixtures"], [".json", ".yaml", ".yml"]), directDmForbiddenPattern),
  ...scanFiles(walkFiles(["apps/web/app/contacts"], [".ts", ".tsx"]), contactQrForbiddenPattern),
];

if (matches.length > 0) {
  console.error("::error::Detected forbidden DM plaintext/key-custody terms, retired server-bypassing DM surfaces, or retired contact QR UI.");
  console.error("These terms are disallowed for DM E2EE envelope delivery policy:");
  console.error(matches.join("\n"));
  process.exit(1);
}

console.log("[dm-transport-policy] Runtime, config/workflow, web, and contract surfaces passed DM E2EE envelope policy guardrail");
