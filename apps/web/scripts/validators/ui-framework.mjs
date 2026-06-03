import { readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join } from "node:path";

const roots = ["app", "components"];
const allowedPressedFiles = new Set([
  "components/hubs/item.tsx",
  "components/ui/button.tsx",
  "components/ui/list-action-button.tsx",
  "components/ui/menu.tsx",
]);
const bannedControlPatterns = [
  /\bbackButton\b/,
  /\bsendButton\b/,
  /\bchannelButton\b/,
  /\bchannelBadge\b/,
  /\bcontextMenuItem\b/,
  /\bserverMenuButton\b/,
  /\bbuttonGhost\b/,
];
const ignoredDirectories = new Set([".next", "coverage", "node_modules"]);
const failures = [];

function normalizePath(path) {
  return path.replaceAll("\\", "/");
}

function walk(directory, visitFile) {
  for (const entry of readdirSync(directory)) {
    const fullPath = join(directory, entry);
    const stat = statSync(fullPath);

    if (stat.isDirectory()) {
      if (!ignoredDirectories.has(entry)) {
        walk(fullPath, visitFile);
      }
      continue;
    }

    if ([".css", ".ts", ".tsx"].includes(extname(entry))) {
      visitFile(fullPath);
    }
  }
}

function auditPressedButtons(fullPath, source) {
  const normalizedPath = normalizePath(fullPath);
  if (allowedPressedFiles.has(normalizedPath) || normalizedPath.includes(".test.")) {
    return;
  }

  if (source.includes("aria-pressed=")) {
    failures.push(`${normalizedPath}: pressed state must go through shared button/menu/list-action primitives`);
  }
}

function auditControlCopies(fullPath, source) {
  const normalizedPath = normalizePath(fullPath);
  if (normalizedPath.startsWith("components/ui/")) {
    return;
  }

  bannedControlPatterns.forEach((pattern) => {
    if (pattern.test(source)) {
      failures.push(`${normalizedPath}: repeated control class matched ${pattern}`);
    }
  });

}

for (const root of roots) {
  walk(root, (fullPath) => {
    const source = readFileSync(fullPath, "utf8");

    auditPressedButtons(fullPath, source);
    auditControlCopies(fullPath, source);
  });
}

if (failures.length > 0) {
  process.stderr.write(`UI framework audit failed: repeated controls must use shared APIs.\n${failures.join("\n")}\n`);
  process.exit(1);
}

process.stdout.write("UI framework audit passed\n");
