// @vitest-environment jsdom

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { Button, ButtonLink } from "./buttons/button";
import { IconButton } from "./buttons/icon-button";
import { Badge } from "./display/badge";
import { List, ListButton, ListLink, ListRow } from "./navigation/list";
import { Menu } from "./navigation/menu";
import { Popup } from "./overlays/popup";
import { ToggleGroup } from "./toggles/toggle-group";
import { ToggleButton } from "./toggles/toggle-button";

describe("shared controls", () => {
  it("maps pressed state through the shared toggle button behavior", async () => {
    const onPressedChange = vi.fn();
    const user = userEvent.setup();

    render(
      <ToggleButton onPressedChange={onPressedChange} pressed={false}>
        Muted
      </ToggleButton>,
    );

    const button = screen.getByRole("button", { name: "Muted" });
    expect(button).toHaveAttribute("aria-pressed", "false");

    await user.click(button);

    expect(onPressedChange).toHaveBeenCalledWith(true);
  });

  it("uses the same pressed behavior for toggle group options", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <ToggleGroup
        label="View mode"
        onChange={onChange}
        options={[
          { id: "list", label: "List" },
          { id: "cards", label: "Cards" },
        ]}
        value="list"
      />,
    );

    expect(screen.getByRole("button", { name: "List" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "Cards" })).toHaveAttribute("aria-pressed", "false");

    await user.click(screen.getByRole("button", { name: "Cards" }));

    expect(onChange).toHaveBeenCalledWith("cards");
  });

  it("exposes toggle group sizes through shared classes", () => {
    render(
      <>
        <ToggleGroup
          label="Small view mode"
          onChange={vi.fn()}
          options={[
            { id: "list", label: "List" },
            { id: "cards", label: "Cards" },
          ]}
          size="sm"
          value="list"
        />
        <ToggleGroup
          label="Medium view mode"
          onChange={vi.fn()}
          options={[
            { id: "list", label: "List" },
            { id: "cards", label: "Cards" },
          ]}
          value="list"
        />
        <ToggleGroup
          label="Large view mode"
          onChange={vi.fn()}
          options={[
            { id: "list", label: "List" },
            { id: "cards", label: "Cards" },
          ]}
          size="lg"
          value="list"
        />
      </>,
    );

    expect(screen.getByRole("group", { name: "Small view mode" }).className).toContain("toggleGroupSm");
    expect(screen.getByRole("group", { name: "Medium view mode" }).className).not.toContain("toggleGroupSm");
    expect(screen.getByRole("group", { name: "Medium view mode" }).className).not.toContain("toggleGroupLg");
    expect(screen.getByRole("group", { name: "Large view mode" }).className).toContain("toggleGroupLg");
  });

  it("renders link-capable buttons with navigation semantics", () => {
    render(<ButtonLink href="/servers">Servers</ButtonLink>);

    expect(screen.getByRole("link", { name: "Servers" })).toHaveAttribute("href", "/servers");
  });

  it("removes target navigation from disabled link-capable buttons", () => {
    render(
      <ButtonLink disabled href="/onboarding/access">
        Continue to access
      </ButtonLink>,
    );

    const link = screen.getByRole("link", { name: "Continue to access" });

    expect(link).toHaveAttribute("aria-disabled", "true");
    expect(link).toHaveAttribute("href", "#");
    expect(link).toHaveAttribute("tabindex", "-1");
  });

  it("exposes the same three sizes for text and icon-only buttons", () => {
    render(
      <>
        <Button size="sm">Small</Button>
        <Button>Medium</Button>
        <Button size="lg">Large</Button>
        <IconButton label="Small icon" size="sm">
          <span aria-hidden="true">S</span>
        </IconButton>
        <IconButton label="Medium icon">
          <span aria-hidden="true">M</span>
        </IconButton>
        <IconButton label="Large icon" size="lg">
          <span aria-hidden="true">L</span>
        </IconButton>
      </>,
    );

    expect(screen.getByRole("button", { name: "Small" }).className).toContain("buttonSm");
    expect(screen.getByRole("button", { name: "Medium" }).className).not.toContain("buttonSm");
    expect(screen.getByRole("button", { name: "Medium" }).className).not.toContain("buttonLg");
    expect(screen.getByRole("button", { name: "Large" }).className).toContain("buttonLg");
    expect(screen.getByRole("button", { name: "Small icon" }).className).toContain("buttonIcon");
    expect(screen.getByRole("button", { name: "Small icon" }).className).toContain("buttonSm");
    expect(screen.getByRole("button", { name: "Medium icon" }).className).toContain("buttonIcon");
    expect(screen.getByRole("button", { name: "Large icon" }).className).toContain("buttonIcon");
    expect(screen.getByRole("button", { name: "Large icon" }).className).toContain("buttonLg");
  });

  it("lets button icons choose a shared icon size independent of control height", () => {
    render(
      <IconButton iconSize="lg" label="Large glyph">
        <span aria-hidden="true">G</span>
      </IconButton>,
    );

    expect(screen.getByRole("button", { name: "Large glyph" }).className).toContain("buttonIconSizeLg");
  });

  it("maps button alignment, tone, and pressed tone through shared props", () => {
    render(
      <>
        <Button align="center" tone="success">
          Centered success
        </Button>
        <Button pressed pressedTone="danger" tone="danger">
          Dangerous toggle
        </Button>
      </>,
    );

    expect(screen.getByRole("button", { name: "Centered success" }).className).toContain("alignCenter");
    expect(screen.getByRole("button", { name: "Centered success" }).className).toContain("buttonToneSuccess");
    expect(screen.getByRole("button", { name: "Dangerous toggle" }).className).toContain("buttonPressedDanger");
  });

  it("exposes three badge sizes", () => {
    render(
      <>
        <Badge size="sm">Small badge</Badge>
        <Badge>Medium badge</Badge>
        <Badge size="lg">Large badge</Badge>
      </>,
    );

    expect(screen.getByText("Small badge").className).toContain("badgeSm");
    expect(screen.getByText("Medium badge").className).not.toContain("badgeSm");
    expect(screen.getByText("Medium badge").className).not.toContain("badgeLg");
    expect(screen.getByText("Large badge").className).toContain("badgeLg");
  });

  it("maps numeric badges through the shared counter shape", () => {
    render(
      <Badge shape="counter" size="sm" tone="accent">
        2
      </Badge>,
    );

    expect(screen.getByText("2").className).toContain("badgeCounter");
  });

  it("moves focus between list primary actions with arrow keys", async () => {
    const user = userEvent.setup();

    render(
      <List>
        <ListButton name="Pin tab" />
        <ListButton end={<button type="button">End action</button>} name="Close tab" />
      </List>,
    );

    const first = screen.getByRole("button", { name: "Pin tab" });
    const second = screen.getByRole("button", { name: "Close tab" });

    first.focus();
    await user.keyboard("{ArrowDown}");

    expect(second).toHaveFocus();
  });

  it("keeps list active state visual while pressed and current remain explicit semantics", () => {
    render(
      <List>
        <ListButton active name="Visual active" />
        <ListButton name="Pressed action" pressed />
        <ListLink current href="/servers" name="Servers" />
      </List>,
    );

    expect(screen.getByRole("button", { name: "Visual active" })).not.toHaveAttribute("aria-pressed");
    expect(screen.getByRole("button", { name: "Pressed action" })).toHaveAttribute("aria-pressed", "true");
    const currentLink = screen
      .getAllByRole("link", { name: "Servers" })
      .find((link) => link.getAttribute("aria-current") === "page");

    expect(currentLink).toBeInTheDocument();
  });

  it("lets list containers opt out of the default panel frame", () => {
    render(
      <>
        <List aria-label="Panel list">
          <ListButton name="Framed row" />
        </List>
        <List aria-label="Plain list" panel={false}>
          <ListButton name="Plain row" />
        </List>
      </>,
    );

    expect(screen.getByRole("list", { name: "Panel list" })).toHaveAttribute("data-list-panel", "true");
    expect(screen.getByRole("list", { name: "Plain list" })).toHaveAttribute("data-list-panel", "false");
  });

  it("maps popup placement separately from menu content", () => {
    render(
      <Popup placement="bottom-center">
        <List role="menu">
          <ListButton name="Open settings" role="menuitem" />
        </List>
      </Popup>,
    );

    const list = screen.getByText("Open settings").closest('[role="menu"]');
    const popup = list?.parentElement;

    expect(popup).toHaveAttribute("data-position", "absolute");
    expect(popup).toHaveAttribute("data-placement", "bottom-center");
    expect(popup?.className).toContain("popup");
    expect(list?.className).toContain("list");
  });

  it("supports centered popup placement", () => {
    render(
      <Popup placement="center">
        <span>Centered popup</span>
      </Popup>,
    );

    expect(screen.getByText("Centered popup").parentElement).toHaveAttribute("data-placement", "center");
  });

  it("supports dialog-style list rows without ARIA menuitem roles", () => {
    render(
      <List role="dialog">
        <ListButton name="Compact mode" pressed />
        <ListRow end={<span>Sidebar</span>} name="Navigation" />
      </List>,
    );

    expect(screen.getByRole("button", { name: "Compact mode" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.queryByRole("menuitemcheckbox", { name: "Compact mode" })).not.toBeInTheDocument();
    expect(screen.getByText("Navigation").className).toContain("listName");
  });

  it("maps list and menu row sizes through shared classes", () => {
    render(
      <>
        <List role="group">
          <ListButton name="Small list row" size="sm" />
          <ListButton name="Large list row" size="lg" />
        </List>
        <Menu
          items={[
            { icon: <span aria-hidden="true">#</span>, id: "small", name: "Small menu row", size: "sm" },
            { icon: <span aria-hidden="true">#</span>, id: "large", name: "Large menu row", size: "lg" },
          ]}
        />
      </>,
    );

    expect(screen.getByRole("button", { name: "Small list row" }).className).toContain("listPrimarySm");
    expect(screen.getByRole("button", { name: "Large list row" }).className).toContain("listPrimaryLg");
    expect(screen.getByRole("button", { name: "Small menu row" }).className).toContain("listPrimarySm");
    expect(screen.getByRole("button", { name: "Large menu row" }).className).toContain("listPrimaryLg");
  });

  it("renders menu links, command rows, skin exceptions, and active current state", async () => {
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
        skin="sidebar"
        spacing="sm"
      />,
    );

    const nav = screen.getByRole("navigation", { name: "Primary" });
    const servers = within(nav).getByRole("link", { name: "Servers" });
    const command = within(nav).getByRole("button", { name: "Command" });

    expect(nav).toHaveAttribute("data-list-panel", "true");
    expect(nav).toHaveAttribute("data-menu-skin", "sidebar");
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

  it("supports controlled, forced, and empty menu expansion states", async () => {
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
