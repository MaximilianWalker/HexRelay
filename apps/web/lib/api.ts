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

const CSRF_COOKIE = "hexrelay_csrf";
const CSRF_STORAGE_KEY = "hexrelay.csrf.runtime.v1";

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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/identity/keys/register`, {
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/auth/challenge`, {
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/auth/verify`, {
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/dev/testing/profiles`, {
    method: "GET",
  });

  return parseResponse(response);
}

export async function activateTestingSession(input: {
  profileId: string;
}): Promise<ApiResult<TestingSessionResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/dev/testing/sessions`, {
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/auth/sessions/revoke`, {
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/invites`, {
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
    items: Array<{
      id: string;
      name: string;
      unread: number;
      favorite: boolean;
      muted: boolean;
    }>;
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
    `${env.NEXT_PUBLIC_API_BASE_URL}/v1/servers?${params.toString()}`,
    { method: "GET" },
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
    `${env.NEXT_PUBLIC_API_BASE_URL}/v1/contacts?${params.toString()}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function createFriendRequest(input: {
  requesterIdentityId: string;
  targetIdentityId: string;
}): Promise<ApiResult<{ request_id: string; status: string }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/friends/requests`, {
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
    `${env.NEXT_PUBLIC_API_BASE_URL}/v1/friends/requests?${params.toString()}`,
    { method: "GET" },
  );

  return parseResponse(response);
}

export async function acceptFriendRequest(input: {
  requestId: string;
}): Promise<ApiResult<{ request_id: string; status: string }>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/v1/friends/requests/${input.requestId}/accept`,
    { method: "POST" },
  );

  return parseResponse(response);
}

export async function declineFriendRequest(input: {
  requestId: string;
}): Promise<ApiResult<undefined>> {
  const response = await apiFetch(
    `${env.NEXT_PUBLIC_API_BASE_URL}/v1/friends/requests/${input.requestId}/decline`,
    { method: "POST" },
  );

  return parseResponse(response);
}

export async function fetchDmPolicy(): Promise<ApiResult<DmPolicyResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/dm/privacy-policy`, {
    method: "GET",
  });

  return parseResponse(response);
}

export async function updateDmPolicy(input: {
  inboundPolicy: DmInboundPolicy;
}): Promise<ApiResult<DmPolicyResponse>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/dm/privacy-policy`, {
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

export async function redeemInvite(input: {
  token: string;
  nodeFingerprint: string;
}): Promise<ApiResult<{ accepted: boolean }>> {
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/invites/redeem`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify({
      token: input.token,
      node_fingerprint: input.nodeFingerprint,
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/contact-invites`, {
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
  const response = await apiFetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/contact-invites/redeem`, {
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
