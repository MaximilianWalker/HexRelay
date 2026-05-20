import { env } from "@/lib/env";

type ApiError = {
  code?: string;
  message?: string;
};

type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; code: string; message: string };

export type DmInboundPolicy = "friends_only" | "same_server" | "anyone";

export type DmPolicyResponse = {
  inbound_policy: DmInboundPolicy;
  offline_delivery_mode: string;
};

export type DmProfileDeviceHeartbeatResponse = {
  identity_id: string;
  devices: Array<{
    device_id: string;
    active: boolean;
    last_seen_at: string;
  }>;
};

export type DmFanoutCatchUpResponse = {
  status: "ready" | "blocked";
  reason_code: string;
  transport_profile: "encrypted_envelope_server";
  device_id: string;
  replay_count: number;
  next_cursor: string;
  deduped_message_ids: string[];
  items: Array<{
    envelope_id: string;
    cursor: string;
    thread_id: string;
    message_id: string;
    ciphertext: string;
    source_device_id?: string | null;
  }>;
};

export type TestingProfileSummary = {
  profile_id: string;
  identity_id: string;
  purpose: string;
};

export type TestingSessionResponse = {
  profile_id: string;
  identity_id: string;
  session_id: string;
  expires_at: string;
  csrf_token: string;
};

export type ServerSummary = {
  id: string;
  name: string;
  unread: number;
  favorite: boolean;
  muted: boolean;
};

export type ServerChannelSummary = {
  id: string;
  name: string;
  kind: string;
  last_message_seq: number;
};

export type ServerChannelMessage = {
  message_id: string;
  channel_id: string;
  author_id: string;
  channel_seq: number;
  content: string;
  reply_to_message_id?: string | null;
  mentions: string[];
  created_at: string;
  edited_at?: string | null;
  deleted_at?: string | null;
};

export type ServerChannelMessagePage = {
  items: ServerChannelMessage[];
  next_cursor?: string | null;
};

const CSRF_COOKIE = "hexrelay_csrf";
const CSRF_STORAGE_KEY = "hexrelay.csrf.runtime";

function readCookie(name: string): string | null {
  if (typeof document === "undefined") {
    return null;
  }

  const pairs = document.cookie.split(";");
  for (const pair of pairs) {
    const [cookieName, ...rest] = pair.trim().split("=");
    if (cookieName === name) {
      return rest.join("=") || null;
    }
  }

  return null;
}

function readStoredCsrfToken(): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.sessionStorage.getItem(CSRF_STORAGE_KEY);
  } catch {
    return null;
  }
}

function readCsrfToken(): string | null {
  return readCookie(CSRF_COOKIE) ?? readStoredCsrfToken();
}

export function storeCsrfToken(token: string): void {
  if (typeof window === "undefined") {
    return;
  }

  const trimmed = token.trim();

  try {
    if (trimmed) {
      window.sessionStorage.setItem(CSRF_STORAGE_KEY, trimmed);
      return;
    }

    window.sessionStorage.removeItem(CSRF_STORAGE_KEY);
  } catch {
    // Cookie-based CSRF still works when browser storage is unavailable.
  }
}

async function apiFetch(url: string, init?: RequestInit): Promise<Response> {
  const method = (init?.method ?? "GET").toUpperCase();
  const headers = new Headers(init?.headers ?? {});

  if (method !== "GET" && method !== "HEAD") {
    const csrf = readCsrfToken();
    if (csrf) {
      headers.set("x-csrf-token", csrf);
    }
  }

  return fetch(url, {
    ...init,
    headers,
    credentials: "include",
  });
}

async function parseResponse<T>(response: Response): Promise<ApiResult<T>> {
  if (response.ok) {
    if (response.status === 204) {
      return { ok: true, data: undefined as T };
    }

    const payload = (await response.json().catch(() => null)) as T | null;
    return { ok: true, data: (payload ?? ({} as T)) as T };
  }

  const payload = (await response.json().catch(() => null)) as ApiError | null;
  return {
    ok: false,
    code: payload?.code ?? "error",
    message: payload?.message ?? "Request failed",
  };
}

export async function registerIdentityKey(input: {
  identityId: string;
  publicKey: string;
}): Promise<ApiResult<undefined>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/identity/keys/register`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      identity_id: input.identityId,
      public_key: input.publicKey,
      algorithm: "ed25519",
    }),
  });

  return parseResponse<undefined>(response);
}

export async function issueAuthChallenge(input: {
  identityId: string;
}): Promise<ApiResult<{ challenge_id: string; nonce: string; expires_at: string }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/auth/challenge`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      identity_id: input.identityId,
    }),
  });

  return parseResponse<{ challenge_id: string; nonce: string; expires_at: string }>(response);
}

export async function verifyAuthChallenge(input: {
  identityId: string;
  challengeId: string;
  signature: string;
}): Promise<ApiResult<{ session_id: string; expires_at: string }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/auth/verify`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      identity_id: input.identityId,
      challenge_id: input.challengeId,
      signature: input.signature,
    }),
  });

  return parseResponse<{ session_id: string; expires_at: string }>(
    response,
  );
}

export async function fetchTestingProfiles(): Promise<
  ApiResult<{ items: TestingProfileSummary[] }>
> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/dev/testing/profiles`, {
    method: "GET",
  });

  return parseResponse(response);
}

export async function activateTestingSession(input: {
  profileId: string;
}): Promise<ApiResult<TestingSessionResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/dev/testing/sessions`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      profile_id: input.profileId,
    }),
  });

  return parseResponse(response);
}

export async function revokeSession(input: {
  sessionId: string;
}): Promise<ApiResult<undefined>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/auth/sessions/revoke`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      session_id: input.sessionId,
    }),
  });

  return parseResponse<undefined>(response);
}

export async function createInvite(input: {
  mode: "one_time" | "multi_use";
  maxUses?: number;
  expiresAt?: string;
}): Promise<ApiResult<{ token: string; mode: string; max_uses?: number; expires_at?: string }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/invites`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      mode: input.mode,
      max_uses: input.maxUses,
      expires_at: input.expiresAt,
    }),
  });

  return parseResponse<{ token: string; mode: string; max_uses?: number; expires_at?: string }>(
    response,
  );
}

export async function fetchServers(input: {
  search?: string;
  favoritesOnly?: boolean;
  unreadOnly?: boolean;
  mutedOnly?: boolean;
}): Promise<
  ApiResult<{
    items: ServerSummary[];
  }>
> {
  const params = new URLSearchParams();
  if (input.search) {
    params.set("search", input.search);
  }
  if (input.favoritesOnly) {
    params.set("favorites_only", "true");
  }
  if (input.unreadOnly) {
    params.set("unread_only", "true");
  }
  if (input.mutedOnly) {
    params.set("muted_only", "true");
  }

  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/servers?${params.toString()}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function fetchServer(input: {
  serverId: string;
}): Promise<ApiResult<{ item: ServerSummary }>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/servers/${encodeURIComponent(input.serverId)}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function fetchServerChannels(input: {
  serverId: string;
}): Promise<ApiResult<{ items: ServerChannelSummary[] }>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/servers/${encodeURIComponent(input.serverId)}/channels`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function fetchServerChannelMessages(input: {
  serverId: string;
  channelId: string;
  cursor?: string;
  limit?: number;
}): Promise<ApiResult<ServerChannelMessagePage>> {
  const params = new URLSearchParams();
  if (input.cursor) {
    params.set("cursor", input.cursor);
  }
  if (input.limit !== undefined) {
    params.set("limit", String(input.limit));
  }

  const query = params.toString();
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/servers/${encodeURIComponent(
      input.serverId,
    )}/channels/${encodeURIComponent(input.channelId)}/messages${query ? `?${query}` : ""}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function createServerChannelMessage(input: {
  serverId: string;
  channelId: string;
  content: string;
  replyToMessageId?: string | null;
  mentionIdentityIds?: string[];
}): Promise<ApiResult<ServerChannelMessage>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/servers/${encodeURIComponent(
      input.serverId,
    )}/channels/${encodeURIComponent(input.channelId)}/messages`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify({
        content: input.content,
        reply_to_message_id: input.replyToMessageId,
        mention_identity_ids: input.mentionIdentityIds ?? [],
      }),
    },
  );

  return parseResponse(response);
}

export async function fetchContacts(input: {
  search?: string;
  onlineOnly?: boolean;
  unreadOnly?: boolean;
  favoritesOnly?: boolean;
}): Promise<
  ApiResult<{
    items: Array<{
      id: string;
      name: string;
      status: string;
      unread: number;
      favorite: boolean;
      inbound_request: boolean;
      pending_request: boolean;
    }>;
  }>
> {
  const params = new URLSearchParams();
  if (input.search) {
    params.set("search", input.search);
  }
  if (input.onlineOnly) {
    params.set("online_only", "true");
  }
  if (input.unreadOnly) {
    params.set("unread_only", "true");
  }
  if (input.favoritesOnly) {
    params.set("favorites_only", "true");
  }

  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/contacts?${params.toString()}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function createFriendRequest(input: {
  requesterIdentityId: string;
  targetIdentityId: string;
}): Promise<ApiResult<{ request_id: string; status: string }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/friends/requests`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      requester_identity_id: input.requesterIdentityId,
      target_identity_id: input.targetIdentityId,
    }),
  });

  return parseResponse(response);
}

export async function fetchFriendRequests(input: {
  identityId: string;
  direction?: "inbound" | "outbound";
}): Promise<
  ApiResult<{
    items: Array<{
      request_id: string;
      requester_identity_id: string;
      target_identity_id: string;
      status: string;
      created_at: string;
    }>;
  }>
> {
  const params = new URLSearchParams();
  params.set("identity_id", input.identityId);
  if (input.direction) {
    params.set("direction", input.direction);
  }

  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/friends/requests?${params.toString()}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function acceptFriendRequest(input: {
  requestId: string;
}): Promise<ApiResult<{ request_id: string; status: string }>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/friends/requests/${input.requestId}/accept`,
    { method: "POST" },
  );

  return parseResponse(response);
}

export async function declineFriendRequest(input: {
  requestId: string;
}): Promise<ApiResult<undefined>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/friends/requests/${input.requestId}/decline`,
    { method: "POST" },
  );

  return parseResponse(response);
}

export async function fetchDmPolicy(): Promise<ApiResult<DmPolicyResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/dm/privacy-policy`, {
    method: "GET",
  });

  return parseResponse(response);
}

export async function updateDmPolicy(input: {
  inboundPolicy: DmInboundPolicy;
}): Promise<ApiResult<DmPolicyResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/dm/privacy-policy`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      inbound_policy: input.inboundPolicy,
    }),
  });

  return parseResponse(response);
}

export async function heartbeatDmProfileDevice(input: {
  deviceId: string;
  deviceSecret: string;
  active: boolean;
}): Promise<ApiResult<DmProfileDeviceHeartbeatResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/dm/profile-devices/heartbeat`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      device_id: input.deviceId,
      device_secret: input.deviceSecret,
      active: input.active,
    }),
  });

  return parseResponse(response);
}

export async function catchUpDmFanout(input: {
  deviceId: string;
  deviceSecret: string;
  cursor?: string;
  limit?: number;
}): Promise<ApiResult<DmFanoutCatchUpResponse>> {
  const body: { device_id: string; device_secret: string; cursor?: string; limit?: number } = {
    device_id: input.deviceId,
    device_secret: input.deviceSecret,
  };
  if (input.cursor) {
    body.cursor = input.cursor;
  }
  if (input.limit !== undefined) {
    body.limit = input.limit;
  }

  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/dm/fanout/catch-up`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });

  return parseResponse(response);
}

export async function redeemInvite(input: {
  token: string;
  serverId: string;
}): Promise<ApiResult<{ accepted: boolean }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/invites/redeem`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      token: input.token,
      server_id: input.serverId,
    }),
  });

  return parseResponse<{ accepted: boolean }>(response);
}

export async function createContactInvite(input: {
  mode: "one_time" | "multi_use";
  maxUses?: number;
  expiresAt?: string;
}): Promise<
  ApiResult<{
    invite_id: string;
    token: string;
    mode: string;
    expires_at?: string;
    max_uses?: number;
    created_at: string;
  }>
> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/contact-invites`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      mode: input.mode,
      max_uses: input.maxUses,
      expires_at: input.expiresAt,
    }),
  });

  return parseResponse(response);
}

export async function redeemContactInvite(input: {
  token: string;
}): Promise<
  ApiResult<{
    request_id: string;
    requester_identity_id: string;
    target_identity_id: string;
    status: string;
    created_at: string;
  }>
> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/contact-invites/redeem`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      token: input.token,
    }),
  });

  return parseResponse(response);
}
