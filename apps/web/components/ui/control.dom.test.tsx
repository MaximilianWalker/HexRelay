// @vitest-environment jsdom

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { Badge } from "./badge";
import { ButtonGroup } from "./button-group";
import { Button, ButtonLink } from "./button";
import { IconButton } from "./icon-button";
import { Menu, MenuItem } from "./menu";
import { ToggleButton } from "./toggle-button";

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

  it("uses the same pressed behavior for button group options", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <ButtonGroup
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

  it("moves focus between menu items with arrow keys", async () => {
    const user = userEvent.setup();

    render(
      <Menu>
        <MenuItem>Pin tab</MenuItem>
        <MenuItem>Close tab</MenuItem>
      </Menu>,
    );

    const first = screen.getByRole("menuitem", { name: "Pin tab" });
    const second = screen.getByRole("menuitem", { name: "Close tab" });

    first.focus();
    await user.keyboard("{ArrowDown}");

    expect(second).toHaveFocus();
  });

  it("supports dialog-style menu rows without ARIA menuitem roles", () => {
    render(
      <Menu role="dialog">
        <MenuItem pressed role="button">
          Compact mode
        </MenuItem>
      </Menu>,
    );

    expect(screen.getByRole("button", { name: "Compact mode" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.queryByRole("menuitemcheckbox", { name: "Compact mode" })).not.toBeInTheDocument();
  });
});
