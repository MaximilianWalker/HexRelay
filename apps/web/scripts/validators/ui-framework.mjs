import { readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join } from "node:path";

const roots = ["app", "components"];
const allowedPressedFiles = new Set([
  "components/ui/button.tsx",
  "components/ui/list-action-button.tsx",
  "components/ui/menu.tsx",
]);
const allowedRawButtonFiles = new Set([
  "components/content-tabs/bar.tsx",
  "components/layout/tabs/root.tsx",
]);
const primitiveComponentPattern =
  /<(?:Button|ButtonLink|IconButton|Badge|Alert|Menu|MenuItem|MenuRow|Popup|ListActionButton|ToggleButton|ToggleSwitch|ButtonGroup)\b[^>]*\bclassName=/s;
const primitiveIconClassPattern =
  /\b(?:icon|trailing)=\{<Icon[A-Za-z0-9]+\b[^}]*\bclassName=\{(?:styles|[A-Za-z0-9]+Styles)\.icon\}/s;
const iconButtonChildClassPattern =
  /<IconButton\b[^>]*>[\s\n\r]*<Icon[A-Za-z0-9]+\b[^>]*\bclassName=\{(?:styles|[A-Za-z0-9]+Styles)\.icon\}/s;
const bannedControlPatterns = [
  /\bbackButton\b/,
  /\bsendButton\b/,
  /\bchannelButton\b/,
  /\bchannelBadge\b/,
  /\bcontextMenuItem\b/,
  /\blinkGhost\b/,
  /\bmemberBadge\b/,
  /\bmentionToken\b/,
  /\bserverTag\b/,
  /\bserverMenuButton\b/,
  /\bbuttonGhost\b/,
  /\.memberMetaStack\s+span/,
  /\.usersStats\s+span/,
  /\.voiceParticipant\s+span/,
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

function auditRawButtons(fullPath, source) {
  const normalizedPath = normalizePath(fullPath);
  if (
    normalizedPath.startsWith("components/ui/") ||
    normalizedPath.includes(".test.") ||
    allowedRawButtonFiles.has(normalizedPath)
  ) {
    return;
  }

  if (source.includes("<button")) {
    failures.push(`${normalizedPath}: raw button must use shared Button, IconButton, PressableButton, MenuItem, or ListActionButton`);
  }
}

function auditPrimitiveOverrides(fullPath, source) {
  const normalizedPath = normalizePath(fullPath);
  if (normalizedPath.startsWith("components/ui/") || normalizedPath.includes(".test.")) {
    return;
  }

  if (primitiveComponentPattern.test(source)) {
    failures.push(`${normalizedPath}: shared UI primitives must use props instead of route-local className overrides`);
  }

  if (primitiveIconClassPattern.test(source) || iconButtonChildClassPattern.test(source)) {
    failures.push(`${normalizedPath}: shared primitive icon slots must not receive locally sized icon classes`);
  }
}

for (const root of roots) {
  walk(root, (fullPath) => {
    const source = readFileSync(fullPath, "utf8");

    auditPressedButtons(fullPath, source);
    auditControlCopies(fullPath, source);
    auditRawButtons(fullPath, source);
    auditPrimitiveOverrides(fullPath, source);
  });
}

if (failures.length > 0) {
  process.stderr.write(`UI framework audit failed: repeated controls must use shared APIs.\n${failures.join("\n")}\n`);
  process.exit(1);
}

process.stdout.write("UI framework audit passed\n");
