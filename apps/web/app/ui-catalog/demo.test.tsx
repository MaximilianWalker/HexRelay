// @vitest-environment jsdom

import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { THEME_STORAGE_KEY } from "@/lib/ui/theme";

import { Demo } from "./demo";

afterEach(() => {
  cleanup();
  document.documentElement.removeAttribute("data-theme");
  window.localStorage.clear();
  window.history.replaceState(null, "", "/");
});

function section(id: string) {
  const element = document.getElementById(id);

  expect(element).toBeInTheDocument();

  return within(element as HTMLElement);
}

describe("UI catalog", () => {
  it("opens and closes the responsive catalog navigation", async () => {
    const user = userEvent.setup();

    render(<Demo />);

    const navButton = screen.getByRole("button", { name: "Catalog" });
    const header = document.querySelector("header") as HTMLElement;
    const brandName = within(header).getByText("HexRelay");
    const title = screen.getByRole("heading", { name: "UI Catalog" });

    expect(screen.queryByRole("dialog", { name: "Catalog navigation" })).not.toBeInTheDocument();
    expect(
      Boolean(
        navButton.compareDocumentPosition(brandName) & Node.DOCUMENT_POSITION_FOLLOWING,
      ),
    ).toBe(true);
    expect(
      Boolean(
        brandName.compareDocumentPosition(title) & Node.DOCUMENT_POSITION_FOLLOWING,
      ),
    ).toBe(true);
    expect(screen.queryByText("Development catalog")).not.toBeInTheDocument();
    expect(
      screen.queryByText("Shared primitives, states, tones, and composed patterns used by app surfaces."),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Shared APIs").className).toContain("badgeLg");
    expect(screen.getByRole("combobox", { name: "Theme" })).toHaveValue("system");
    expect(screen.getByRole("searchbox", { name: "Search components" })).toBeInTheDocument();

    await user.click(navButton);

    const dialog = screen.getByRole("dialog", { name: "Catalog navigation" });
    expect(within(dialog).getByRole("button", { name: "Identity" })).toHaveAttribute("aria-expanded", "true");
    expect(within(dialog).getByRole("link", { name: "Logo" })).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "Inputs & Controls" })).toHaveAttribute("aria-expanded", "false");
    expect(within(dialog).queryByRole("link", { name: "Buttons" })).not.toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Inputs & Controls" }));
    expect(within(dialog).getByRole("link", { name: "Buttons" })).toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Navigation & Actions" }));
    await user.click(within(dialog).getByRole("link", { name: "List" }));
    expect(screen.queryByRole("dialog", { name: "Catalog navigation" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Catalog" }));
    expect(screen.getByRole("dialog", { name: "Catalog navigation" })).toBeInTheDocument();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Catalog navigation" })).not.toBeInTheDocument();
  });

  it("changes the catalog theme through the shared theme preference", async () => {
    const user = userEvent.setup();

    render(<Demo />);
    const themeSelect = screen.getByRole("combobox", { name: "Theme" });

    await user.selectOptions(themeSelect, "light");

    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("light");
    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    expect(themeSelect).toHaveValue("light");

    await user.selectOptions(themeSelect, "dark");

    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(themeSelect).toHaveValue("dark");

    await user.selectOptions(themeSelect, "system");

    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("system");
    expect(document.documentElement).toHaveAttribute("data-theme", "system");
    expect(themeSelect).toHaveValue("system");
  });

  it("filters catalog sections from the command bar search", async () => {
    const user = userEvent.setup();

    render(<Demo />);
    const sidebar = screen.getByRole("complementary", { name: "UI catalog sections" });

    await user.type(screen.getByRole("searchbox", { name: "Search components" }), "badges");

    expect(document.getElementById("buttons")).not.toBeInTheDocument();
    expect(document.getElementById("badges")).toBeInTheDocument();
    expect(within(sidebar).getByRole("button", { name: "Data Display" })).toHaveAttribute("aria-expanded", "true");
    expect(within(sidebar).getByRole("link", { name: "Badges" })).toBeInTheDocument();
    expect(within(sidebar).queryByRole("button", { name: "Inputs & Controls" })).not.toBeInTheDocument();
    expect(within(sidebar).queryByRole("link", { name: "Buttons" })).not.toBeInTheDocument();

    await user.clear(screen.getByRole("searchbox", { name: "Search components" }));

    expect(document.getElementById("buttons")).toBeInTheDocument();
    expect(within(sidebar).getByRole("button", { name: "Inputs & Controls" })).toHaveAttribute("aria-expanded", "false");
    expect(within(sidebar).queryByRole("link", { name: "Buttons" })).not.toBeInTheDocument();

    await user.click(within(sidebar).getByRole("button", { name: "Inputs & Controls" }));

    expect(within(sidebar).getByRole("link", { name: "Buttons" })).toBeInTheDocument();
  });

  it("toggles sidebar category dropdowns", async () => {
    const user = userEvent.setup();

    render(<Demo />);
    const sidebar = screen.getByRole("complementary", { name: "UI catalog sections" });
    const identity = within(sidebar).getByRole("button", { name: "Identity" });
    const inputs = within(sidebar).getByRole("button", { name: "Inputs & Controls" });

    expect(identity).toHaveAttribute("aria-expanded", "true");
    expect(within(sidebar).getByRole("link", { name: "Logo" })).toBeInTheDocument();
    expect(inputs).toHaveAttribute("aria-expanded", "false");
    expect(within(sidebar).queryByRole("link", { name: "Buttons" })).not.toBeInTheDocument();

    await user.click(inputs);

    expect(inputs).toHaveAttribute("aria-expanded", "true");
    expect(within(sidebar).getByRole("link", { name: "Buttons" })).toBeInTheDocument();

    await user.click(inputs);

    expect(inputs).toHaveAttribute("aria-expanded", "false");
    expect(within(sidebar).queryByRole("link", { name: "Buttons" })).not.toBeInTheDocument();
  });

  it("uses custom catalog scrollbar chrome instead of the native rail", () => {
    render(<Demo />);

    const catalogScrollArea = screen.getAllByTestId("scroll-area")[0];
    const catalogViewport = screen.getAllByTestId("scroll-area-viewport")[0];

    expect(catalogScrollArea).toHaveAttribute("data-overlay", "true");
    expect(catalogScrollArea).toHaveStyle({ "--scroll-area-thumb-width": "4px" });
    expect(catalogViewport.className).toContain("content");
  });

  it("shows an empty state when no catalog sections match search", async () => {
    const user = userEvent.setup();

    render(<Demo />);
    const sidebar = screen.getByRole("complementary", { name: "UI catalog sections" });

    await user.type(screen.getByRole("searchbox", { name: "Search components" }), "not-a-component");

    expect(document.getElementById("buttons")).not.toBeInTheDocument();
    expect(document.getElementById("logo")).not.toBeInTheDocument();
    expect(within(sidebar).getByText("No matching components")).toBeInTheDocument();
    expect(screen.getByText("No components found")).toBeInTheDocument();
  });

  it("marks the current hash section as active in the sidebar", async () => {
    window.history.replaceState(null, "", "/ui-catalog#list");

    render(<Demo />);
    const sidebar = screen.getByRole("complementary", { name: "UI catalog sections" });

    await waitFor(() => {
      expect(within(sidebar).getByRole("button", { name: "Navigation & Actions" })).toHaveAttribute(
        "aria-expanded",
        "true",
      );
      expect(within(sidebar).getByRole("link", { name: "List" })).toHaveAttribute("aria-current", "page");
      expect(document.querySelectorAll('[aria-current="page"]')).toHaveLength(1);
    });
  });

  it("documents the current shared control options", async () => {
    const user = userEvent.setup();

    render(<Demo />);

    const logo = section("logo");
    expect(logo.getByText("Logo Mark")).toBeInTheDocument();
    expect(logo.getByText("Logo Lockup")).toBeInTheDocument();
    expect(logo.getAllByText("HexRelay")).toHaveLength(3);
    expect(logo.getAllByRole("img")).toHaveLength(3);

    const buttons = section("buttons");
    expect(buttons.getByText("Tones")).toBeInTheDocument();
    expect(buttons.queryByText("Icon Sizes")).not.toBeInTheDocument();
    expect(buttons.getByRole("button", { name: "Pressed" })).toHaveAttribute("aria-pressed", "true");
    expect(buttons.getByRole("button", { name: "Loading" })).toHaveAttribute("aria-busy", "true");
    expect(buttons.queryByRole("button", { name: "Danger active" })).not.toBeInTheDocument();
    expect(buttons.getByRole("link", { name: "Link" })).toHaveAttribute("href", "#buttons");

    const toggles = section("toggles");
    expect(screen.queryByText("Button Group Sizes")).not.toBeInTheDocument();
    expect(toggles.queryByText("Toggle group")).not.toBeInTheDocument();
    expect(toggles.queryByText("Button", { exact: true })).not.toBeInTheDocument();
    expect(document.querySelector('#toggles [class*="sizeTable"]')).not.toBeInTheDocument();
    expect(document.querySelector('#toggles [class*="controlSizeList"]')).toBeInTheDocument();
    expect(toggles.getByRole("group", { name: "Small view mode" }).className).toContain("toggleGroupSm");
    expect(toggles.getByRole("group", { name: "Medium view mode" }).className).not.toContain("toggleGroupSm");
    expect(toggles.getByRole("group", { name: "Medium view mode" }).className).not.toContain("toggleGroupLg");
    expect(toggles.getByRole("group", { name: "Large view mode" }).className).toContain("toggleGroupLg");

    const list = section("list");
    expect(list.getByText("Items")).toBeInTheDocument();
    expect(list.getByText("Actions")).toBeInTheDocument();
    expect(list.getByText("Static Content")).toBeInTheDocument();
    expect(list.getByText("Sizes")).toBeInTheDocument();
    expect(list.getByText("Without Panel")).toBeInTheDocument();
    expect(list.getByText("Profile settings")).toBeInTheDocument();
    expect(list.getByText("Mute notifications")).toBeInTheDocument();
    expect(list.getByText("Invite contact")).toBeInTheDocument();
    expect(list.getByText("Leave server")).toBeInTheDocument();
    expect(list.getByText("Pin tab")).toBeInTheDocument();
    expect(list.getByText("Voice unavailable")).toBeInTheDocument();
    expect(list.getByText("Sidebar layout")).toBeInTheDocument();
    expect(list.getByText("Edit preferences")).toBeInTheDocument();
    expect(list.getByText("Plain action")).toBeInTheDocument();
    expect(list.queryByText("Rows")).not.toBeInTheDocument();
    expect(list.queryByText("Padding and Width")).not.toBeInTheDocument();
    expect(list.queryByText("Large width")).not.toBeInTheDocument();

    const menu = section("menu");
    expect(menu.getByText("States")).toBeInTheDocument();
    expect(menu.getByText("Nested")).toBeInTheDocument();
    expect(menu.getByText("Panel Menu")).toBeInTheDocument();
    expect(menu.getByText("Compact Panel")).toBeInTheDocument();
    expect(menu.getByRole("button", { name: "With badge" })).toBeInTheDocument();
    expect(menu.getByRole("button", { name: "Mentions" })).toBeInTheDocument();
    expect(menu.getByRole("button", { name: "Inputs & Controls" })).toHaveAttribute("aria-expanded", "true");
    expect(menu.getByRole("button", { name: "Servers" })).toBeInTheDocument();

    const scrollArea = section("scroll-area");
    expect(scrollArea.getByText("Overlay Scrollbar")).toBeInTheDocument();
    expect(scrollArea.getByText("Reserved Track")).toBeInTheDocument();
    expect(scrollArea.getByText("Scroll Buttons")).toBeInTheDocument();
    expect(scrollArea.getByRole("button", { name: "Previous item" })).toHaveAttribute(
      "data-scroll-button-appearance",
      "framed",
    );
    expect(scrollArea.getByRole("button", { name: "Next item" })).toHaveAttribute(
      "data-scroll-button-appearance",
      "plain",
    );
    expect(scrollArea.getAllByText("Announcements")).toHaveLength(2);
    expect(scrollArea.getAllByTestId("scroll-area")).toHaveLength(2);

    const toolbar = section("toolbar");
    expect(toolbar.getByText("Filters")).toBeInTheDocument();
    expect(toolbar.getByText("Search")).toBeInTheDocument();
    expect(toolbar.getByRole("button", { name: "Invite" })).toBeInTheDocument();
    expect(toolbar.getByRole("button", { name: "Toolbar settings" })).toBeInTheDocument();
    expect(toolbar.getByRole("group", { name: "Toolbar view mode" })).toBeInTheDocument();
    expect(toolbar.getByRole("textbox", { name: "Toolbar search" })).toBeInTheDocument();

    const popups = section("popups");
    const dialogsElement = document.getElementById("dialogs");
    const popupsElement = document.getElementById("popups");
    expect(Boolean(dialogsElement?.compareDocumentPosition(popupsElement as Node) & Node.DOCUMENT_POSITION_FOLLOWING)).toBe(true);
    expect(popups.getByRole("heading", { name: "Popups" })).toBeInTheDocument();
    expect(popups.getByText("Popups own anchored placement, fixed-position menus, and compact floating content.")).toBeInTheDocument();
    expect(popups.getByText("Placement")).toBeInTheDocument();
    expect(popups.queryByText("Playground")).not.toBeInTheDocument();
    expect(popups.getByRole("combobox", { name: "Vertical alignment" })).toBeInTheDocument();
    expect(popups.getByRole("combobox", { name: "Horizontal alignment" })).toBeInTheDocument();
    expect(popups.getByRole("combobox", { name: "Content" })).toBeInTheDocument();
    expect(document.querySelectorAll("#popups [data-placement]")).toHaveLength(1);
    expect(document.getElementById("catalog-popup-demo")).toHaveAttribute("data-placement", "bottom-center");

    await user.click(popups.getByRole("button", { name: "Activity" }));
    expect(document.getElementById("catalog-popup-demo")).not.toBeInTheDocument();

    await user.selectOptions(popups.getByRole("combobox", { name: "Vertical alignment" }), "center");
    expect(document.getElementById("catalog-popup-demo")).toHaveAttribute("data-placement", "center");

    await user.selectOptions(popups.getByRole("combobox", { name: "Horizontal alignment" }), "left");
    expect(document.getElementById("catalog-popup-demo")).toHaveAttribute("data-placement", "left-center");

    await user.selectOptions(popups.getByRole("combobox", { name: "Content" }), "list");
    expect(popups.getByText("Open settings")).toBeInTheDocument();

    const messages = section("messages");
    expect(messages.getByText("Channel Rail")).toBeInTheDocument();
    expect(messages.getByText("Timeline And Composer")).toBeInTheDocument();
    expect(messages.getByText("Load earlier messages")).toBeInTheDocument();
    expect(messages.getByPlaceholderText("Message #general")).toHaveValue("Draft a quick channel update...");
    expect(messages.getByText("Online")).toBeInTheDocument();
    expect(messages.getByText("Away")).toBeInTheDocument();

    const profileControls = section("profile-controls");
    expect(profileControls.getByText("Sidebar Controls")).toBeInTheDocument();
    expect(profileControls.getByText("Aline Costa")).toBeInTheDocument();
    expect(profileControls.getByRole("group", { name: "Profile actions" })).toBeInTheDocument();

    const contentTabs = section("content-tabs");
    expect(contentTabs.getByText("Scrollable Bar")).toBeInTheDocument();
    expect(contentTabs.getByRole("button", { name: "Chat" })).toHaveAttribute("aria-pressed", "true");
    expect(contentTabs.getByRole("button", { name: "New" })).toBeInTheDocument();

    const settingsRows = section("settings-rows");
    expect(settingsRows.getByText("Preference Rows")).toBeInTheDocument();
    expect(settingsRows.getAllByText("Message alerts").length).toBeGreaterThan(0);
    expect(settingsRows.getByText("Interface density")).toBeInTheDocument();
    expect(settingsRows.getByText("Delivery route")).toBeInTheDocument();
    expect(settingsRows.getByText("Cache controls")).toBeInTheDocument();

    const hubSurfaces = section("hub-surfaces");
    expect(hubSurfaces.getByText("Toolbar, Surface, And Bulk Actions")).toBeInTheDocument();
    expect(hubSurfaces.getByRole("textbox", { name: "Search servers" })).toBeInTheDocument();
    expect(hubSurfaces.getByText("Atlas Team")).toBeInTheDocument();
    expect(hubSurfaces.getByText("1 selected")).toBeInTheDocument();

    const workspaceRows = section("workspace-rows");
    expect(workspaceRows.getByText("Server Identity")).toBeInTheDocument();
    expect(workspaceRows.getByText("Members")).toBeInTheDocument();
    expect(workspaceRows.getByText("Voice Participants")).toBeInTheDocument();
    expect(workspaceRows.getAllByText("Aline Costa").length).toBeGreaterThan(0);
    expect(workspaceRows.getByText("Speaking")).toBeInTheDocument();

    const contacts = section("contacts");
    expect(contacts.getByText("Requests")).toBeInTheDocument();
    expect(contacts.getByText("Discovery")).toBeInTheDocument();
    expect(contacts.getByText("Friend requests")).toBeInTheDocument();
    expect(contacts.getByText("Nuno Reis")).toBeInTheDocument();
  });
});
