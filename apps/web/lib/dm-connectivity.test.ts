import { afterEach, describe, expect, it } from "vitest";

import {
  isDmPairingImportFresh,
  preflightReasonLabel,
  readDmPairingImport,
  storeDmPairingImport,
  type DmPairingImportRecord,
} from "./dm-connectivity";

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

const PAIRING_IMPORT_STORAGE_KEY = "hexrelay.dm_pairing_imports.v1";

function installStorage(): void {
  (globalThis as { window?: unknown }).window = {
    sessionStorage: new MemoryStorage(),
  };
}

function pairingRecord(overrides: Partial<DmPairingImportRecord> = {}): DmPairingImportRecord {
  return {
    inviterIdentityId: "usr-nora-k",
    inviterIdentityKey: {
      public_key: "aa".repeat(32),
      algorithm: "ed25519",
      fingerprint: "fingerprint-1",
    },
    endpointHints: ["tcp://127.0.0.1:4040"],
    importedAt: "2026-05-07T00:00:00.000Z",
    expiresAt: "2030-05-07T00:00:00.000Z",
    ...overrides,
  };
}

describe("DM connectivity helpers", () => {
  afterEach(() => {
    delete (globalThis as { window?: unknown }).window;
  });

  it("stores and reads session-scoped pairing imports", () => {
    installStorage();

    storeDmPairingImport(pairingRecord());

    const record = readDmPairingImport("usr-nora-k");
    expect(record?.inviterIdentityId).toBe("usr-nora-k");
    expect(record?.endpointHints).toEqual(["tcp://127.0.0.1:4040"]);
  });

  it("treats expired pairing imports as missing", () => {
    installStorage();

    storeDmPairingImport(pairingRecord({ expiresAt: "2020-05-07T00:00:00.000Z" }));

    expect(readDmPairingImport("usr-nora-k")).toBeNull();
  });

  it("treats malformed pairing imports as missing", () => {
    installStorage();
    const windowRef = globalThis.window as { sessionStorage: MemoryStorage };
    windowRef.sessionStorage.setItem(
      PAIRING_IMPORT_STORAGE_KEY,
      JSON.stringify({
        "usr-nora-k": {
          inviterIdentityId: "usr-nora-k",
          inviterIdentityKey: null,
          endpointHints: "tcp://127.0.0.1:4040",
          importedAt: "2026-05-07T00:00:00.000Z",
          expiresAt: "2030-05-07T00:00:00.000Z",
        },
      }),
    );

    expect(readDmPairingImport("usr-nora-k")).toBeNull();
  });

  it("treats mismatched pairing import identities as missing", () => {
    installStorage();
    const windowRef = globalThis.window as { sessionStorage: MemoryStorage };
    windowRef.sessionStorage.setItem(
      PAIRING_IMPORT_STORAGE_KEY,
      JSON.stringify({
        "usr-jules-p": pairingRecord({ inviterIdentityId: "usr-nora-k" }),
      }),
    );

    expect(readDmPairingImport("usr-jules-p")).toBeNull();
  });

  it("checks pairing import freshness deterministically", () => {
    expect(
      isDmPairingImportFresh(pairingRecord({ expiresAt: "2026-05-07T00:01:00.000Z" }), Date.parse("2026-05-07T00:00:00.000Z")),
    ).toBe(true);
    expect(
      isDmPairingImportFresh(pairingRecord({ expiresAt: "2026-05-07T00:00:00.000Z" }), Date.parse("2026-05-07T00:01:00.000Z")),
    ).toBe(false);
  });

  it("maps preflight reason codes to operator-facing labels", () => {
    expect(preflightReasonLabel("peer_unreachable")).toBe("Peer unreachable");
    expect(preflightReasonLabel("preflight_ok_lan")).toBe("Ready on local network");
  });
});
