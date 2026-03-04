import { env } from "@/lib/env";

type ApiError = {
  code?: string;
  message?: string;
};

type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; code: string; message: string };

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
  const response = await fetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/identity/keys/register`, {
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
  const response = await fetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/auth/challenge`, {
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
  const response = await fetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/auth/verify`, {
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

  return parseResponse<{ session_id: string; expires_at: string }>(response);
}

export async function revokeSession(input: {
  sessionId: string;
}): Promise<ApiResult<undefined>> {
  const response = await fetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/auth/sessions/revoke`, {
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
  const response = await fetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/invites`, {
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

export async function redeemInvite(input: {
  token: string;
  nodeFingerprint: string;
}): Promise<ApiResult<{ accepted: boolean }>> {
  const response = await fetch(`${env.NEXT_PUBLIC_API_BASE_URL}/v1/invites/redeem`, {
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
