import { describe, expect, it } from "vitest";

import {
  buildDmDeviceProof,
  buildDmEnvelopeAck,
  buildRealtimeWebSocketUrl,
  catchUpAndAckDmEnvelopes,
  envelopeFromCatchUp,
  envelopeFromDispatch,
  getOrCreateDmDeviceId,
  getOrCreateDmDeviceSecret,
  isDmDeviceVerifiedEvent,
  isDmEnvelopeDispatchedEvent,
  parseRealtimeEvent,
  readStoredDmEnvelopes,
  storeDmEnvelope,
} from "./dm-realtime";

class MemoryStorage implements Storage {
  private values = new Map<string, string>();

  get length(): number {
    return this.values.size;
  }

  clear(): void {
    this.values.clear();
  }

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  key(index: number): string | null {
    return Array.from(this.values.keys())[index] ?? null;
  }

  removeItem(key: string): void {
    this.values.delete(key);
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

const dispatchEvent = {
  event_type: "dm.envelope.dispatched",
  correlation_id: "corr-1",
  data: {
    envelope_id: "dm-env-1",
    message_id: "msg-1",
    thread_id: "dm-usr-a-usr-b",
    sender_identity_id: "usr-a",
    recipient_identity_id: "usr-b",
    target_device_id: "web-main",
    ciphertext: "enc:payload",
    accepted_at: "2026-05-08T15:00:00Z",
    dispatched_at: "2026-05-08T15:00:01Z",
    delivery_cursor: "7",
    transport_scope: "encrypted_envelope_server",
  },
};

describe("dm realtime helpers", () => {
  it("creates and reuses browser-safe DM device ids", () => {
    const storage = new MemoryStorage();

    const created = getOrCreateDmDeviceId(storage);
    const reused = getOrCreateDmDeviceId(storage);

    expect(created).toMatch(/^web-[A-Za-z0-9_-]+$/);
    expect(reused).toBe(created);
  });

  it("creates and reuses browser-safe DM device secrets", () => {
    const storage = new MemoryStorage();

    const created = getOrCreateDmDeviceSecret(storage);
    const reused = getOrCreateDmDeviceSecret(storage);

    expect(created).toMatch(/^secret-[A-Fa-f0-9]{64}$/);
    expect(reused).toBe(created);
  });

  it("replaces invalid stored device ids", () => {
    const storage = new MemoryStorage();
    storage.setItem("hexrelay.dm.device-id", "bad/device");

    const created = getOrCreateDmDeviceId(storage);

    expect(created).toMatch(/^web-[A-Za-z0-9_-]+$/);
    expect(created).not.toBe("bad/device");
  });

  it("adds only the browser device id query parameter to websocket URLs", () => {
    expect(buildRealtimeWebSocketUrl("ws://127.0.0.1:8081/ws?existing=1", "web-main")).toBe(
      "ws://127.0.0.1:8081/ws?existing=1&device_id=web-main",
    );
  });

  it("builds device proof events without putting secrets in URLs", () => {
    expect(buildDmDeviceProof("web-main", "secret-web-main", { correlationId: "corr-proof" })).toEqual({
      event_type: "dm.device.proof",
      correlation_id: "corr-proof",
      data: {
        device_id: "web-main",
        device_secret: "secret-web-main",
      },
    });
  });

  it("recognizes runtime DM envelope dispatch events", () => {
    const parsed = parseRealtimeEvent(JSON.stringify(dispatchEvent));

    expect(isDmEnvelopeDispatchedEvent(parsed)).toBe(true);
    expect(isDmEnvelopeDispatchedEvent({ ...dispatchEvent, event_type: "presence.updated" })).toBe(false);
  });

  it("recognizes runtime DM device verification events", () => {
    expect(
      isDmDeviceVerifiedEvent({
        event_type: "dm.device.verified",
        data: { device_id: "web-main", verified_at: "2026-05-08T15:00:01Z" },
      }),
    ).toBe(true);
  });

  it("stores ciphertext envelopes before ack construction", () => {
    const storage = new MemoryStorage();
    const parsed = parseRealtimeEvent(JSON.stringify(dispatchEvent));

    expect(isDmEnvelopeDispatchedEvent(parsed)).toBe(true);
    if (!isDmEnvelopeDispatchedEvent(parsed)) {
      return;
    }

    const envelope = envelopeFromDispatch(parsed);
    const stored = storeDmEnvelope("usr-b", "web-main", envelope, storage);
    const ack = buildDmEnvelopeAck(envelope, {
      correlationId: "corr-ack",
      receivedAt: "2026-05-08T15:00:02Z",
    });

    expect(stored).toBe(true);
    expect(readStoredDmEnvelopes("usr-b", "web-main", storage)).toMatchObject([
      {
        envelopeId: "dm-env-1",
        ciphertext: "enc:payload",
      },
    ]);
    expect(ack).toEqual({
      event_type: "dm.envelope.ack",
      correlation_id: "corr-ack",
      data: {
        envelope_id: "dm-env-1",
        message_id: "msg-1",
        thread_id: "dm-usr-a-usr-b",
        recipient_identity_id: "usr-b",
        device_id: "web-main",
        delivery_cursor: "7",
        ack_status: "received",
        received_at: "2026-05-08T15:00:02Z",
      },
    });
  });

  it("converts catch-up items into ackable stored envelopes", () => {
    const envelope = envelopeFromCatchUp({
      item: {
        envelope_id: "dm-env-2",
        cursor: "8",
        thread_id: "dm-usr-a-usr-b",
        message_id: "msg-2",
        ciphertext: "enc:catch-up",
        source_device_id: "sender-main",
      },
      recipientIdentityId: "usr-b",
      deviceId: "web-main",
    });

    expect(envelope).toMatchObject({
      envelopeId: "dm-env-2",
      deliveryCursor: "8",
      sourceDeviceId: "sender-main",
      recipientIdentityId: "usr-b",
      deviceId: "web-main",
    });
  });

  it("continues catch-up after a full page is stored and acked", async () => {
    const sentAcks: string[] = [];
    const stored: string[] = [];
    const page = (start: number, count: number) =>
      Array.from({ length: count }, (_, index) => {
        const cursor = start + index;
        return {
          envelope_id: `dm-env-${cursor}`,
          cursor: String(cursor),
          thread_id: "dm-usr-a-usr-b",
          message_id: `msg-${cursor}`,
          ciphertext: `enc:${cursor}`,
          source_device_id: null,
        };
      });
    const pages = [page(1, 100), page(101, 20)];

    const result = await catchUpAndAckDmEnvelopes({
      identityId: "usr-b",
      deviceId: "web-main",
      limit: 100,
      maxPages: 5,
      catchUp: async () => {
        const items = pages.shift() ?? [];
        return {
          status: "ready",
          replay_count: items.length,
          items,
        };
      },
      storeEnvelope: (envelope) => {
        stored.push(envelope.envelopeId);
        return true;
      },
      sendAck: (envelope) => {
        sentAcks.push(envelope.envelopeId);
      },
      waitForAck: async () => true,
    });

    expect(result).toEqual({
      status: "current",
      pages: 2,
      stored: 120,
      acked: 120,
    });
    expect(stored).toHaveLength(120);
    expect(sentAcks).toHaveLength(120);
  });
});
