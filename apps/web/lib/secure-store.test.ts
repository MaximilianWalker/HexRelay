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

  it("falls back to localStorage when provider setItem fails", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return null;
      },
      async removeItem() {
        throw new Error("remove failed");
      },
      async setItem() {
        throw new Error("set failed");
      },
    };

    await secureSetItem("key-a", "value-a");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-a")).toBe("value-a");
  });

  it("falls back to localStorage when provider get/remove fail", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-b", "value-b");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        throw new Error("get failed");
      },
      async removeItem() {
        throw new Error("remove failed");
      },
      async setItem() {
        return;
      },
    };

    expect(await secureGetItem("key-b")).toBe("value-b");
    await secureRemoveItem("key-b");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-b")).toBeNull();
  });

  it("reads fallback value when provider returns null", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-c", "value-c");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return null;
      },
      async removeItem() {
        return;
      },
      async setItem() {
        return;
      },
    };

    expect(await secureGetItem("key-c")).toBe("value-c");
  });

  it("migrates fallback value back to provider when provider recovers", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    let providerValue: string | null = null;
    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-f", "fallback-f");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return providerValue;
      },
      async removeItem() {
        return;
      },
      async setItem(_key, value) {
        providerValue = value;
      },
    };

    expect(await secureGetItem("key-f")).toBe("fallback-f");
    expect(providerValue).toBe("fallback-f");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-f")).toBeNull();
  });

  it("clears fallback even when provider remove succeeds", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-d", "value-d");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return null;
      },
      async removeItem() {
        return;
      },
      async setItem() {
        return;
      },
    };

    await secureRemoveItem("key-d");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-d")).toBeNull();
  });

  it("clears stale fallback when provider set succeeds", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-e", "stale-value");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return "provider-value";
      },
      async removeItem() {
        return;
      },
      async setItem() {
        return;
      },
    };

    await secureSetItem("key-e", "new-value");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-e")).toBeNull();
  });

  it("prefers fallback over stale provider value", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    let providerValue = "stale-provider";
    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-g", "fresh-fallback");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return providerValue;
      },
      async removeItem() {
        return;
      },
      async setItem(_key, value) {
        providerValue = value;
      },
    };

    expect(await secureGetItem("key-g")).toBe("fresh-fallback");
    expect(providerValue).toBe("fresh-fallback");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-g")).toBeNull();
  });

  it("keeps fallback when provider resync fails", async () => {
    const windowRef = globalThis.window as {
      __HEXRELAY_SECURE_STORE__?: {
        getItem(key: string): Promise<string | null>;
        removeItem(key: string): Promise<void>;
        setItem(key: string, value: string): Promise<void>;
      };
      localStorage: MemoryStorage;
    };

    windowRef.localStorage.setItem("hexrelay.secure.fallback.v1.key-h", "fallback-h");
    windowRef.__HEXRELAY_SECURE_STORE__ = {
      async getItem() {
        return "stale-provider";
      },
      async removeItem() {
        return;
      },
      async setItem() {
        throw new Error("provider unavailable");
      },
    };

    expect(await secureGetItem("key-h")).toBe("fallback-h");
    expect(windowRef.localStorage.getItem("hexrelay.secure.fallback.v1.key-h")).toBe("fallback-h");
  });
});
