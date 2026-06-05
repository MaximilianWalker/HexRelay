// @vitest-environment jsdom

import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { Demo } from "./demo";

afterEach(() => {
  cleanup();
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

    const navButton = screen.getByRole("button", { name: "Open catalog navigation" });
    const title = screen.getByRole("heading", { name: "UI catalog" });
    const headerTitle = navButton.parentElement?.nextElementSibling;

    expect(screen.queryByRole("dialog", { name: "Catalog navigation" })).not.toBeInTheDocument();
    expect(
      Boolean(
        navButton.compareDocumentPosition(title) & Node.DOCUMENT_POSITION_FOLLOWING,
      ),
    ).toBe(true);
    expect(headerTitle).toBe(title);
    expect(screen.queryByText("Development catalog")).not.toBeInTheDocument();
    expect(
      screen.queryByText("Shared primitives, states, tones, and composed patterns used by app surfaces."),
    ).not.toBeInTheDocument();

    await user.click(navButton);

    const dialog = screen.getByRole("dialog", { name: "Catalog navigation" });
    expect(within(dialog).getByRole("link", { name: "Buttons" })).toBeInTheDocument();

    await user.click(within(dialog).getByRole("link", { name: "Menus" }));
    expect(screen.queryByRole("dialog", { name: "Catalog navigation" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Open catalog navigation" }));
    expect(screen.getByRole("dialog", { name: "Catalog navigation" })).toBeInTheDocument();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Catalog navigation" })).not.toBeInTheDocument();
  });

  it("documents the current shared control options", async () => {
    const user = userEvent.setup();

    render(<Demo />);

    const buttons = section("buttons");
    expect(buttons.getByText("Tones")).toBeInTheDocument();
    expect(buttons.queryByText("Icon Sizes")).not.toBeInTheDocument();
    expect(buttons.getByRole("button", { name: "Pressed" })).toHaveAttribute("aria-pressed", "true");
    expect(buttons.queryByRole("button", { name: "Loading" })).not.toBeInTheDocument();
    expect(buttons.queryByRole("button", { name: "Danger active" })).not.toBeInTheDocument();
    expect(buttons.queryByRole("link", { name: "Link" })).not.toBeInTheDocument();

    const toggles = section("toggles");
    expect(screen.queryByText("Button Group Sizes")).not.toBeInTheDocument();
    expect(toggles.queryByText("Button group")).not.toBeInTheDocument();
    expect(toggles.queryByText("Button", { exact: true })).not.toBeInTheDocument();
    expect(document.querySelector('#toggles [class*="sizeTable"]')).not.toBeInTheDocument();
    expect(document.querySelector('#toggles [class*="controlSizeList"]')).toBeInTheDocument();
    expect(toggles.getByRole("group", { name: "Small view mode" }).className).toContain("buttonGroupSm");
    expect(toggles.getByRole("group", { name: "Medium view mode" }).className).not.toContain("buttonGroupSm");
    expect(toggles.getByRole("group", { name: "Medium view mode" }).className).not.toContain("buttonGroupLg");
    expect(toggles.getByRole("group", { name: "Large view mode" }).className).toContain("buttonGroupLg");

    const menus = section("menus");
    expect(menus.getByText("Items")).toBeInTheDocument();
    expect(menus.getByText("Actions")).toBeInTheDocument();
    expect(menus.getByText("Static Content")).toBeInTheDocument();
    expect(menus.getByText("Sizes")).toBeInTheDocument();
    expect(menus.getByText("Profile settings")).toBeInTheDocument();
    expect(menus.getByText("Mute notifications")).toBeInTheDocument();
    expect(menus.getByText("Invite contact")).toBeInTheDocument();
    expect(menus.getByText("Leave server")).toBeInTheDocument();
    expect(menus.getByText("Pin tab")).toBeInTheDocument();
    expect(menus.getByText("Voice unavailable")).toBeInTheDocument();
    expect(menus.getByText("Sidebar layout")).toBeInTheDocument();
    expect(menus.getByText("Edit preferences")).toBeInTheDocument();
    expect(menus.queryByText("Rows")).not.toBeInTheDocument();
    expect(menus.queryByText("Padding and Width")).not.toBeInTheDocument();
    expect(menus.queryByText("Large width")).not.toBeInTheDocument();

    const popups = section("popups");
    const dialogsElement = document.getElementById("dialogs");
    const popupsElement = document.getElementById("popups");
    expect(Boolean(dialogsElement?.compareDocumentPosition(popupsElement as Node) & Node.DOCUMENT_POSITION_FOLLOWING)).toBe(true);
    expect(popups.queryByRole("heading", { name: "Popups" })).not.toBeInTheDocument();
    expect(popups.queryByText("Popups own anchored placement")).not.toBeInTheDocument();
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

    await user.selectOptions(popups.getByRole("combobox", { name: "Content" }), "menu");
    expect(popups.getByText("Open settings")).toBeInTheDocument();
  });
});
