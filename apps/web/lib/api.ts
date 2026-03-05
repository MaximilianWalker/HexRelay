import { env } from "@/lib/env";

type ApiError = {
  code?: string;
  message?: string;
};

type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; code: string; message: string };

const CSRF_COOKIE = "hexrelay_csrf";

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

async function apiFetch(url: string, init?: RequestInit): Promise<Response> {
  const method = (init?.method ?? "GET").toUpperCase();
  const headers = new Headers(init?.headers ?? {});

  if (method !== "GET" && method !== "HEAD") {
    const csrf = readCookie(CSRF_COOKIE);
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
