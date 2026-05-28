import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { WorkspaceTab } from "./workspace-tabs";

class MemoryStorage {
  private values = new Map<string, string>();

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  removeItem(key: string): void {
    this.values.delete(key);
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

class ThrowingStorage {
  getItem(): string | null {
    throw new Error("storage unavailable");
  }

  removeItem(): void {
    throw new Error("storage unavailable");
  }

  setItem(): void {
    throw new Error("storage unavailable");
  }
}

type StorageLike = Pick<Storage, "getItem" | "removeItem" | "setItem">;

function buildWindow(localStorage: StorageLike, sessionStorage: StorageLike) {
  const target = new EventTarget();

  return {
    localStorage,
    sessionStorage,
    addEventListener: target.addEventListener.bind(target),
    removeEventListener: target.removeEventListener.bind(target),
    dispatchEvent: target.dispatchEvent.bind(target),
  };
}

function installWindow(localStorage = new MemoryStorage(), sessionStorage = new MemoryStorage()) {
  (globalThis as { window?: unknown }).window = buildWindow(localStorage, sessionStorage);
}

function makeTab(id: string, pinned = false): WorkspaceTab {
  const [kind, value] = id.split(":", 2) as [WorkspaceTab["kind"], string];

  return {
    id,
    kind,
    href: kind === "dm" ? `/contacts/${value}/messages` : `/servers/${value}`,
    label: value,
    pinned,
    updatedAt: "2026-04-10T00:00:00Z",
  };
}

async function loadModules() {
  const [tabs, preferences] = await Promise.all([
    import("./workspace-tabs"),
    import("./workspace-preferences"),
  ]);

  return { ...tabs, ...preferences };
}

describe("workspace tabs", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.spyOn(Date.prototype, "toISOString").mockReturnValue("2026-04-10T00:00:00Z");
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.resetModules();
    delete (globalThis as { window?: unknown }).window;
  });

  it("maps workspace routes into stable tab records", async () => {
    const { routeToWorkspaceTab } = await loadModules();

    expect(routeToWorkspaceTab("/servers/core%20team/channels")?.href).toBe("/servers/core%20team");
    expect(routeToWorkspaceTab("/contacts/usr-nora/messages")?.id).toBe("dm:usr-nora");
    expect(routeToWorkspaceTab("/settings")).toBeNull();
  });

  it("preserves previously saved unpinned tabs when restore-all is re-enabled", async () => {
    const localStorage = new MemoryStorage();
    installWindow(localStorage, new MemoryStorage());
    const {
      openWorkspaceTab,
      readWorkspaceTabsSnapshot,
      setTabRestoreMode,
      syncWorkspaceTabsForRestoreMode,
    } = await loadModules();

    setTabRestoreMode("all");
    openWorkspaceTab(makeTab("server:archive"));
    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["server:archive"]);

    installWindow(localStorage, new MemoryStorage());
    setTabRestoreMode("pinned");
    expect(readWorkspaceTabsSnapshot()).toEqual([]);

    syncWorkspaceTabsForRestoreMode("all");
    setTabRestoreMode("all");

    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["server:archive"]);
  });

  it("persists pinned tabs across sessions and drops unpinned tabs after restart", async () => {
    const localStorage = new MemoryStorage();
    installWindow(localStorage, new MemoryStorage());
    const { openWorkspaceTab, readWorkspaceTabsSnapshot, setTabRestoreMode, toggleWorkspaceTabPinned } =
      await loadModules();

    setTabRestoreMode("pinned");
    openWorkspaceTab(makeTab("dm:usr-nora"));
    toggleWorkspaceTabPinned("dm:usr-nora");

    expect(readWorkspaceTabsSnapshot()).toMatchObject([{ id: "dm:usr-nora", pinned: true }]);

    installWindow(localStorage, new MemoryStorage());
    setTabRestoreMode("pinned");
    expect(readWorkspaceTabsSnapshot()).toMatchObject([{ id: "dm:usr-nora", pinned: true }]);

    toggleWorkspaceTabPinned("dm:usr-nora");
    expect(readWorkspaceTabsSnapshot()).toMatchObject([{ id: "dm:usr-nora", pinned: false }]);

    installWindow(localStorage, new MemoryStorage());
    setTabRestoreMode("pinned");
    expect(readWorkspaceTabsSnapshot()).toEqual([]);
  });

  it("does not resurrect explicitly closed tabs when restore-all is re-enabled", async () => {
    const localStorage = new MemoryStorage();
    installWindow(localStorage, new MemoryStorage());
    const {
      closeWorkspaceTab,
      openWorkspaceTab,
      readWorkspaceTabsSnapshot,
      setTabRestoreMode,
      syncWorkspaceTabsForRestoreMode,
    } = await loadModules();

    setTabRestoreMode("all");
    openWorkspaceTab(makeTab("server:archive"));
    syncWorkspaceTabsForRestoreMode("pinned");
    setTabRestoreMode("pinned");

    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["server:archive"]);

    closeWorkspaceTab("server:archive");
    installWindow(localStorage, new MemoryStorage());
    setTabRestoreMode("pinned");
    syncWorkspaceTabsForRestoreMode("all");
    setTabRestoreMode("all");

    expect(readWorkspaceTabsSnapshot()).toEqual([]);
  });

  it("keeps an explicitly unpinned tab restorable as an unpinned tab", async () => {
    const localStorage = new MemoryStorage();
    installWindow(localStorage, new MemoryStorage());
    const {
      openWorkspaceTab,
      readWorkspaceTabsSnapshot,
      setTabRestoreMode,
      syncWorkspaceTabsForRestoreMode,
      toggleWorkspaceTabPinned,
    } = await loadModules();

    setTabRestoreMode("all");
    openWorkspaceTab(makeTab("dm:usr-nora"));
    toggleWorkspaceTabPinned("dm:usr-nora");
    syncWorkspaceTabsForRestoreMode("pinned");
    setTabRestoreMode("pinned");
    toggleWorkspaceTabPinned("dm:usr-nora");

    installWindow(localStorage, new MemoryStorage());
    setTabRestoreMode("pinned");
    syncWorkspaceTabsForRestoreMode("all");
    setTabRestoreMode("all");

    expect(readWorkspaceTabsSnapshot()).toMatchObject([{ id: "dm:usr-nora", pinned: false }]);
  });

  it("keeps tabs usable when storage reads and writes throw", async () => {
    installWindow(new ThrowingStorage(), new ThrowingStorage());
    const { openWorkspaceTab, readWorkspaceTabsSnapshot, setTabRestoreMode, toggleWorkspaceTabPinned } =
      await loadModules();

    setTabRestoreMode("pinned");
    openWorkspaceTab(makeTab("server:local"));
    expect(readWorkspaceTabsSnapshot()).toMatchObject([{ id: "server:local", pinned: false }]);

    toggleWorkspaceTabPinned("server:local");
    expect(readWorkspaceTabsSnapshot()).toMatchObject([{ id: "server:local", pinned: true }]);
  });

  it("supports manual reorder and server/contact tab cleanup", async () => {
    installWindow(new MemoryStorage(), new MemoryStorage());
    const {
      closeWorkspaceTabsForContact,
      closeWorkspaceTabsForServer,
      moveWorkspaceTab,
      openWorkspaceTab,
      readWorkspaceTabsSnapshot,
      reorderWorkspaceTab,
      setTabRestoreMode,
    } = await loadModules();

    setTabRestoreMode("all");
    openWorkspaceTab(makeTab("server:a"));
    openWorkspaceTab(makeTab("server:b"));
    openWorkspaceTab(makeTab("dm:c"));

    openWorkspaceTab({ ...makeTab("server:a"), label: "Server A" });
    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["server:a", "server:b", "dm:c"]);
    expect(readWorkspaceTabsSnapshot()[0]?.label).toBe("Server A");

    moveWorkspaceTab("server:b", -1);
    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["server:b", "server:a", "dm:c"]);

    reorderWorkspaceTab("dm:c", "server:b");
    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["dm:c", "server:b", "server:a"]);

    closeWorkspaceTabsForServer("b");
    closeWorkspaceTabsForContact("c");
    expect(readWorkspaceTabsSnapshot().map((tab) => tab.id)).toEqual(["server:a"]);
  });
});
