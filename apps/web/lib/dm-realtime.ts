const DEVICE_ID_STORAGE_KEY = "hexrelay.dm.device-id";
const DEVICE_SECRET_STORAGE_KEY = "hexrelay.dm.device-secret";
const ENVELOPE_STORAGE_PREFIX = "hexrelay.dm.envelopes";
const MAX_STORED_ENVELOPES = 500;
const DEVICE_ID_PATTERN = /^[A-Za-z0-9_-]{1,64}$/;
const DEVICE_SECRET_PATTERN = /^[A-Za-z0-9_-]{16,128}$/;

export type DmEnvelopeAckSource = {
  envelopeId: string;
  messageId: string;
  threadId: string;
  recipientIdentityId: string;
  deviceId: string;
  deliveryCursor: string;
};

export type StoredDmEnvelope = DmEnvelopeAckSource & {
  ciphertext: string;
  senderIdentityId?: string;
  sourceDeviceId?: string | null;
  acceptedAt?: string;
  dispatchedAt?: string;
  storedAt: string;
};

export type DmEnvelopeDispatchedEvent = {
  event_type: "dm.envelope.dispatched";
  correlation_id?: string;
  data: {
    envelope_id: string;
    message_id: string;
    thread_id: string;
    sender_identity_id: string;
    recipient_identity_id: string;
    target_device_id: string;
    ciphertext: string;
    accepted_at: string;
    dispatched_at: string;
    delivery_cursor: string;
    transport_scope: "encrypted_envelope_server";
  };
};

export type DmEnvelopeAckEvent = {
  event_type: "dm.envelope.ack";
  data: {
    envelope_id: string;
    message_id: string;
    thread_id: string;
    recipient_identity_id: string;
    device_id: string;
    delivery_cursor: string;
    ack_status: "received";
    received_at: string;
  };
};

export type DmDeviceVerifiedEvent = {
  event_type: "dm.device.verified";
  data: {
    device_id: string;
    verified_at: string;
  };
};

export type DmFanoutCatchUpItem = {
  envelope_id: string;
  cursor: string;
  thread_id: string;
  message_id: string;
  ciphertext: string;
  source_device_id?: string | null;
};

export type DmFanoutCatchUpPayload = {
  status: "ready" | "blocked";
  replay_count: number;
  items: DmFanoutCatchUpItem[];
};

export type DmCatchUpResult = {
  status: "current" | "blocked" | "failed" | "max_pages";
  pages: number;
  stored: number;
  acked: number;
};

function browserStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function isValidDeviceId(value: string): boolean {
  return DEVICE_ID_PATTERN.test(value);
}

function isValidDeviceSecret(value: string): boolean {
  return DEVICE_SECRET_PATTERN.test(value);
}

function createDeviceId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `web-${crypto.randomUUID()}`;
  }

  if (typeof crypto !== "undefined" && typeof crypto.getRandomValues === "function") {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
    return `web-${hex}`;
  }

  return `web-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
}

function createDeviceSecret(): string | null {
  if (typeof crypto === "undefined" || typeof crypto.getRandomValues !== "function") {
    return null;
  }

  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
  return `secret-${hex}`;
}

function createCorrelationId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }

  return createDeviceId();
}

export function getOrCreateDmDeviceId(storage = browserStorage()): string | null {
  if (!storage) {
    return null;
  }

  try {
    const existing = storage.getItem(DEVICE_ID_STORAGE_KEY);
    if (existing && isValidDeviceId(existing)) {
      return existing;
    }

    const created = createDeviceId();
    storage.setItem(DEVICE_ID_STORAGE_KEY, created);
    return created;
  } catch {
    return null;
  }
}

export function getOrCreateDmDeviceSecret(storage = browserStorage()): string | null {
  if (!storage) {
    return null;
  }

  try {
    const existing = storage.getItem(DEVICE_SECRET_STORAGE_KEY);
    if (existing && isValidDeviceSecret(existing)) {
      return existing;
    }

    const created = createDeviceSecret();
    if (!created) {
      return null;
    }
    storage.setItem(DEVICE_SECRET_STORAGE_KEY, created);
    return created;
  } catch {
    return null;
  }
}

export function buildRealtimeWebSocketUrl(baseUrl: string, deviceId: string): string {
  const url = new URL(baseUrl);
  url.searchParams.set("device_id", deviceId);
  return url.toString();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(value: Record<string, unknown>, key: string): string | null {
  const field = value[key];
  return typeof field === "string" && field.length > 0 ? field : null;
}

export function parseRealtimeEvent(raw: string): unknown | null {
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return null;
  }
}

export function isDmEnvelopeDispatchedEvent(value: unknown): value is DmEnvelopeDispatchedEvent {
  if (!isRecord(value) || value.event_type !== "dm.envelope.dispatched") {
    return false;
  }

  const data = value.data;
  if (!isRecord(data)) {
    return false;
  }

  return [
    "envelope_id",
    "message_id",
    "thread_id",
    "sender_identity_id",
    "recipient_identity_id",
    "target_device_id",
    "ciphertext",
    "accepted_at",
    "dispatched_at",
    "delivery_cursor",
  ].every((key) => stringField(data, key) !== null) && data.transport_scope === "encrypted_envelope_server";
}

export function isDmEnvelopeAckEvent(value: unknown): value is DmEnvelopeAckEvent {
  if (!isRecord(value) || value.event_type !== "dm.envelope.ack") {
    return false;
  }

  const data = value.data;
  if (!isRecord(data)) {
    return false;
  }

  return [
    "envelope_id",
    "message_id",
    "thread_id",
    "recipient_identity_id",
    "device_id",
    "delivery_cursor",
    "received_at",
  ].every((key) => stringField(data, key) !== null) && data.ack_status === "received";
}

export function isDmDeviceVerifiedEvent(value: unknown): value is DmDeviceVerifiedEvent {
  if (!isRecord(value) || value.event_type !== "dm.device.verified") {
    return false;
  }

  const data = value.data;
  if (!isRecord(data)) {
    return false;
  }

  return stringField(data, "device_id") !== null && stringField(data, "verified_at") !== null;
}

export function envelopeFromDispatch(event: DmEnvelopeDispatchedEvent): StoredDmEnvelope {
  const { data } = event;
  return {
    envelopeId: data.envelope_id,
    messageId: data.message_id,
    threadId: data.thread_id,
    recipientIdentityId: data.recipient_identity_id,
    deviceId: data.target_device_id,
    deliveryCursor: data.delivery_cursor,
    ciphertext: data.ciphertext,
    senderIdentityId: data.sender_identity_id,
    acceptedAt: data.accepted_at,
    dispatchedAt: data.dispatched_at,
    storedAt: new Date().toISOString(),
  };
}

export function envelopeFromCatchUp(input: {
  item: DmFanoutCatchUpItem;
  recipientIdentityId: string;
  deviceId: string;
}): StoredDmEnvelope {
  return {
    envelopeId: input.item.envelope_id,
    messageId: input.item.message_id,
    threadId: input.item.thread_id,
    recipientIdentityId: input.recipientIdentityId,
    deviceId: input.deviceId,
    deliveryCursor: input.item.cursor,
    ciphertext: input.item.ciphertext,
    sourceDeviceId: input.item.source_device_id ?? null,
    storedAt: new Date().toISOString(),
  };
}

function storageKey(identityId: string, deviceId: string): string {
  return `${ENVELOPE_STORAGE_PREFIX}.${encodeURIComponent(identityId)}.${encodeURIComponent(deviceId)}`;
}

export function readStoredDmEnvelopes(
  identityId: string,
  deviceId: string,
  storage = browserStorage(),
): StoredDmEnvelope[] {
  if (!storage) {
    return [];
  }

  try {
    const parsed = JSON.parse(storage.getItem(storageKey(identityId, deviceId)) ?? "[]") as unknown;
    return Array.isArray(parsed) ? parsed.filter(isStoredEnvelope) : [];
  } catch {
    return [];
  }
}

function isStoredEnvelope(value: unknown): value is StoredDmEnvelope {
  if (!isRecord(value)) {
    return false;
  }

  return [
    "envelopeId",
    "messageId",
    "threadId",
    "recipientIdentityId",
    "deviceId",
    "deliveryCursor",
    "ciphertext",
    "storedAt",
  ].every((key) => stringField(value, key) !== null);
}

export function storeDmEnvelope(
  identityId: string,
  deviceId: string,
  envelope: StoredDmEnvelope,
  storage = browserStorage(),
): boolean {
  if (!storage || identityId !== envelope.recipientIdentityId || deviceId !== envelope.deviceId) {
    return false;
  }

  try {
    const existing = readStoredDmEnvelopes(identityId, deviceId, storage).filter(
      (item) => item.envelopeId !== envelope.envelopeId,
    );
    const next = [...existing, envelope]
      .sort((left, right) => Number(left.deliveryCursor) - Number(right.deliveryCursor))
      .slice(-MAX_STORED_ENVELOPES);
    storage.setItem(storageKey(identityId, deviceId), JSON.stringify(next));
    return true;
  } catch {
    return false;
  }
}

export async function catchUpAndAckDmEnvelopes(input: {
  identityId: string;
  deviceId: string;
  limit: number;
  maxPages: number;
  catchUp: () => Promise<DmFanoutCatchUpPayload | null>;
  storeEnvelope: (envelope: StoredDmEnvelope) => boolean;
  sendAck: (envelope: StoredDmEnvelope) => void;
  waitForAck: (envelopeId: string) => Promise<boolean>;
}): Promise<DmCatchUpResult> {
  let pages = 0;
  let stored = 0;
  let acked = 0;

  while (pages < input.maxPages) {
    const page = await input.catchUp();
    if (!page) {
      return { status: "failed", pages, stored, acked };
    }
    if (page.status !== "ready") {
      return { status: "blocked", pages, stored, acked };
    }
    if (page.items.length === 0) {
      return { status: "current", pages, stored, acked };
    }

    pages += 1;
    const ackWaits = page.items.map(async (item) => {
      const envelope = envelopeFromCatchUp({
        item,
        recipientIdentityId: input.identityId,
        deviceId: input.deviceId,
      });
      if (!input.storeEnvelope(envelope)) {
        return false;
      }

      stored += 1;
      const ackWait = input.waitForAck(envelope.envelopeId);
      input.sendAck(envelope);
      return ackWait;
    });

    const ackResults = await Promise.all(ackWaits);
    acked += ackResults.filter(Boolean).length;

    if (page.items.length < input.limit) {
      return { status: "current", pages, stored, acked };
    }
  }

  return { status: "max_pages", pages, stored, acked };
}

export function buildDmEnvelopeAck(
  source: DmEnvelopeAckSource,
  options?: { correlationId?: string; receivedAt?: string },
): {
  event_type: "dm.envelope.ack";
  correlation_id: string;
  data: {
    envelope_id: string;
    message_id: string;
    thread_id: string;
    recipient_identity_id: string;
    device_id: string;
    delivery_cursor: string;
    ack_status: "received";
    received_at: string;
  };
} {
  return {
    event_type: "dm.envelope.ack",
    correlation_id: options?.correlationId ?? createCorrelationId(),
    data: {
      envelope_id: source.envelopeId,
      message_id: source.messageId,
      thread_id: source.threadId,
      recipient_identity_id: source.recipientIdentityId,
      device_id: source.deviceId,
      delivery_cursor: source.deliveryCursor,
      ack_status: "received",
      received_at: options?.receivedAt ?? new Date().toISOString(),
    },
  };
}

export function buildDmDeviceProof(
  deviceId: string,
  deviceSecret: string,
  options?: { correlationId?: string },
): {
  event_type: "dm.device.proof";
  correlation_id: string;
  data: {
    device_id: string;
    device_secret: string;
  };
} {
  return {
    event_type: "dm.device.proof",
    correlation_id: options?.correlationId ?? createCorrelationId(),
    data: {
      device_id: deviceId,
      device_secret: deviceSecret,
    },
  };
}
