import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { getPersonaSession } from "./sessions";

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

describe("sessions", () => {
  beforeEach(() => {
    (globalThis as { window?: unknown }).window = {
      localStorage: new MemoryStorage(),
      sessionStorage: new MemoryStorage(),
    };
  });

  afterEach(() => {
    delete (globalThis as { window?: unknown }).window;
  });

  it("migrates legacy localStorage token into sessionStorage", () => {
    const windowRef = globalThis.window as {
      localStorage: MemoryStorage;
      sessionStorage: MemoryStorage;
    };

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
    expect(session?.accessToken).toBe("legacy-token");
    expect(windowRef.sessionStorage.getItem("hexrelay.session.token.v1.persona-1")).toBe(
      "legacy-token",
    );

    const persisted = windowRef.localStorage.getItem("hexrelay.session.v1.persona-1");
    expect(persisted).not.toContain("accessToken");
  });

  it("scrubs legacy localStorage token even when sessionStorage already has token", () => {
    const windowRef = globalThis.window as {
      localStorage: MemoryStorage;
      sessionStorage: MemoryStorage;
    };

    windowRef.localStorage.setItem(
      "hexrelay.session.v1.persona-2",
      JSON.stringify({
        sessionId: "sess-2",
        accessToken: "legacy-token-2",
        expiresAt: "2030-01-01T00:00:00Z",
        updatedAt: "2030-01-01T00:00:00Z",
      }),
    );
    windowRef.sessionStorage.setItem("hexrelay.session.token.v1.persona-2", "active-token-2");

    const session = getPersonaSession("persona-2");
    expect(session?.accessToken).toBe("active-token-2");

    const persisted = windowRef.localStorage.getItem("hexrelay.session.v1.persona-2");
    expect(persisted).not.toContain("accessToken");
  });
});
