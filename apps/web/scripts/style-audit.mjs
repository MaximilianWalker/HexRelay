import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const colorRoots = ["app", "components"];
const rawColorAllowedFiles = new Set(["app/styles/tokens.css", "app/styles/themes.css"]);
const frameworkRoots = ["components"];
const sharedUiStylesPath = "components/ui/control.module.css";
const sharedActionControlBaseExpectations = [
  {
    selector: ".button",
    properties: {
      "height": "var(--size-control-md)",
      "min-height": "var(--size-control-md)",
    },
  },
  {
    selector: ".buttonSm",
    properties: {
      "height": "var(--size-control-sm)",
      "min-height": "var(--size-control-sm)",
      "font-size": "var(--text-sm)",
    },
  },
  {
    selector: ".button > svg",
    properties: {
      "display": "block",
      "width": "var(--size-icon-sm)",
      "height": "var(--size-icon-sm)",
      "flex": "0 0 var(--size-icon-sm)",
      "stroke-width": "1.8",
    },
  },
  {
    selector: ".segmentedButton",
    properties: {
      "height": "var(--size-control-md)",
      "min-height": "var(--size-control-md)",
    },
  },
  {
    selector: ".segmentedButton > svg",
    properties: {
      "display": "block",
      "width": "var(--size-icon-sm)",
      "height": "var(--size-icon-sm)",
      "flex": "0 0 var(--size-icon-sm)",
      "stroke-width": "1.8",
    },
  },
];
const sharedActionControlTypographyExpectations = [
  {
    selector: ".button",
    properties: {
      "font-family": "inherit",
      "font-size": "var(--text-md)",
      "font-weight": "var(--weight-medium)",
      "line-height": "var(--line-tight)",
      "white-space": "nowrap",
    },
  },
  {
    selector: ".segmentedButton",
    properties: {
      "font-family": "inherit",
      "font-size": "var(--text-md)",
      "font-weight": "var(--weight-medium)",
      "line-height": "var(--line-tight)",
      "white-space": "nowrap",
    },
  },
];
const activeControlExpectations = [
  {
    selector: ".buttonPressed",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
  {
    selector: ".buttonPressed:hover",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
  {
    selector: ".buttonPressed:focus-visible",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
  {
    selector: ".segmentedButton.segmentedButtonActive",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
];
const forbiddenSharedControlTypographyOverrides = [
  ".buttonPrimary",
  ".buttonSecondary",
  ".buttonGhost",
  ".buttonDanger",
  ".buttonPressed",
  ".segmentedButton.segmentedButtonActive",
].flatMap((selector) =>
  ["font-family", "font-size", "font-weight", "line-height"].map((property) => ({
    property,
    selector,
  })),
);

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

function parseCssDeclarations(block) {
  const declarations = new Map();

  block
    .split(";")
    .map((part) => part.trim())
    .filter(Boolean)
    .forEach((declaration) => {
      const separator = declaration.indexOf(":");
      if (separator === -1) {
        return;
      }

      declarations.set(declaration.slice(0, separator).trim(), declaration.slice(separator + 1).trim());
    });

  return declarations;
}

function findSelectorDeclarations(css, selector) {
  const blockPattern = /([^{}]+)\{([^{}]*)\}/g;
  const declarations = new Map();
  let match;

  while ((match = blockPattern.exec(css)) !== null) {
    const selectors = match[1]
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);

    if (selectors.includes(selector)) {
      parseCssDeclarations(match[2]).forEach((value, property) => {
        declarations.set(property, value);
      });
    }
  }

  return declarations.size > 0 ? declarations : null;
}

function auditSharedActiveControlTokens() {
  const css = readFileSync(sharedUiStylesPath, "utf8");

  [
    ...sharedActionControlBaseExpectations,
    ...sharedActionControlTypographyExpectations,
    ...activeControlExpectations,
  ].forEach(({ properties, selector }) => {
    const declarations = findSelectorDeclarations(css, selector);
    if (!declarations) {
      failures.push(`${sharedUiStylesPath}: missing shared control selector: ${selector}`);
      return;
    }

    Object.entries(properties).forEach(([property, expectedValue]) => {
      const actualValue = declarations.get(property);
      if (actualValue !== expectedValue) {
        failures.push(
          `${sharedUiStylesPath}: ${selector} must set ${property}: ${expectedValue}; found ${actualValue ?? "missing"}`,
        );
      }
    });
  });

  forbiddenSharedControlTypographyOverrides.forEach(({ property, selector }) => {
    const declarations = findSelectorDeclarations(css, selector);
    if (declarations?.has(property)) {
      failures.push(`${sharedUiStylesPath}: ${selector} must inherit ${property} from the shared control base`);
    }
  });
}

colorRoots.forEach((root) => walk(root, auditRawColors));
frameworkRoots.forEach((root) => walk(root, auditFrameworkSpacing));
auditSharedActiveControlTokens();

if (failures.length > 0) {
  process.stderr.write(
    `Style audit failed: route/component CSS must use semantic colors, and framework CSS must use spacing/radius tokens.\n${failures.join("\n")}\n`,
  );
  process.exit(1);
}

process.stdout.write("Style audit passed\n");
