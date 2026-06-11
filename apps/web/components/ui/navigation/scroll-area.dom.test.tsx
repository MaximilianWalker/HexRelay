// @vitest-environment jsdom

import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ScrollArea } from "./scroll-area";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

function mockScrollableViewport(viewport: HTMLElement) {
  Object.defineProperties(viewport, {
    clientHeight: { configurable: true, value: 100 },
    scrollHeight: { configurable: true, value: 500 },
    scrollTop: { configurable: true, value: 0, writable: true },
  });
}

describe("ScrollArea", () => {
  it("overlays content and stays visible by default", async () => {
    render(<ScrollArea>Scrollable content</ScrollArea>);

    const root = screen.getByTestId("scroll-area");
    const viewport = screen.getByTestId("scroll-area-viewport");

    mockScrollableViewport(viewport);

    await act(async () => {
      fireEvent.scroll(viewport);
    });

    expect(root).toHaveAttribute("data-overlay", "true");
    expect(root).toHaveAttribute("data-scrollbar-visible", "true");
    expect(root).not.toHaveAttribute("data-hide-when-idle");
    expect(root).toHaveStyle({ "--scroll-area-thumb-width": "4px" });
  });

  it("supports reserved width and numeric scrollbar width", () => {
    render(
      <ScrollArea overlay={false} width={10}>
        Scrollable content
      </ScrollArea>,
    );

    const root = screen.getByTestId("scroll-area");

    expect(root).toHaveAttribute("data-overlay", "false");
    expect(root).toHaveStyle({
      "--scroll-area-thumb-width": "10px",
      "--scroll-area-track-width": "14px",
    });
  });

  it("can hide after the configured idle delay", async () => {
    vi.useFakeTimers();

    render(
      <ScrollArea hideDelayMs={300} hideWhenIdle>
        Scrollable content
      </ScrollArea>,
    );

    const root = screen.getByTestId("scroll-area");
    const viewport = screen.getByTestId("scroll-area-viewport");

    mockScrollableViewport(viewport);

    await act(async () => {
      fireEvent.scroll(viewport);
    });

    expect(root).toHaveAttribute("data-hide-when-idle", "true");
    expect(root).toHaveAttribute("data-scrollbar-visible", "true");

    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(root).not.toHaveAttribute("data-scrollbar-visible");
  });
});
