import type { DmConnectivityPreflightReasonCode, DmPairingIdentityKey } from "@/lib/api";

const PAIRING_IMPORT_STORAGE_KEY = "hexrelay.dm_pairing_imports.v1";

export type DmPairingImportRecord = {
  inviterIdentityId: string;
  inviterIdentityKey: DmPairingIdentityKey;
  endpointHints: string[];
  importedAt: string;
  expiresAt: string;
};

type StoredPairingImports = Record<string, DmPairingImportRecord>;

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function isPairingIdentityKey(value: unknown): value is DmPairingIdentityKey {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }

  const key = value as Partial<DmPairingIdentityKey>;
  return (
    typeof key.public_key === "string" &&
    typeof key.algorithm === "string" &&
    typeof key.fingerprint === "string"
  );
}

function isPairingImportRecord(value: unknown): value is DmPairingImportRecord {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }

  const record = value as Partial<DmPairingImportRecord>;
  return (
    typeof record.inviterIdentityId === "string" &&
    isPairingIdentityKey(record.inviterIdentityKey) &&
    isStringArray(record.endpointHints) &&
    typeof record.importedAt === "string" &&
    typeof record.expiresAt === "string"
  );
}

function readStoredPairingImports(): StoredPairingImports {
  if (typeof window === "undefined") {
    return {};
  }

  try {
    const raw = window.sessionStorage.getItem(PAIRING_IMPORT_STORAGE_KEY);
    if (!raw) {
      return {};
    }

    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }

    return parsed as StoredPairingImports;
  } catch {
    return {};
  }
}

function writeStoredPairingImports(value: StoredPairingImports): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.sessionStorage.setItem(PAIRING_IMPORT_STORAGE_KEY, JSON.stringify(value));
  } catch {
    // Losing session-scoped diagnostics should not block contact management.
  }
}

export function isDmPairingImportFresh(record: DmPairingImportRecord, nowMs = Date.now()): boolean {
  const expiresAtMs = Date.parse(record.expiresAt);
  return Number.isFinite(expiresAtMs) && expiresAtMs > nowMs;
}

export function storeDmPairingImport(record: DmPairingImportRecord): void {
  const imports = readStoredPairingImports();
  imports[record.inviterIdentityId] = record;
  writeStoredPairingImports(imports);
}

export function readDmPairingImport(identityId: string): DmPairingImportRecord | null {
  const imports = readStoredPairingImports();
  const record = imports[identityId];
  if (!record) {
    return null;
  }

  if (!isPairingImportRecord(record) || record.inviterIdentityId !== identityId) {
    delete imports[identityId];
    writeStoredPairingImports(imports);
    return null;
  }

  if (!isDmPairingImportFresh(record)) {
    delete imports[identityId];
    writeStoredPairingImports(imports);
    return null;
  }

  return record;
}

export function preflightReasonLabel(reasonCode: DmConnectivityPreflightReasonCode): string {
  switch (reasonCode) {
    case "preflight_ok":
      return "Ready for direct connect";
    case "preflight_ok_lan":
      return "Ready on local network";
    case "preflight_blocked_user":
      return "Blocked contact";
    case "pairing_missing":
      return "Advanced pairing missing";
    case "port_unavailable":
      return "Local port unavailable";
    case "policy_blocked":
      return "DM policy blocked";
    case "peer_unreachable":
      return "Peer unreachable";
  }
}
