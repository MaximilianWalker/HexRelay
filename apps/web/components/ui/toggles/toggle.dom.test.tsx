// @vitest-environment jsdom

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ToggleButton } from "./toggle-button";
import { ToggleGroup } from "./toggle-group";

describe("shared toggles", () => {
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
});
