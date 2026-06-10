import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const profileMenuCss = readFileSync(join(__dirname, "menu.module.css"), "utf8");
const profileCardCss = readFileSync(join(__dirname, "card.module.css"), "utf8");
const listCss = readFileSync(join(__dirname, "../ui/list/styles.module.css"), "utf8");
const popupCss = readFileSync(join(__dirname, "../ui/popup/styles.module.css"), "utf8");
const sharedMenuCss = readFileSync(join(__dirname, "../ui/menu/styles.module.css"), "utf8");
const layoutCss = readFileSync(join(__dirname, "../layout/main.module.css"), "utf8");
const settingsCss = readFileSync(join(__dirname, "../../app/settings/styles.module.css"), "utf8");

describe("profile menu layout styles", () => {
  it("keeps list row sizing in the shared list primitive", () => {
    const sharedRowRule = listCss.match(/\.listPrimary,\s*\.listRow\s*\{(?<body>[^}]+)\}/)?.groups?.body ?? "";
    const listRule = listCss.match(/\.list \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

    expect(sharedRowRule).toContain("height: var(--list-row-height);");
    expect(sharedRowRule).toContain("align-items: center;");
    expect(sharedRowRule).toContain("padding: var(--space-0) var(--list-row-padding-inline);");
    expect(listRule).not.toContain("padding-block");
    expect(listCss).not.toContain(".listPaddingSm");
    expect(listCss).not.toContain(".listWidthLg");
    expect(sharedMenuCss).not.toContain(".listPrimary");
    expect(sharedMenuCss).not.toContain(".listItem");
    expect(sharedMenuCss).not.toContain(".listRow");
    expect(sharedMenuCss).not.toContain(".listIcon");
    expect(profileMenuCss).not.toContain(".listPrimary");
    expect(profileMenuCss).not.toContain(".layoutItem");
    expect(profileMenuCss).not.toContain(".listIcon");
  });

  it("keeps sidebar menu skin as a cosmetic shared exception", () => {
    const sidebarSkinRule =
      sharedMenuCss.match(
        /\.menu\[data-menu-skin="sidebar"\] \[data-list-item="true"\],\s*\.menu\[data-menu-skin="sidebar"\] \[data-list-row="true"\]\s*\{(?<body>[^}]+)\}/,
      )
        ?.groups?.body ?? "";
    const menuPrimaryRule =
      sharedMenuCss.match(/\.menu \[data-list-primary="true"\],\s*\.menu \[data-list-row="true"\]\s*\{(?<body>[^}]+)\}/)
        ?.groups?.body ?? "";
    const menuRowRule =
      sharedMenuCss.match(/\.menu \[data-list-item="true"\],\s*\.menu \[data-list-row="true"\]\s*\{(?<body>[^}]+)\}/)
        ?.groups?.body ?? "";

    expect(sharedMenuCss).toContain(".menu[data-list-panel=\"true\"][data-menu-skin=\"sidebar\"]");
    expect(sharedMenuCss).toContain(".menu[data-menu-idle-border=\"hidden\"] [data-list-item=\"true\"]");
    expect(sharedMenuCss).toContain(".menu[data-menu-skin=\"sidebar\"] [data-list-item=\"true\"]");
    expect(sharedMenuCss).toContain("[data-list-icon-color=\"accent\"]");
    expect(listCss).toContain(".listIconAccent");
    expect(menuRowRule).toContain("background: transparent");
    expect(sharedMenuCss).toContain("background: var(--color-surface-selected);");
    expect(sidebarSkinRule).not.toContain("border-color: transparent");
    expect(menuPrimaryRule).not.toContain("padding:");
    expect(sharedMenuCss).not.toContain("data-menu-variant");
    expect(sharedMenuCss).not.toContain("data-menu-surface");
    expect(sharedMenuCss).not.toContain("arrow-rail");
    expect(layoutCss).not.toContain(".sidebar .topNav");
    expect(layoutCss).not.toContain(".sidebar .navLinkActive");
  });

  it("keeps popup placement in the shared popup primitive", () => {
    expect(popupCss).toContain(".popup[data-placement=\"right-end\"]");
    expect(popupCss).toContain(".popup[data-placement=\"right-center\"]");
    expect(popupCss).toContain("left: calc(100% + var(--space-4));");
    expect(popupCss).toContain("transform: translateY(-50%);");
    expect(sharedMenuCss).not.toContain(".menu[data-placement=");
    expect(profileMenuCss).not.toContain("data-placement");
    expect(profileMenuCss).not.toContain("position: absolute");
    expect(profileMenuCss).not.toContain("bottom: calc(100%");
  });

  it("renders navigation choices as one inline button group instead of a second row", () => {
    const layoutChoicesRule = profileMenuCss.match(/\.layoutChoices \{(?<body>[^}]+)\}/)?.groups?.body ?? "";

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
