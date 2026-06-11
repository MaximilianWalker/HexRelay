// @vitest-environment jsdom

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { Menu } from "./menu";

describe("shared menus", () => {
  it("maps list and menu row sizes through shared classes", () => {
    render(
      <Menu
        items={[
          { icon: <span aria-hidden="true">#</span>, id: "small", name: "Small menu row", size: "sm" },
          { icon: <span aria-hidden="true">#</span>, id: "large", name: "Large menu row", size: "lg" },
        ]}
      />,
    );

    expect(screen.getByRole("button", { name: "Small menu row" }).className).toContain("listPrimarySm");
    expect(screen.getByRole("button", { name: "Large menu row" }).className).toContain("listPrimaryLg");
  });

  it("renders links, command rows, active current state, and core visual props", async () => {
    const onSelect = vi.fn();
    const user = userEvent.setup();

    render(
      <Menu
        activeId="servers"
        activeIndicator="none"
        aria-label="Primary"
        as="nav"
        collapsed
        iconColor="accent"
        idleBorder={false}
        items={[
          { href: "/home", icon: <span aria-hidden="true">H</span>, id: "home", name: "Home" },
          { href: "/servers", icon: <span aria-hidden="true">S</span>, id: "servers", name: "Servers" },
          { icon: <span aria-hidden="true">C</span>, id: "command", name: "Command", onSelect },
        ]}
        panel
        spacing="sm"
      />,
    );

    const nav = screen.getByRole("navigation", { name: "Primary" });
    const servers = within(nav).getByRole("link", { name: "Servers" });
    const command = within(nav).getByRole("button", { name: "Command" });

    expect(nav).toHaveAttribute("data-list-panel", "true");
    expect(nav).toHaveAttribute("data-menu-collapsed", "true");
    expect(nav).toHaveAttribute("data-menu-active-indicator", "none");
    expect(nav).toHaveAttribute("data-menu-idle-border", "hidden");
    expect(nav).toHaveAttribute("data-menu-spacing", "sm");
    expect(servers.querySelector('[aria-hidden="true"]')?.className).toContain("listIconAccent");
    expect(servers).toHaveAttribute("aria-current", "page");
    expect(servers).not.toHaveAttribute("aria-pressed");

    await user.click(command);

    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it("supports controlled, forced, and empty expansion states", async () => {
    const onExpandedChange = vi.fn();
    const user = userEvent.setup();

    const { rerender } = render(
      <Menu
        expandedIds={[]}
        forceExpandedIds={["group"]}
        items={[
          {
            id: "group",
            items: [{ id: "child", name: "Child" }],
            name: "Group",
          },
        ]}
        onExpandedChange={onExpandedChange}
      />,
    );

    expect(screen.getByRole("button", { name: "Group" })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: "Child" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Group" }));

    expect(onExpandedChange).toHaveBeenCalledWith(["group"]);

    rerender(<Menu empty={<span>No menu items</span>} items={[]} />);

    expect(screen.getByText("No menu items")).toBeInTheDocument();
  });
});
