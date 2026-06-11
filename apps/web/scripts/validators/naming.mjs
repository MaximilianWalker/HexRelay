import { readdirSync, readFileSync, statSync } from "node:fs";
import { extname, join, relative } from "node:path";

const roots = ["app", "components", "lib", "scripts"];
const sourceExtensions = new Set([".css", ".mjs", ".ts", ".tsx"]);
const ignoredDirectories = new Set([
  ".next",
  "coverage",
  "node_modules",
]);
const ignoredParentNames = new Set([
  "accessibility",
  "app",
  "buttons",
  "components",
  "display",
  "feedback",
  "forms",
  "lib",
  "navigation",
  "overlays",
  "scripts",
  "styles",
  "surfaces",
  "toggles",
]);
const ignoredFileNames = new Set([
  "page.tsx",
  "layout.tsx",
  "loading.tsx",
  "not-found.tsx",
  "route.ts",
  "styles.module.css",
  "types.ts",
]);

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

    if (sourceExtensions.has(extname(entry))) {
      visitFile(fullPath);
    }
  }
}

function singularize(value) {
  if (value.endsWith("ies")) {
    return `${value.slice(0, -3)}y`;
  }

  if (value.endsWith("s")) {
    return value.slice(0, -1);
  }

  return value;
}

function contextTokens(parentName) {
  const normalized = parentName.replace(/^\[(.+)\]$/, "$1").toLowerCase();
  if (!normalized || ignoredParentNames.has(normalized) || parentName.startsWith("[") || parentName.startsWith("(")) {
    return [];
  }

  return [...new Set([normalized, singularize(normalized), ...normalized.split(/[-_]/), ...normalized.split(/[-_]/).map(singularize)])]
    .filter((token) => token.length > 2 && !ignoredParentNames.has(token));
}

function stem(fileName) {
  return fileName
    .replace(/\.module\.css$/, "")
    .replace(/\.(mjs|tsx?|css)$/, "")
    .toLowerCase();
}

function startsWithContext(name, token) {
  return name === token || name.startsWith(`${token}-`) || name.startsWith(`${token}_`);
}

function pascalToken(token) {
  return token
    .split(/[-_]/)
    .filter(Boolean)
    .map((part) => `${part[0]?.toUpperCase() ?? ""}${part.slice(1)}`)
    .join("");
}

function pathContextTokens(fullPath) {
  const parts = normalizePath(fullPath).split("/");
  const fileName = parts.at(-1) ?? "";
  const directories = parts.slice(0, -1);
  const tokens = directories.flatMap((directory) => contextTokens(directory));

  return {
    fileName,
    tokens: [...new Set(tokens)],
  };
}

function auditFileName(fullPath) {
  const { fileName, tokens } = pathContextTokens(fullPath);

  if (ignoredFileNames.has(fileName)) {
    return;
  }

  const fileStem = stem(fileName);
  for (const token of tokens) {
    if (startsWithContext(fileStem, token)) {
      failures.push(
        `${normalizePath(fullPath)}: redundant filename context "${token}" already provided by path context`,
      );
      return;
    }
  }
}

function auditExports(fullPath) {
  const { tokens } = pathContextTokens(fullPath);
  if (tokens.length === 0) {
    return;
  }

  const source = readFileSync(fullPath, "utf8");
  const exportPattern = /\bexport\s+(?:type\s+|interface\s+|function\s+|const\s+|class\s+)([A-Z][A-Za-z0-9_]*)/g;
  let match;

  while ((match = exportPattern.exec(source)) !== null) {
    const exportName = match[1];
    for (const token of tokens) {
      const prefix = pascalToken(token);
      if (prefix && exportName !== prefix && exportName.startsWith(prefix)) {
        failures.push(
          `${normalizePath(fullPath)}: export "${exportName}" repeats path context "${token}"`,
        );
        return;
      }
    }
  }
}

for (const root of roots) {
  walk(root, (fullPath) => {
    const localPath = normalizePath(relative(process.cwd(), fullPath));
    if (localPath.includes("/node_modules/") || localPath.includes("/.next/")) {
      return;
    }

    auditFileName(fullPath);
    auditExports(fullPath);
  });
}

if (failures.length > 0) {
  process.stderr.write(`Naming audit failed: avoid repeating directory or module context in local names.\n${failures.join("\n")}\n`);
  process.exit(1);
}

process.stdout.write("Naming audit passed\n");
