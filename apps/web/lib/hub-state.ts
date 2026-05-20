export type HubKind = "servers" | "contacts";
export type HubLayout = "cards" | "list";

export type HubFilters = {
  search: string;
  pinnedOnly: boolean;
  unreadOnly: boolean;
  mutedOnly: boolean;
};

export type HubSelection = {
  selecting: boolean;
  selectedIds: string[];
};

export const DEFAULT_HUB_FILTERS: HubFilters = {
  search: "",
  pinnedOnly: false,
  unreadOnly: false,
  mutedOnly: false,
};

export const EMPTY_HUB_SELECTION: HubSelection = {
  selecting: false,
  selectedIds: [],
};

export function serializeHubFilters(filters: HubFilters): string {
  return JSON.stringify({
    search: filters.search.trim(),
    pinnedOnly: filters.pinnedOnly,
    unreadOnly: filters.unreadOnly,
    mutedOnly: filters.mutedOnly,
  });
}

export function parseHubFilters(raw: string | null): HubFilters {
  if (!raw) {
    return DEFAULT_HUB_FILTERS;
  }

  try {
    const parsed = JSON.parse(raw) as Partial<HubFilters>;
    return {
      search: typeof parsed.search === "string" ? parsed.search : "",
      pinnedOnly: parsed.pinnedOnly === true,
      unreadOnly: parsed.unreadOnly === true,
      mutedOnly: parsed.mutedOnly === true,
    };
  } catch {
    return DEFAULT_HUB_FILTERS;
  }
}

export function toggleHubSelected(selection: HubSelection, itemId: string): HubSelection {
  const selected = new Set(selection.selectedIds);
  if (selected.has(itemId)) {
    selected.delete(itemId);
  } else {
    selected.add(itemId);
  }

  return {
    selecting: true,
    selectedIds: [...selected],
  };
}

export function clearHubSelection(): HubSelection {
  return EMPTY_HUB_SELECTION;
}

export function selectedHubItems<T extends { id: string }>(items: T[], selection: HubSelection): T[] {
  const selected = new Set(selection.selectedIds);
  return items.filter((item) => selected.has(item.id));
}
