import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  clearPersonaPrivateKey,
  clearPersonaSession,
  getPersonaPrivateKey,
  getPersonaSession,
  setPersonaPrivateKey,
  setPersonaSession,
} from "./sessions";

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

function buildWindow(provider?: SecureStoreProvider) {
  return {
    __HEXRELAY_SECURE_STORE__: provider,
    localStorage: new MemoryStorage(),
    sessionStorage: new MemoryStorage(),
  };
}

describe("sessions", () => {
  beforeEach(() => {
    vi.stubGlobal("atob", (value: string) => Buffer.from(value, "base64").toString("binary"));
    vi.stubGlobal("btoa", (value: string) => Buffer.from(value, "binary").toString("base64"));
    vi.spyOn(Date.prototype, "toISOString").mockReturnValue("2030-01-01T00:00:00Z");
    vi.stubGlobal("TextEncoder", TextEncoder);
    vi.stubGlobal("TextDecoder", TextDecoder);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    delete (globalThis as { window?: unknown }).window;
  });

  it("returns null when session storage is unavailable or payload is malformed", () => {
    expect(getPersonaSession("persona-x")).toBeNull();

    (globalThis as { window?: unknown }).window = buildWindow();
    const windowRef = globalThis.window as ReturnType<typeof buildWindow>;
    windowRef.sessionStorage.setItem("hexrelay.session.runtime.v1.persona-x", "not-json");
    expect(getPersonaSession("persona-x")).toBeNull();

    windowRef.sessionStorage.setItem(
      "hexrelay.session.runtime.v1.persona-x",
      JSON.stringify({ sessionId: "sess-only" }),
    );
    expect(getPersonaSession("persona-x")).toBeNull();
  });

  it("stores and clears runtime persona sessions", () => {
    (globalThis as { window?: unknown }).window = buildWindow();
    const windowRef = globalThis.window as ReturnType<typeof buildWindow>;

    setPersonaSession("persona-1", {
      sessionId: "sess-1",
      expiresAt: "2031-01-01T00:00:00Z",
    });

    expect(getPersonaSession("persona-1")).toEqual({
      sessionId: "sess-1",
      expiresAt: "2031-01-01T00:00:00Z",
    });

    clearPersonaSession("persona-1");
    expect(windowRef.sessionStorage.getItem("hexrelay.session.runtime.v1.persona-1")).toBeNull();
    expect(windowRef.localStorage.getItem("hexrelay.session.v1.persona-1")).toBeNull();
  });

  it("migrates legacy localStorage session into sessionStorage", () => {
    (globalThis as { window?: unknown }).window = buildWindow();
    const windowRef = globalThis.window as ReturnType<typeof buildWindow>;

    windowRef.localStorage.setItem(
      "hexrelay.session.v1.persona-1",
      JSON.stringify({
        sessionId: "sess-1",
        accessToken: "legacy-token",
        expiresAt: "2030-01-01T00:00:00Z",
        updatedAt: "2030-01-01T00:00:00Z",
      }),
    );

    const session = getPersonaSession("persona-1");
    expect(session).toEqual({
      sessionId: "sess-1",
      expiresAt: "2030-01-01T00:00:00Z",
    });

    expect(windowRef.localStorage.getItem("hexrelay.session.v1.persona-1")).toBeNull();
    expect(windowRef.sessionStorage.getItem("hexrelay.session.runtime.v1.persona-1")).toContain("sess-1");
  });

  it("persists and decrypts persona private keys with the secure-store provider", async () => {
    const stored = new Map<string, string>();
    const deriveKey = vi.fn(async () => ({ id: "aes-key" }));
    const encrypt = vi.fn(async () => new Uint8Array([9, 8, 7, 6]).buffer);
    const decrypt = vi.fn(async () => new TextEncoder().encode("deadbeef").buffer);

    vi.stubGlobal("crypto", {
      getRandomValues<T extends ArrayBufferView>(values: T): T {
        const array = values as unknown as Uint8Array;
        array.set([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        return values;
      },
      subtle: {
        decrypt,
        deriveKey,
        encrypt,
        importKey: vi.fn(async () => ({ id: "master-material" })),
      },
    });

    (globalThis as { window?: unknown }).window = buildWindow({
      async getItem(key) {
        return stored.get(key) ?? null;
      },
      async removeItem(key) {
        stored.delete(key);
      },
      async setItem(key, value) {
        stored.set(key, value);
      },
    });

    await setPersonaPrivateKey("persona-1", "deadbeef");

    expect(stored.get("hexrelay.identity.master-key.v1")).toBeTruthy();
    expect(stored.get("hexrelay.identity.private.v1.persona-1")).toBeTruthy();
    expect(deriveKey).toHaveBeenCalledTimes(1);
    expect(encrypt).toHaveBeenCalledTimes(1);

    const decrypted = await getPersonaPrivateKey("persona-1");
    expect(decrypted).toBe("deadbeef");
    expect(decrypt).toHaveBeenCalledTimes(1);
  });

  it("returns null for missing or undecryptable private keys and clears stored key handles", async () => {
    const stored = new Map<string, string>();

    vi.stubGlobal("crypto", {
      getRandomValues<T extends ArrayBufferView>(values: T): T {
        return values;
      },
      subtle: {
        decrypt: vi.fn(async () => {
          throw new Error("decrypt failed");
        }),
        deriveKey: vi.fn(async () => ({ id: "aes-key" })),
        encrypt: vi.fn(async () => new Uint8Array([1, 2, 3]).buffer),
        importKey: vi.fn(async () => ({ id: "master-material" })),
      },
    });

    (globalThis as { window?: unknown }).window = buildWindow({
      async getItem(key) {
        return stored.get(key) ?? null;
      },
      async removeItem(key) {
        stored.delete(key);
      },
      async setItem(key, value) {
        stored.set(key, value);
      },
    });

    expect(await getPersonaPrivateKey("persona-missing")).toBeNull();

    stored.set("hexrelay.identity.private.v1.persona-bad", "broken.payload");
    expect(await getPersonaPrivateKey("persona-bad")).toBeNull();

    clearPersonaPrivateKey("persona-bad");
    await Promise.resolve();
    expect(stored.has("hexrelay.identity.private.v1.persona-bad")).toBe(false);
  });
});
