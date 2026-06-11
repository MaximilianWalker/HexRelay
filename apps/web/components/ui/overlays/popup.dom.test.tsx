// @vitest-environment jsdom

import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { List, ListButton } from "../navigation/list";
import { Popup } from "./popup";

describe("shared popups", () => {
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
});
