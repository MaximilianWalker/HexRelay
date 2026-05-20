import { describe, expect, it } from "vitest";

import {
  DEFAULT_HUB_FILTERS,
  clearHubSelection,
  parseHubFilters,
  selectedHubItems,
  serializeHubFilters,
  toggleHubSelected,
} from "./hub-state";

describe("hub state", () => {
  it("serializes and parses hub filters", () => {
    const serialized = serializeHubFilters({
      search: "  atlas  ",
      pinnedOnly: true,
      unreadOnly: false,
      mutedOnly: true,
    });

    expect(parseHubFilters(serialized)).toEqual({
      search: "atlas",
      pinnedOnly: true,
      unreadOnly: false,
      mutedOnly: true,
    });
  });

  it("falls back to defaults for invalid persisted filters", () => {
    expect(parseHubFilters("{bad")).toEqual(DEFAULT_HUB_FILTERS);
    expect(parseHubFilters(null)).toEqual(DEFAULT_HUB_FILTERS);
  });

  it("tracks selected hub items deterministically", () => {
    const first = toggleHubSelected(clearHubSelection(), "a");
    const second = toggleHubSelected(first, "b");
    const third = toggleHubSelected(second, "a");

    expect(second.selectedIds).toEqual(["a", "b"]);
    expect(third.selectedIds).toEqual(["b"]);
    expect(selectedHubItems([{ id: "a" }, { id: "b" }], third)).toEqual([{ id: "b" }]);
  });
});
