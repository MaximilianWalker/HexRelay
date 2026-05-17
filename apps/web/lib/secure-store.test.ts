import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { secureGetItem, secureRemoveItem, secureSetItem } from "./secure-store";

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

type SecureStoreProvider = {
  getItem(key: string): Promise<string | null>;
  removeItem(key: string): Promise<void>;
  setItem(key: string, value: string): Promise<void>;
};

type WindowWithSecureStore = {
  __HEXRELAY_SECURE_STORE__?: SecureStoreProvider;
  localStorage: MemoryStorage;
  sessionStorage: MemoryStorage;
};

function currentWindow(): WindowWithSecureStore {
  return globalThis.window as unknown as WindowWithSecureStore;
}

describe("secure-store", () => {
  beforeEach(() => {
    (globalThis as { window?: unknown }).window = {
      localStorage: new MemoryStorage(),
      sessionStorage: new MemoryStorage(),
    };
  });

  afterEach(() => {
    delete (globalThis as { window?: unknown }).window;
  });

  it("returns null when the secure-store provider is unavailable", async () => {
    const windowRef = currentWindow();
    windowRef.sessionStorage.setItem("hexrelay.secure.fallback.key-a", "value-a");

    expect(await secureGetItem("key-a")).toBeNull();
  });

  it("rejects writes when the secure-store provider is unavailable", async () => {
    const windowRef = currentWindow();

    await expect(secureSetItem("key-a", "value-a")).rejects.toThrow(
      "Secure storage provider unavailable",
    );
    expect(windowRef.sessionStorage.getItem("hexrelay.secure.fallback.key-a")).toBeNull();
  });

  it("stores, reads, and removes values through the secure-store provider", async () => {
    const stored = new Map<string, string>();
    const windowRef = currentWindow();
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem(key) {
        return stored.get(key) ?? null;
      },
      async removeItem(key) {
        stored.delete(key);
      },
      async setItem(key, value) {
        stored.set(key, value);
      },
    };

    await secureSetItem("key-b", "value-b");

    expect(await secureGetItem("key-b")).toBe("value-b");
    expect(windowRef.sessionStorage.getItem("hexrelay.secure.fallback.key-b")).toBeNull();

    await secureRemoveItem("key-b");
    expect(await secureGetItem("key-b")).toBeNull();
  });

  it("returns null when provider reads fail", async () => {
    const windowRef = currentWindow();
    windowRef.sessionStorage.setItem("hexrelay.secure.fallback.key-c", "value-c");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        throw new Error("get failed");
      },
      async removeItem() {
        return;
      },
      async setItem() {
        return;
      },
    };

    expect(await secureGetItem("key-c")).toBeNull();
  });

  it("rejects provider write failures without using browser storage", async () => {
    const windowRef = currentWindow();
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return null;
      },
      async removeItem() {
        return;
      },
      async setItem() {
        throw new Error("set failed");
      },
    };

    await expect(secureSetItem("key-d", "value-d")).rejects.toThrow(
      "Secure storage provider write failed",
    );
    expect(windowRef.sessionStorage.getItem("hexrelay.secure.fallback.key-d")).toBeNull();
  });

  it("treats provider remove failures as best-effort cleanup", async () => {
    const windowRef = currentWindow();
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return null;
      },
      async removeItem() {
        throw new Error("remove failed");
      },
      async setItem() {
        return;
      },
    };

    await expect(secureRemoveItem("key-e")).resolves.toBeUndefined();
  });
});
