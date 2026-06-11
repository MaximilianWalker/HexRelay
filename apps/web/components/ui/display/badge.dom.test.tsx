// @vitest-environment jsdom

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Badge } from "./badge";

describe("shared badges", () => {
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
});
