import {
  notifyWorkspacePreferenceChange,
  readTabRestoreMode,
  subscribeWorkspacePreferences,
  type TabRestoreMode,
} from "@/lib/workspace-preferences";

export type WorkspaceTabKind = "server" | "dm";

export type WorkspaceTab = {
  id: string;
  kind: WorkspaceTabKind;
  href: string;
  label: string;
  imageLabel?: string;
  unread?: number;
  pinned: boolean;
  updatedAt: string;
};

const PINNED_TABS_KEY = "hexrelay.workspace-tabs.pinned";
const SESSION_TABS_KEY = "hexrelay.workspace-tabs.session";
const ALL_TABS_KEY = "hexrelay.workspace-tabs.all";
const TABS_EVENT = "hexrelay-workspace-tabs-changed";

let cachedSnapshotKey = "";
let cachedSnapshot: WorkspaceTab[] = [];
const fallbackLocalItems = new Map<string, string>();
const fallbackSessionItems = new Map<string, string>();

function readLocalStorageItem(key: string): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return fallbackLocalItems.get(key) ?? window.localStorage.getItem(key);
  } catch {
    return fallbackLocalItems.get(key) ?? null;
  }
}

function writeLocalStorageItem(key: string, value: string): void {
  if (typeof window === "undefined") {
    return;
  }

  fallbackLocalItems.set(key, value);
  try {
    window.localStorage.setItem(key, value);
    fallbackLocalItems.delete(key);
  } catch {
    // Keep the in-memory value so tabs still work when persistence is blocked.
  }
}

function readSessionStorageItem(key: string): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return fallbackSessionItems.get(key) ?? window.sessionStorage.getItem(key);
  } catch {
    return fallbackSessionItems.get(key) ?? null;
  }
}

function writeSessionStorageItem(key: string, value: string): void {
  if (typeof window === "undefined") {
    return;
  }

  fallbackSessionItems.set(key, value);
  try {
    window.sessionStorage.setItem(key, value);
    fallbackSessionItems.delete(key);
  } catch {
    // Keep the in-memory value so current-session tabs still work.
  }
}

function notifyTabsChange(): void {
  if (typeof window === "undefined") {
    return;
  }

  window.dispatchEvent(new Event(TABS_EVENT));
}

function parseTabs(raw: string | null): WorkspaceTab[] {
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw) as WorkspaceTab[];
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed.filter(
      (tab) =>
        typeof tab.id === "string" &&
        (tab.kind === "server" || tab.kind === "dm") &&
        typeof tab.href === "string" &&
        typeof tab.label === "string" &&
        typeof tab.pinned === "boolean" &&
        typeof tab.updatedAt === "string",
    );
  } catch {
    return [];
  }
}

function serializeTabs(tabs: WorkspaceTab[]): string {
  return JSON.stringify(dedupeTabs(tabs));
}

function dedupeTabs(tabs: WorkspaceTab[]): WorkspaceTab[] {
  const byId = new Map<string, WorkspaceTab>();
  tabs.forEach((tab) => {
    const existing = byId.get(tab.id);
    if (!existing) {
      byId.set(tab.id, tab);
      return;
    }

    byId.set(tab.id, {
      ...existing,
      ...tab,
      pinned: existing.pinned || tab.pinned,
      updatedAt: existing.updatedAt <= tab.updatedAt ? existing.updatedAt : tab.updatedAt,
    });
  });

  return [...byId.values()].sort((first, second) => {
    if (first.pinned !== second.pinned) {
      return first.pinned ? -1 : 1;
    }

    return 0;
  });
}

function readPinnedTabs(): WorkspaceTab[] {
  return parseTabs(readLocalStorageItem(PINNED_TABS_KEY)).map((tab) => ({ ...tab, pinned: true }));
}

function readSessionTabs(): WorkspaceTab[] {
  return parseTabs(readSessionStorageItem(SESSION_TABS_KEY)).map((tab) => ({ ...tab, pinned: false }));
}

function readAllTabs(): WorkspaceTab[] {
  return parseTabs(readLocalStorageItem(ALL_TABS_KEY));
}

function writePinnedTabs(tabs: WorkspaceTab[]): void {
  writeLocalStorageItem(PINNED_TABS_KEY, serializeTabs(tabs.filter((tab) => tab.pinned)));
}

function writeSessionTabs(tabs: WorkspaceTab[]): void {
  writeSessionStorageItem(SESSION_TABS_KEY, serializeTabs(tabs.filter((tab) => !tab.pinned)));
}

function writeAllTabs(tabs: WorkspaceTab[]): void {
  writeLocalStorageItem(ALL_TABS_KEY, serializeTabs(tabs));
}

function removeAllTab(tabId: string): void {
  writeAllTabs(readAllTabs().filter((tab) => tab.id !== tabId));
}

function upsertAllTab(tab: WorkspaceTab): void {
  writeAllTabs([...readAllTabs().filter((item) => item.id !== tab.id), tab]);
}

function readTabsForMode(mode: TabRestoreMode): WorkspaceTab[] {
  const pinned = readPinnedTabs();
  const unpinned = mode === "all" ? readAllTabs().filter((tab) => !tab.pinned) : readSessionTabs();
  return dedupeTabs([...pinned, ...unpinned]);
}

function persistTabs(tabs: WorkspaceTab[], mode = readTabRestoreMode()): void {
  const next = dedupeTabs(tabs);
  writePinnedTabs(next);

  if (mode === "all") {
    writeAllTabs(next);
  } else {
    writeSessionTabs(next);
  }
}

function safeDecodeSegment(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function shortLabel(value: string): string {
  const readable = value.replace(/[-_]+/g, " ").trim() || value;
  if (readable.length <= 28) {
    return readable;
  }

  return `${readable.slice(0, 18)}...${readable.slice(-6)}`;
}

export function routeToWorkspaceTab(pathname: string): WorkspaceTab | null {
  const serverMatch = pathname.match(/^\/servers\/([^/]+)/);
  if (serverMatch?.[1]) {
    const serverId = safeDecodeSegment(serverMatch[1]);
    return {
      id: `server:${serverId}`,
      kind: "server",
      href: `/servers/${encodeURIComponent(serverId)}`,
      label: shortLabel(serverId),
      pinned: false,
      updatedAt: new Date().toISOString(),
    };
  }

  const dmMatch = pathname.match(/^\/contacts\/([^/]+)\/messages(?:\/|$)/);
  if (dmMatch?.[1]) {
    const contactId = safeDecodeSegment(dmMatch[1]);
    return {
      id: `dm:${contactId}`,
      kind: "dm",
      href: `/contacts/${encodeURIComponent(contactId)}/messages`,
      label: shortLabel(contactId),
      pinned: false,
      updatedAt: new Date().toISOString(),
    };
  }

  return null;
}

export function readWorkspaceTabsSnapshot(): WorkspaceTab[] {
  const mode = readTabRestoreMode();
  const snapshotKey = [
    mode,
    readLocalStorageItem(PINNED_TABS_KEY),
    readLocalStorageItem(ALL_TABS_KEY),
    readSessionStorageItem(SESSION_TABS_KEY),
  ].join("|");

  if (snapshotKey === cachedSnapshotKey) {
    return cachedSnapshot;
  }

  cachedSnapshotKey = snapshotKey;
  cachedSnapshot = readTabsForMode(mode);
  return cachedSnapshot;
}

export function subscribeWorkspaceTabs(onChange: () => void): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  const unsubscribePreferences = subscribeWorkspacePreferences(onChange);

  function handleStorage(event: StorageEvent): void {
    if ([PINNED_TABS_KEY, SESSION_TABS_KEY, ALL_TABS_KEY].includes(event.key ?? "")) {
      onChange();
    }
  }

  window.addEventListener("storage", handleStorage);
  window.addEventListener(TABS_EVENT, onChange);

  return () => {
    unsubscribePreferences();
    window.removeEventListener("storage", handleStorage);
    window.removeEventListener(TABS_EVENT, onChange);
  };
}

export function openWorkspaceTab(tab: WorkspaceTab): void {
  const tabs = readWorkspaceTabsSnapshot();
  const existing = tabs.find((item) => item.id === tab.id);
  const nextTab = existing ? { ...existing, ...tab, pinned: existing.pinned, updatedAt: existing.updatedAt } : tab;
  const next = existing ? tabs.map((item) => (item.id === tab.id ? nextTab : item)) : [...tabs, nextTab];
  persistTabs(next);
  notifyTabsChange();
}

export function closeWorkspaceTab(tabId: string): void {
  persistTabs(readWorkspaceTabsSnapshot().filter((tab) => tab.id !== tabId));
  removeAllTab(tabId);
  notifyTabsChange();
}

export function closeWorkspaceTabsForServer(serverId: string): void {
  closeWorkspaceTab(`server:${serverId}`);
}

export function closeWorkspaceTabsForContact(contactId: string): void {
  closeWorkspaceTab(`dm:${contactId}`);
}

export function toggleWorkspaceTabPinned(tabId: string): void {
  const next = readWorkspaceTabsSnapshot().map((tab) =>
    tab.id === tabId ? { ...tab, pinned: !tab.pinned } : tab,
  );
  const toggled = next.find((tab) => tab.id === tabId);

  persistTabs(next);
  if (toggled) {
    upsertAllTab(toggled);
  }
  notifyTabsChange();
}

export function moveWorkspaceTab(tabId: string, direction: -1 | 1): void {
  const tabs = readWorkspaceTabsSnapshot();
  const index = tabs.findIndex((tab) => tab.id === tabId);
  const nextIndex = index + direction;
  if (index < 0 || nextIndex < 0 || nextIndex >= tabs.length) {
    return;
  }
  if (tabs[index]?.pinned !== tabs[nextIndex]?.pinned) {
    return;
  }

  const next = [...tabs];
  const [tab] = next.splice(index, 1);
  if (!tab) {
    return;
  }
  next.splice(nextIndex, 0, tab);
  persistTabs(next);
  notifyTabsChange();
}

export function reorderWorkspaceTab(tabId: string, beforeTabId: string): void {
  if (tabId === beforeTabId) {
    return;
  }

  const tabs = readWorkspaceTabsSnapshot();
  const tab = tabs.find((item) => item.id === tabId);
  const beforeTab = tabs.find((item) => item.id === beforeTabId);
  if (!tab || !beforeTab || tab.pinned !== beforeTab.pinned) {
    return;
  }

  const withoutTab = tabs.filter((item) => item.id !== tabId);
  const beforeIndex = withoutTab.findIndex((item) => item.id === beforeTabId);
  if (beforeIndex < 0) {
    return;
  }

  const next = [...withoutTab];
  next.splice(beforeIndex, 0, tab);
  persistTabs(next);
  notifyTabsChange();
}

export function syncWorkspaceTabsForRestoreMode(mode: TabRestoreMode): void {
  const tabs =
    mode === "all"
      ? dedupeTabs([...readAllTabs(), ...readSessionTabs(), ...readPinnedTabs()])
      : readWorkspaceTabsSnapshot();
  persistTabs(tabs, mode);
  notifyTabsChange();
  notifyWorkspacePreferenceChange();
}
