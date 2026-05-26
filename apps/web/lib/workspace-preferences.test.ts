import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

class MemoryStorage {
  private values = new Map<string, string>();

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

class ThrowingStorage {
  getItem(): string | null {
    throw new Error("storage unavailable");
  }

  setItem(): void {
    throw new Error("storage unavailable");
  }
}

type StorageLike = Pick<Storage, "getItem" | "setItem">;

function buildWindow(localStorage: StorageLike) {
  const target = new EventTarget();

  return {
    localStorage,
    matchMedia: (query: string) => ({
      matches: query.includes("700px"),
      media: query,
      addEventListener: () => {},
      removeEventListener: () => {},
    }),
    addEventListener: target.addEventListener.bind(target),
    removeEventListener: target.removeEventListener.bind(target),
    dispatchEvent: target.dispatchEvent.bind(target),
  };
}

function installWindow(localStorage: StorageLike = new MemoryStorage()) {
  (globalThis as { window?: unknown }).window = buildWindow(localStorage);
}

describe("workspace preferences", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.resetModules();
    delete (globalThis as { window?: unknown }).window;
  });

  it("defaults message layout to bubble cards", async () => {
    installWindow();
    const { readMessageLayout } = await import("./workspace-preferences");

    expect(readMessageLayout()).toBe("bubble-cards");
  });

  it("persists message layout and notifies subscribers", async () => {
    installWindow();
    const { readMessageLayout, setMessageLayout, subscribeWorkspacePreferences } = await import(
      "./workspace-preferences"
    );
    let changes = 0;
    const unsubscribe = subscribeWorkspacePreferences(() => {
      changes += 1;
    });

    setMessageLayout("continuous-feed");

    expect(readMessageLayout()).toBe("continuous-feed");
    expect(changes).toBe(1);

    unsubscribe();
  });

  it("falls back to bubble cards for unknown stored message layout values", async () => {
    const storage = new MemoryStorage();
    storage.setItem("hexrelay.ui.message-layout", "legacy-density");
    installWindow(storage);
    const { readMessageLayout } = await import("./workspace-preferences");

    expect(readMessageLayout()).toBe("bubble-cards");
  });

  it("keeps message layout usable when storage throws", async () => {
    installWindow(new ThrowingStorage());
    const { readMessageLayout, setMessageLayout } = await import("./workspace-preferences");

    setMessageLayout("continuous-feed");

    expect(readMessageLayout()).toBe("continuous-feed");
  });

  it("persists message bubble size and falls back for unknown stored values", async () => {
    installWindow();
    const { readMessageBubbleSize, setMessageBubbleSize, subscribeWorkspacePreferences } = await import(
      "./workspace-preferences"
    );
    let changes = 0;
    const unsubscribe = subscribeWorkspacePreferences(() => {
      changes += 1;
    });

    expect(readMessageBubbleSize()).toBe("comfortable");

    setMessageBubbleSize("compact");

    expect(readMessageBubbleSize()).toBe("compact");
    expect(changes).toBe(1);

    unsubscribe();

    vi.resetModules();
    const storage = new MemoryStorage();
    storage.setItem("hexrelay.ui.message-bubble-size", "tiny");
    installWindow(storage);
    const { readMessageBubbleSize: readInvalidMessageBubbleSize } = await import("./workspace-preferences");

    expect(readInvalidMessageBubbleSize()).toBe("comfortable");
  });

  it("persists message alignment and falls back for unknown stored values", async () => {
    installWindow();
    const { readMessageAlignment, setMessageAlignment, subscribeWorkspacePreferences } = await import(
      "./workspace-preferences"
    );
    let changes = 0;
    const unsubscribe = subscribeWorkspacePreferences(() => {
      changes += 1;
    });

    expect(readMessageAlignment()).toBe("conversation-sides");

    setMessageAlignment("single-column");

    expect(readMessageAlignment()).toBe("single-column");
    expect(changes).toBe(1);

    unsubscribe();

    vi.resetModules();
    const storage = new MemoryStorage();
    storage.setItem("hexrelay.ui.message-alignment", "legacy-left");
    installWindow(storage);
    const { readMessageAlignment: readInvalidMessageAlignment } = await import("./workspace-preferences");

    expect(readInvalidMessageAlignment()).toBe("conversation-sides");
  });

  it("keeps message visualization preferences usable when storage throws", async () => {
    installWindow(new ThrowingStorage());
    const {
      readMessageAlignment,
      readMessageBubbleSize,
      setMessageAlignment,
      setMessageBubbleSize,
    } = await import("./workspace-preferences");

    setMessageBubbleSize("compact");
    setMessageAlignment("single-column");

    expect(readMessageBubbleSize()).toBe("compact");
    expect(readMessageAlignment()).toBe("single-column");
  });

  it("stores server and contact hub layouts independently", async () => {
    installWindow();
    const { readHubLayout, setHubLayout } = await import("./workspace-preferences");

    expect(readHubLayout("servers")).toBe("list");
    expect(readHubLayout("contacts")).toBe("list");

    setHubLayout("servers", "cards");
    setHubLayout("contacts", "list");

    expect(readHubLayout("servers")).toBe("cards");
    expect(readHubLayout("contacts")).toBe("list");
  });

  it("keeps hub layout usable when storage throws", async () => {
    installWindow(new ThrowingStorage());
    const { readHubLayout, setHubLayout } = await import("./workspace-preferences");

    setHubLayout("contacts", "cards");

    expect(readHubLayout("contacts")).toBe("cards");
  });
});
