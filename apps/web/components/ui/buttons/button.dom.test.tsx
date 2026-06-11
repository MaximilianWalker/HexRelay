// @vitest-environment jsdom

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Button, ButtonLink } from "./button";
import { IconButton } from "./icon-button";

describe("shared buttons", () => {
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
});
