// @vitest-environment jsdom

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ScrollButton } from "./scroll-button";

describe("shared scroll buttons", () => {
  it("renders directional scroll controls with shared appearances", () => {
    render(
      <>
        <ScrollButton direction="previous" label="Scroll left" />
        <ScrollButton appearance="plain" direction="next" label="Scroll right" />
      </>,
    );

    expect(screen.getByRole("button", { name: "Scroll left" })).toHaveAttribute(
      "data-scroll-button-appearance",
      "framed",
    );
    expect(screen.getByRole("button", { name: "Scroll right" })).toHaveAttribute(
      "data-scroll-button-appearance",
      "plain",
    );
  });
});
