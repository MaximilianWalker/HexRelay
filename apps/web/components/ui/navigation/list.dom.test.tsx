// @vitest-environment jsdom

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";

import { List, ListButton, ListLink, ListRow } from "./list";

describe("shared lists", () => {
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

  it("maps implicit list rows to listitem semantics without changing non-list roles", () => {
    render(
      <>
        <List aria-label="Semantic list">
          <ListButton name="Button row" />
          <ListLink href="/servers" name="Link row" />
          <ListRow name="Static row" />
        </List>
        <List aria-label="Grouped controls" role="group">
          <ListButton name="Grouped action" />
        </List>
      </>,
    );

    const semanticList = screen.getByRole("list", { name: "Semantic list" });

    expect(semanticList).toBeInTheDocument();
    expect(within(semanticList).getAllByRole("listitem")).toHaveLength(3);
    expect(screen.getByRole("group", { name: "Grouped controls" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Grouped action" }).parentElement).not.toHaveAttribute(
      "role",
      "listitem",
    );
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
});
