import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const menuCss = readFileSync(join(__dirname, "workspace-profile-menu.module.css"), "utf8");
const profileCardCss = readFileSync(join(__dirname, "workspace-profile-card.module.css"), "utf8");
const shellCss = readFileSync(join(__dirname, "workspace-shell.module.css"), "utf8");
const settingsCss = readFileSync(join(__dirname, "../app/settings/styles.module.css"), "utf8");

describe("workspace profile menu layout styles", () => {
  it("keeps every menu row on the same fixed block size", () => {
    const sharedRowRule = menuCss.match(/\.menuItem\s*,\s*\.layoutItem\s*\{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(sharedRowRule).toContain("height: var(--space-20);");
    expect(sharedRowRule).not.toContain("min-height");
  });

  it("renders navigation choices as one inline button group instead of a second row", () => {
    const layoutChoicesRule = menuCss.match(/\.layoutChoices \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(layoutChoicesRule).toContain("display: inline-flex;");
    expect(layoutChoicesRule).not.toContain("grid-column");
    expect(layoutChoicesRule).not.toContain("margin-top");
  });
});

describe("settings page layout styles", () => {
  it("centers the settings content when the page reaches its max width", () => {
    const pageRule = settingsCss.match(/\.page \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(pageRule).toContain("max-width: 1100px;");
    expect(pageRule).toContain("justify-self: center;");
  });

  it("balances scrollbar gutters in topbar content so centered settings stay visually aligned", () => {
    const topbarBodyRule = shellCss.match(/\.topbarMode \.body \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(topbarBodyRule).toContain("scrollbar-gutter: stable both-edges;");
  });
});

describe("topbar profile controls responsive styles", () => {
  it("collapses optional profile text on narrow topbar layouts to keep menus inside the viewport", () => {
    expect(profileCardCss).toContain(":global([data-profile-placement=\"topbar\"]) .details");
    expect(profileCardCss).toContain("display: none;");
  });
});
