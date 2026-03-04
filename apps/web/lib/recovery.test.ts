import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { getOrCreateRecoveryPhraseForPersona } from "./recovery";

vi.mock("@/lib/sessions", () => ({
  getPersonaPrivateKey: vi.fn(async () => "ab".repeat(32)),
}));

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

describe("recovery phrase derivation", () => {
  beforeEach(() => {
    (globalThis as { window?: unknown }).window = {
      localStorage: new MemoryStorage(),
      sessionStorage: new MemoryStorage(),
    };
  });

  afterEach(() => {
    delete (globalThis as { window?: unknown }).window;
  });

  it("derives stable 12-word phrase for same persona", async () => {
    const first = await getOrCreateRecoveryPhraseForPersona("persona-a");
    const second = await getOrCreateRecoveryPhraseForPersona("persona-a");

    expect(first).toHaveLength(12);
    expect(second).toEqual(first);
  });
});
