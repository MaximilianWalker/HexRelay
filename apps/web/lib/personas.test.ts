import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  ensurePersona,
  readActivePersonaId,
  readPersonas,
  removePersona,
  switchPersona,
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
      localStorage: new MemoryStorage(),
      sessionStorage: new MemoryStorage(),
    };
    const windowRef = globalThis.window as { localStorage: MemoryStorage };
    windowRef.localStorage.setItem("hexrelay.personas.v1", "not-json");

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
