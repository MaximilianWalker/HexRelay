"use client";

import { useEffect, useSyncExternalStore } from "react";

import { catchUpDmFanout, heartbeatDmProfileDevice } from "@/lib/api";
import {
  buildDmDeviceProof,
  buildDmEnvelopeAck,
  buildRealtimeWebSocketUrl,
  catchUpAndAckDmEnvelopes,
  envelopeFromDispatch,
  getOrCreateDmDeviceId,
  getOrCreateDmDeviceSecret,
  isDmDeviceVerifiedEvent,
  isDmEnvelopeAckEvent,
  isDmEnvelopeDispatchedEvent,
  parseRealtimeEvent,
  storeDmEnvelope,
  type StoredDmEnvelope,
} from "@/lib/dm-realtime";
import { env } from "@/lib/env";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";
import { subscribeWorkspacePreferences } from "@/lib/workspace-preferences";

const EMPTY_IDENTITY = "";
const ACK_WAIT_MS = 5_000;
const CATCH_UP_LIMIT = 100;
const MAX_CATCH_UP_PAGES = 25;
const RECONNECT_DELAY_MS = 5_000;
const HEARTBEAT_RETRY_DELAY_MS = 5_000;
const PROOF_RETRY_DELAY_MS = 5_000;

function readActiveIdentitySnapshot(): string {
  try {
    const personas = readPersonas();
    return readActivePersonaId() ?? personas[0]?.id ?? EMPTY_IDENTITY;
  } catch {
    return EMPTY_IDENTITY;
  }
}

function isSessionUsable(identityId: string): boolean {
  const session = getPersonaSession(identityId);
  if (!session) {
    return false;
  }

  const expiresAt = Date.parse(session.expiresAt);
  return Number.isNaN(expiresAt) || expiresAt > Date.now();
}

export function RealtimeClient() {
  const identityId = useSyncExternalStore(
    subscribeWorkspacePreferences,
    readActiveIdentitySnapshot,
    () => EMPTY_IDENTITY,
  );

  useEffect(() => {
    if (!identityId || !isSessionUsable(identityId)) {
      return;
    }

    const deviceId = getOrCreateDmDeviceId();
    const deviceSecret = getOrCreateDmDeviceSecret();
    if (!deviceId || !deviceSecret) {
      return;
    }
    const activeDeviceId = deviceId;
    const activeDeviceSecret = deviceSecret;

    let closed = false;
    let socket: WebSocket | null = null;
    let reconnectTimer: number | null = null;
    let heartbeatTimer: number | null = null;
    let proofRetryTimer: number | null = null;
    let deviceVerified = false;
    const pendingAcks = new Map<string, (acked: boolean) => void>();

    function clearProofRetry(): void {
      if (proofRetryTimer !== null) {
        window.clearTimeout(proofRetryTimer);
        proofRetryTimer = null;
      }
    }

    function scheduleReconnect(): void {
      if (closed || reconnectTimer !== null) {
        return;
      }

      reconnectTimer = window.setTimeout(() => {
        reconnectTimer = null;
        connect();
      }, RECONNECT_DELAY_MS);
    }

    function scheduleHeartbeatRetry(): void {
      if (closed || heartbeatTimer !== null) {
        return;
      }

      heartbeatTimer = window.setTimeout(() => {
        heartbeatTimer = null;
        startAfterHeartbeat();
      }, HEARTBEAT_RETRY_DELAY_MS);
    }

    function sendDeviceProof(targetSocket: WebSocket): void {
      if (closed || deviceVerified || targetSocket.readyState !== WebSocket.OPEN) {
        return;
      }

      targetSocket.send(JSON.stringify(buildDmDeviceProof(activeDeviceId, activeDeviceSecret)));
      clearProofRetry();
      proofRetryTimer = window.setTimeout(() => {
        proofRetryTimer = null;
        if (!closed && !deviceVerified && socket === targetSocket) {
          sendDeviceProof(targetSocket);
        }
      }, PROOF_RETRY_DELAY_MS);
    }

    function sendAck(source: StoredDmEnvelope): void {
      if (socket?.readyState !== WebSocket.OPEN) {
        return;
      }

      socket.send(JSON.stringify(buildDmEnvelopeAck(source)));
    }

    function acceptEnvelope(envelope: StoredDmEnvelope): void {
      if (envelope.recipientIdentityId !== identityId || envelope.deviceId !== activeDeviceId) {
        return;
      }

      if (storeDmEnvelope(identityId, activeDeviceId, envelope)) {
        sendAck(envelope);
      }
    }

    function waitForAck(envelopeId: string): Promise<boolean> {
      return new Promise((resolve) => {
        const timer = window.setTimeout(() => {
          pendingAcks.delete(envelopeId);
          resolve(false);
        }, ACK_WAIT_MS);

        pendingAcks.set(envelopeId, (acked) => {
          window.clearTimeout(timer);
          resolve(acked);
        });
      });
    }

    function resolveAck(envelopeId: string): void {
      const resolve = pendingAcks.get(envelopeId);
      if (!resolve) {
        return;
      }

      pendingAcks.delete(envelopeId);
      resolve(true);
    }

    async function catchUpAndAck(): Promise<void> {
      await catchUpAndAckDmEnvelopes({
        identityId,
        deviceId: activeDeviceId,
        limit: CATCH_UP_LIMIT,
        maxPages: MAX_CATCH_UP_PAGES,
        catchUp: async () => {
          const result = await catchUpDmFanout({
            deviceId: activeDeviceId,
            deviceSecret: activeDeviceSecret,
            limit: CATCH_UP_LIMIT,
          }).catch(() => null);
          return !closed && result?.ok ? result.data : null;
        },
        storeEnvelope: (envelope) => storeDmEnvelope(identityId, activeDeviceId, envelope),
        sendAck,
        waitForAck,
      });
    }

    function connect(): void {
      if (closed) {
        return;
      }

      if (socket?.readyState === WebSocket.OPEN || socket?.readyState === WebSocket.CONNECTING) {
        return;
      }

      const nextSocket = new WebSocket(buildRealtimeWebSocketUrl(env.NEXT_PUBLIC_REALTIME_WS_URL, activeDeviceId));
      socket = nextSocket;
      deviceVerified = false;

      nextSocket.addEventListener("open", () => {
        sendDeviceProof(nextSocket);
      });
      nextSocket.addEventListener("message", (event: MessageEvent) => {
        if (typeof event.data !== "string") {
          return;
        }

        const parsed = parseRealtimeEvent(event.data);
        if (isDmDeviceVerifiedEvent(parsed) && parsed.data.device_id === activeDeviceId) {
          deviceVerified = true;
          clearProofRetry();
          void catchUpAndAck();
          return;
        }

        if (isDmEnvelopeDispatchedEvent(parsed)) {
          acceptEnvelope(envelopeFromDispatch(parsed));
          return;
        }

        if (isDmEnvelopeAckEvent(parsed)) {
          resolveAck(parsed.data.envelope_id);
        }
      });
      nextSocket.addEventListener("close", () => {
        clearProofRetry();
        if (socket === nextSocket) {
          socket = null;
        }
        scheduleReconnect();
      });
    }

    function startAfterHeartbeat(): void {
      void heartbeatDmProfileDevice({ deviceId: activeDeviceId, deviceSecret: activeDeviceSecret, active: true })
        .catch(() => null)
        .then((result) => {
          if (closed) {
            return;
          }

          if (result?.ok) {
            connect();
            return;
          }

          scheduleHeartbeatRetry();
        });
    }

    startAfterHeartbeat();

    return () => {
      closed = true;
      if (heartbeatTimer !== null) {
        window.clearTimeout(heartbeatTimer);
      }
      if (reconnectTimer !== null) {
        window.clearTimeout(reconnectTimer);
      }
      clearProofRetry();
      pendingAcks.forEach((resolve) => resolve(false));
      pendingAcks.clear();
      socket?.close();
    };
  }, [identityId]);

  return null;
}
