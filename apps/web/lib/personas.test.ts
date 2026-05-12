import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  ensurePersona,
  readActivePersonaId,
  readPersonas,
  removePersona,
  switchPersona,
  upsertPersona,
} from "./personas";

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

describe("personas", () => {
  beforeEach(() => {
    const uuidValues = ["persona-a", "persona-b"];
    let nowCalls = 0;

    vi.spyOn(crypto, "randomUUID").mockImplementation(() => uuidValues.shift() ?? "persona-z");
    vi.spyOn(Date.prototype, "toISOString").mockImplementation(function toISOStringMock() {
      nowCalls += 1;
      return `2026-04-10T00:00:0${nowCalls}Z`;
    });

    (globalThis as { window?: unknown }).window = {
      dispatchEvent: vi.fn(() => true),
      localStorage: new MemoryStorage(),
      sessionStorage: new MemoryStorage(),
    };
  });

  afterEach(() => {
    vi.restoreAllMocks();
    delete (globalThis as { window?: unknown }).window;
  });

  it("returns empty/default values when storage is unavailable or invalid", () => {
    delete (globalThis as { window?: unknown }).window;
    expect(readPersonas()).toEqual([]);
    expect(readActivePersonaId()).toBeNull();

    (globalThis as { window?: unknown }).window = {
      dispatchEvent: vi.fn(() => true),
      localStorage: new MemoryStorage(),
      sessionStorage: new MemoryStorage(),
    };
    const windowRef = globalThis.window as { localStorage: MemoryStorage };
    windowRef.localStorage.setItem("hexrelay.personas", "not-json");

    expect(readPersonas()).toEqual([]);
  });

  it("creates a new persona, trims the name, and marks it active", () => {
    const created = ensurePersona("  Nora  ");

    expect(created).toEqual({
      id: "persona-a",
      name: "Nora",
      createdAt: "2026-04-10T00:00:01Z",
      lastSelectedAt: "2026-04-10T00:00:01Z",
    });
    expect(readActivePersonaId()).toBe("persona-a");
    expect(readPersonas()).toEqual([created]);
  });

  it("reuses existing personas case-insensitively and refreshes lastSelectedAt", () => {
    const first = ensurePersona("Nora");
    const second = ensurePersona(" nora ");

    expect(second.id).toBe(first.id);
    expect(second.createdAt).toBe(first.createdAt);
    expect(second.lastSelectedAt).toBe("2026-04-10T00:00:02Z");
    expect(readPersonas()).toEqual([second]);
    expect(readActivePersonaId()).toBe(first.id);
  });

  it("switches personas and only updates the selected record timestamp", () => {
    const first = ensurePersona("Nora");
    const second = ensurePersona("Milo");

    const switched = switchPersona(first.id);

    expect(readActivePersonaId()).toBe(first.id);
    expect(switched).toEqual([
      {
        ...second,
      },
      {
        ...first,
        lastSelectedAt: "2026-04-10T00:00:03Z",
      },
    ]);
  });

  it("upserts an explicit fixture persona id and marks it active", () => {
    const first = upsertPersona({ id: "usr-test-alice", name: "alice.primary" });
    const second = upsertPersona({ id: "usr-test-alice", name: "Alice" });

    expect(first.id).toBe("usr-test-alice");
    expect(second).toEqual({
      ...first,
      name: "Alice",
      lastSelectedAt: "2026-04-10T00:00:02Z",
    });
    expect(readPersonas()).toEqual([second]);
    expect(readActivePersonaId()).toBe("usr-test-alice");
  });

  it("uses fixture persona id as fallback name and emits preference events", () => {
    const windowRef = globalThis.window as {
      dispatchEvent: (event: Event) => boolean;
    };
    const dispatchEvent = vi.spyOn(windowRef, "dispatchEvent");

    const created = upsertPersona({ id: "usr-test-alice", name: "   " });
    switchPersona(created.id);
    removePersona(created.id);

    expect(created).toEqual({
      id: "usr-test-alice",
      name: "usr-test-alice",
      createdAt: "2026-04-10T00:00:01Z",
      lastSelectedAt: "2026-04-10T00:00:01Z",
    });
    expect(dispatchEvent).toHaveBeenCalledTimes(3);
    expect(dispatchEvent.mock.calls.map(([event]) => event.type)).toEqual([
      "hexrelay-ui-preferences-changed",
      "hexrelay-ui-preferences-changed",
      "hexrelay-ui-preferences-changed",
    ]);
  });

  it("removes inactive personas without disturbing the active selection", () => {
    const first = ensurePersona("Nora");
    const second = ensurePersona("Milo");

    const remaining = removePersona(first.id);

    expect(remaining).toEqual([second]);
    expect(readActivePersonaId()).toBe(second.id);
  });

  it("reassigns or clears the active persona when the active record is removed", () => {
    const first = ensurePersona("Nora");
    const second = ensurePersona("Milo");

    expect(removePersona(second.id)).toEqual([first]);
    expect(readActivePersonaId()).toBe(first.id);

    expect(removePersona(first.id)).toEqual([]);
    expect(readActivePersonaId()).toBeNull();
  });
});
