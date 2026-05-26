import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const colorRoots = ["app", "components"];
const rawColorAllowedFiles = new Set(["app/styles/tokens.css", "app/styles/themes.css"]);
const frameworkRoots = [
  "components/ui",
  "components/hubs",
  "components/chat",
  "components/settings",
  "components/onboarding",
];

const rawColorPattern = /#[0-9a-fA-F]{3,8}\b|rgba?\(|hsla?\(/;
const rawSpacingPattern =
  /^\s*(?:gap|row-gap|column-gap|padding(?:-(?:top|right|bottom|left))?|margin(?:-(?:top|right|bottom|left))?|border-radius)\s*:[^;]*\d+(?:\.\d+)?px\b/;
const failures = [];

function normalizePath(path) {
  return path.replaceAll("\\", "/");
}

function walk(directory, visitFile) {
  for (const entry of readdirSync(directory)) {
    const fullPath = join(directory, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) {
      walk(fullPath, visitFile);
      continue;
    }

    if (!fullPath.endsWith(".css")) {
      continue;
    }

    visitFile(fullPath);
  }
}

function auditRawColors(fullPath) {
  const normalizedPath = normalizePath(fullPath);
  if (rawColorAllowedFiles.has(normalizedPath)) {
    return;
  }

  const lines = readFileSync(fullPath, "utf8").split(/\r?\n/);
  lines.forEach((line, index) => {
    if (rawColorPattern.test(line)) {
      failures.push(`${fullPath}:${index + 1}: raw color: ${line.trim()}`);
    }
  });
}

function auditFrameworkSpacing(fullPath) {
    const lines = readFileSync(fullPath, "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
      if (rawSpacingPattern.test(line)) {
        failures.push(`${fullPath}:${index + 1}: raw spacing/radius: ${line.trim()}`);
      }
    });
}

colorRoots.forEach((root) => walk(root, auditRawColors));
frameworkRoots.forEach((root) => walk(root, auditFrameworkSpacing));

if (failures.length > 0) {
  process.stderr.write(
    `Style audit failed: route/component CSS must use semantic colors, and framework CSS must use spacing/radius tokens.\n${failures.join("\n")}\n`,
  );
  process.exit(1);
}

process.stdout.write("Style audit passed\n");
