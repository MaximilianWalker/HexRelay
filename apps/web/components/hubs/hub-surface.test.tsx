import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { HubSurface } from "./hub-surface";

const items = [
  {
    id: "srv-1",
    muted: false,
    name: "Atlas",
    pinned: true,
    unread: 2,
  },
];

describe("HubSurface", () => {
  it("does not render a persistent select button", () => {
    const markup = renderToStaticMarkup(
      <HubSurface
        items={items}
        layout="cards"
        noun="server"
        onOpen={vi.fn()}
        onToggleSelected={vi.fn()}
        selectedIds={new Set()}
        selecting={false}
      />,
    );

    expect(markup).not.toContain(">Select<");
  });

  it("marks selected items through the primary item button while selecting", () => {
    const markup = renderToStaticMarkup(
      <HubSurface
        items={items}
        layout="list"
        noun="server"
        onOpen={vi.fn()}
        onToggleSelected={vi.fn()}
        selectedIds={new Set(["srv-1"])}
        selecting
      />,
    );

    expect(markup).toContain('aria-pressed="true"');
    expect(markup).toContain("2 unread");
  });
});
