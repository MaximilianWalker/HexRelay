import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  acceptFriendRequest,
  createContactInvite,
  createFriendRequest,
  createInvite,
  declineFriendRequest,
  fetchContacts,
  fetchFriendRequests,
  fetchServers,
  issueAuthChallenge,
  redeemContactInvite,
  redeemInvite,
  registerIdentityKey,
  revokeSession,
  verifyAuthChallenge,
} from "./api";

describe("api auth transport", () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, "document", {
      configurable: true,
      value: {
        cookie: "hexrelay_csrf=csrf-123",
      },
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
    delete (globalThis as { document?: unknown }).document;
  });

  it("uses cookie credentials for contacts requests", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );

    const result = await fetchContacts({
      search: "nora",
      unreadOnly: true,
    });

    expect(result.ok).toBe(true);
    const [, init] = fetchMock.mock.calls[0] ?? [];
    expect(init?.credentials).toBe("include");
  });

  it("sends csrf header for session revoke", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(new Response(null, { status: 204 }));

    const result = await revokeSession({ sessionId: "sess-1" });

    expect(result.ok).toBe(true);
    const [, init] = fetchMock.mock.calls[0] ?? [];
    const headers = new Headers(init?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(headers.get("authorization")).toBeNull();
    expect(init?.body).toBe('{"session_id":"sess-1"}');
  });

  it("builds query parameters for server filters", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );

    await fetchServers({
      search: "atlas",
      favoritesOnly: true,
      unreadOnly: true,
      mutedOnly: true,
    });

    const [url] = fetchMock.mock.calls[0] ?? [];
    expect(String(url)).toContain("search=atlas");
    expect(String(url)).toContain("favorites_only=true");
    expect(String(url)).toContain("unread_only=true");
    expect(String(url)).toContain("muted_only=true");
  });

  it("sends content-type and csrf for friend request creation", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ request_id: "fr-1", status: "pending" }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );

    const result = await createFriendRequest({
      requesterIdentityId: "usr-a",
      targetIdentityId: "usr-b",
    });

    expect(result.ok).toBe(true);
    const [, init] = fetchMock.mock.calls[0] ?? [];
    const headers = new Headers(init?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(headers.get("content-type")).toBe("application/json");
  });

  it("returns fallback error payload for non-json error response", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response("failure", {
        status: 500,
        headers: { "content-type": "text/plain" },
      }),
    );

    const result = await fetchContacts({});

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.code).toBe("error");
      expect(result.message).toBe("Request failed");
    }
  });

  it("supports identity registration and auth challenge endpoints", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(new Response(null, { status: 201 }))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            challenge_id: "challenge-1",
            nonce: "nonce-1",
            expires_at: "2030-01-01T00:00:00Z",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const register = await registerIdentityKey({
      identityId: "usr-a",
      publicKey: "a".repeat(64),
    });
    const challenge = await issueAuthChallenge({ identityId: "usr-a" });

    expect(register.ok).toBe(true);
    expect(challenge.ok).toBe(true);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("supports auth verify and invite create/redeem endpoints", async () => {
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            session_id: "sess-1",
            expires_at: "2030-01-01T00:00:00Z",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ token: "inv-1", mode: "one_time", max_uses: 1 }),
          { status: 201, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ accepted: true }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      );

    const verify = await verifyAuthChallenge({
      identityId: "usr-a",
      challengeId: "challenge-1",
      signature: "b".repeat(128),
    });
    const invite = await createInvite({ mode: "one_time" });
    const redeem = await redeemInvite({ token: "inv-1", nodeFingerprint: "node-1" });

    expect(verify.ok).toBe(true);
    expect(invite.ok).toBe(true);
    expect(redeem.ok).toBe(true);
  });

  it("supports friend-request list and transitions", async () => {
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [] }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ request_id: "fr-1", status: "accepted" }), {
          status: 200,
          headers: { "content-type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(new Response(null, { status: 204 }));

    const list = await fetchFriendRequests({
      identityId: "usr-a",
      direction: "inbound",
    });
    const accept = await acceptFriendRequest({ requestId: "fr-1" });
    const decline = await declineFriendRequest({ requestId: "fr-2" });

    expect(list.ok).toBe(true);
    expect(accept.ok).toBe(true);
    expect(decline.ok).toBe(true);
  });

  it("sends csrf and correct URL for contact invite creation", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(
          JSON.stringify({
            invite_id: "ci-1",
            token: "contact-token-abc",
            mode: "one_time",
            created_at: "2026-03-20T00:00:00Z",
          }),
          { status: 201, headers: { "content-type": "application/json" } },
        ),
      );

    const result = await createContactInvite({ mode: "one_time" });

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.data.token).toBe("contact-token-abc");
      expect(result.data.invite_id).toBe("ci-1");
    }
    const [url, init] = fetchMock.mock.calls[0] ?? [];
    expect(String(url)).toContain("/v1/contact-invites");
    expect(String(url)).not.toContain("/redeem");
    const headers = new Headers(init?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(headers.get("content-type")).toBe("application/json");
    expect(init?.body).toContain('"mode":"one_time"');
  });

  it("sends csrf and correct URL for contact invite redeem", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(
          JSON.stringify({
            request_id: "fr-99",
            requester_identity_id: "usr-inviter",
            target_identity_id: "usr-redeemer",
            status: "pending",
            created_at: "2026-03-20T00:00:00Z",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const result = await redeemContactInvite({ token: "contact-token-abc" });

    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.data.request_id).toBe("fr-99");
      expect(result.data.status).toBe("pending");
      expect(result.data.requester_identity_id).toBe("usr-inviter");
    }
    const [url, init] = fetchMock.mock.calls[0] ?? [];
    expect(String(url)).toContain("/v1/contact-invites/redeem");
    const headers = new Headers(init?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(init?.body).toBe('{"token":"contact-token-abc"}');
  });

  it("returns error codes for failed contact invite redeem", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({ code: "invite_expired", message: "Invite has expired" }),
        { status: 400, headers: { "content-type": "application/json" } },
      ),
    );

    const result = await redeemContactInvite({ token: "expired-token" });

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.code).toBe("invite_expired");
      expect(result.message).toBe("Invite has expired");
    }
  });
});
