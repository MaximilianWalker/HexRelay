import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const menuCss = readFileSync(join(__dirname, "menu.module.css"), "utf8");
const profileCardCss = readFileSync(join(__dirname, "card.module.css"), "utf8");
const controlCss = readFileSync(join(__dirname, "../ui/control.module.css"), "utf8");
const layoutCss = readFileSync(join(__dirname, "../layout/main.module.css"), "utf8");
const settingsCss = readFileSync(join(__dirname, "../../app/settings/styles.module.css"), "utf8");

describe("profile menu layout styles", () => {
  it("keeps menu row sizing in the shared menu primitive", () => {
    const sharedRowRule = controlCss.match(/\.menuItem,\s*\.menuRow\s*\{(?<body>[^}]+)\}/)?.groups?.body ?? "";
    const menuRule = controlCss.match(/\.menu \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(sharedRowRule).toContain("height: var(--menu-item-height);");
    expect(sharedRowRule).toContain("align-items: center;");
    expect(sharedRowRule).toContain("padding: var(--space-0) var(--menu-item-padding-inline);");
    expect(menuRule).not.toContain("padding-block");
    expect(controlCss).not.toContain(".menuPaddingSm");
    expect(controlCss).not.toContain(".menuWidthLg");
    expect(menuCss).not.toContain(".menuItem");
    expect(menuCss).not.toContain(".layoutItem");
    expect(menuCss).not.toContain(".menuIcon");
  });

  it("keeps popup placement in the shared popup primitive", () => {
    expect(controlCss).toContain(".popup[data-placement=\"right-end\"]");
    expect(controlCss).toContain(".popup[data-placement=\"right-center\"]");
    expect(controlCss).toContain("left: calc(100% + var(--space-4));");
    expect(controlCss).toContain("transform: translateY(-50%);");
    expect(controlCss).not.toContain(".menu[data-placement=");
    expect(menuCss).not.toContain("data-placement");
    expect(menuCss).not.toContain("position: absolute");
    expect(menuCss).not.toContain("bottom: calc(100%");
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
    const topbarBodyRule = layoutCss.match(/\.topbarMode \.body \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(topbarBodyRule).toContain("scrollbar-gutter: stable both-edges;");
  });
});

describe("topbar profile controls responsive styles", () => {
  it("collapses optional profile text on narrow topbar layouts to keep menus inside the viewport", () => {
    expect(profileCardCss).toContain(":global([data-profile-placement=\"topbar\"]) .details");
    expect(profileCardCss).toContain("display: none;");
  });
});
