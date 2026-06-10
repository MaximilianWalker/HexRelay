import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const colorRoots = ["app", "components"];
const rawColorAllowedFiles = new Set(["app/styles/tokens.css", "app/styles/themes.css"]);
const frameworkRoots = ["components"];
const sharedButtonStylesPath = "components/ui/button/styles.module.css";
const sharedToggleGroupStylesPath = "components/ui/toggle-group/styles.module.css";
const sharedListStylesPath = "components/ui/list/styles.module.css";
const sharedMenuStylesPath = "components/ui/menu/styles.module.css";
const sharedActionControlBaseExpectations = [
  {
    stylesPath: sharedButtonStylesPath,
    selector: ".button",
    properties: {
      "height": "var(--size-control-md)",
      "min-height": "var(--size-control-md)",
    },
  },
  {
    stylesPath: sharedButtonStylesPath,
    selector: ".buttonSm",
    properties: {
      "height": "var(--size-control-sm)",
      "min-height": "var(--size-control-sm)",
      "font-size": "var(--text-sm)",
    },
  },
  {
    stylesPath: sharedButtonStylesPath,
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
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroup",
    properties: {
      "--button-group-height": "var(--size-control-md)",
      "--button-group-font-size": "var(--text-md)",
      "--button-group-icon-size": "var(--size-icon-sm)",
      "height": "var(--button-group-height)",
      "min-height": "var(--button-group-height)",
    },
  },
  {
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroupSm",
    properties: {
      "--button-group-height": "var(--size-control-sm)",
      "--button-group-font-size": "var(--text-sm)",
    },
  },
  {
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroupLg",
    properties: {
      "--button-group-height": "var(--size-control-lg)",
      "--button-group-font-size": "var(--text-body)",
      "--button-group-icon-size": "var(--size-icon-md)",
    },
  },
  {
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroupButton",
    properties: {
      "height": "100%",
      "min-height": "0",
    },
  },
  {
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroupButton > svg",
    properties: {
      "display": "block",
      "width": "var(--button-group-icon-size)",
      "height": "var(--button-group-icon-size)",
      "flex": "0 0 var(--button-group-icon-size)",
      "stroke-width": "1.8",
    },
  },
  {
    stylesPath: sharedListStylesPath,
    selector: ".listItem",
    properties: {
      "display": "grid",
      "min-height": "var(--list-row-height)",
    },
  },
  {
    stylesPath: sharedListStylesPath,
    selector: ".listPrimary",
    properties: {
      "height": "var(--list-row-height)",
      "min-height": "var(--list-row-height)",
    },
  },
  {
    stylesPath: sharedListStylesPath,
    selector: ".list[data-list-panel=\"false\"]",
    properties: {
      "border": "0",
      "background": "transparent",
      "box-shadow": "none",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu",
    properties: {
      "--menu-spacing": "var(--gap-list)",
      "gap": "var(--menu-spacing)",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu[data-list-panel=\"true\"]",
    properties: {
      "padding": "var(--menu-spacing)",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu[data-menu-spacing=\"sm\"]",
    properties: {
      "--menu-spacing": "var(--space-2)",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu[data-list-panel=\"true\"][data-menu-skin=\"sidebar\"]",
    properties: {
      "border-radius": "var(--radius-xl)",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu[data-menu-idle-border=\"hidden\"] [data-list-item=\"true\"]",
    properties: {
      "border-color": "transparent",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu[data-menu-skin=\"sidebar\"] [data-list-item=\"true\"]",
    properties: {
      "background": "transparent",
    },
  },
  {
    stylesPath: sharedMenuStylesPath,
    selector: ".menu[data-menu-skin=\"sidebar\"] [data-list-icon-color=\"accent\"]",
    properties: {
      "stroke-width": "1.9",
    },
  },
  {
    stylesPath: sharedListStylesPath,
    selector: ".listIconAccent",
    properties: {
      "color": "var(--color-accent-strong)",
    },
  },
];
const sharedActionControlTypographyExpectations = [
  {
    stylesPath: sharedButtonStylesPath,
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
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroupButton",
    properties: {
      "font-family": "inherit",
      "font-size": "var(--button-group-font-size)",
      "font-weight": "var(--weight-medium)",
      "line-height": "var(--line-tight)",
      "white-space": "nowrap",
    },
  },
  {
    stylesPath: sharedListStylesPath,
    selector: ".listPrimary",
    properties: {
      "font-family": "inherit",
      "font-size": "inherit",
      "line-height": "inherit",
    },
  },
];
const activeControlExpectations = [
  {
    stylesPath: sharedButtonStylesPath,
    selector: ".buttonPressed",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
  {
    stylesPath: sharedButtonStylesPath,
    selector: ".buttonPressed:hover",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
  {
    stylesPath: sharedButtonStylesPath,
    selector: ".buttonPressed:focus-visible",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
  {
    stylesPath: sharedToggleGroupStylesPath,
    selector: ".buttonGroupButton.buttonGroupButtonActive",
    properties: {
      "background": "var(--color-accent-strong)",
      "border-color": "var(--color-accent-strong)",
      "color": "var(--color-text-inverse)",
    },
  },
];
const forbiddenSharedControlTypographyOverrides = [
  { selector: ".buttonPrimary", stylesPath: sharedButtonStylesPath },
  { selector: ".buttonSecondary", stylesPath: sharedButtonStylesPath },
  { selector: ".buttonGhost", stylesPath: sharedButtonStylesPath },
  { selector: ".buttonDanger", stylesPath: sharedButtonStylesPath },
  { selector: ".buttonPressed", stylesPath: sharedButtonStylesPath },
  { selector: ".buttonGroupButton.buttonGroupButtonActive", stylesPath: sharedToggleGroupStylesPath },
].flatMap(({ selector, stylesPath }) =>
  ["font-family", "font-size", "font-weight", "line-height"].map((property) => ({
    property,
    selector,
    stylesPath,
  })),
);

const rawColorPattern = /#[0-9a-fA-F]{3,8}\b|rgba?\(|hsla?\(/;
const rawSpacingPattern =
  /^\s*(?:gap|row-gap|column-gap|padding(?:-(?:top|right|bottom|left))?|margin(?:-(?:top|right|bottom|left))?|border-radius)\s*:[^;]*\d+(?:\.\d+)?px\b/;
const failures = [];
const cssCache = new Map();

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

function readCss(stylesPath) {
  if (!cssCache.has(stylesPath)) {
    cssCache.set(stylesPath, readFileSync(stylesPath, "utf8"));
  }

  return cssCache.get(stylesPath);
}

function auditSharedActiveControlTokens() {
  [
    ...sharedActionControlBaseExpectations,
    ...sharedActionControlTypographyExpectations,
    ...activeControlExpectations,
  ].forEach(({ properties, selector, stylesPath }) => {
    const css = readCss(stylesPath);
    const declarations = findSelectorDeclarations(css, selector);
    if (!declarations) {
      failures.push(`${stylesPath}: missing shared control selector: ${selector}`);
      return;
    }

    Object.entries(properties).forEach(([property, expectedValue]) => {
      const actualValue = declarations.get(property);
      if (actualValue !== expectedValue) {
        failures.push(
          `${stylesPath}: ${selector} must set ${property}: ${expectedValue}; found ${actualValue ?? "missing"}`,
        );
      }
    });
  });

  forbiddenSharedControlTypographyOverrides.forEach(({ property, selector, stylesPath }) => {
    const css = readCss(stylesPath);
    const declarations = findSelectorDeclarations(css, selector);
    if (declarations?.has(property)) {
      failures.push(`${stylesPath}: ${selector} must inherit ${property} from the shared control base`);
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
