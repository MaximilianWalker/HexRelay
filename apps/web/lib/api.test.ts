import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  acceptFriendRequest,
  activateTestingSession,
  createContactInvite,
  createFriendRequest,
  createInvite,
  declineFriendRequest,
  fetchDmPolicy,
  fetchContacts,
  fetchFriendRequests,
  fetchServers,
  fetchTestingProfiles,
  issueAuthChallenge,
  redeemContactInvite,
  redeemInvite,
  registerIdentityKey,
  revokeSession,
  storeCsrfToken,
  updateDmPolicy,
  verifyAuthChallenge,
} from "./api";

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
    delete (globalThis as { window?: unknown }).window;
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

  it("supports dev testing profile endpoints", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [
              {
                profile_id: "alice.primary",
                identity_id: "usr-test-alice",
                purpose: "Happy path",
              },
            ],
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            profile_id: "alice.primary",
            identity_id: "usr-test-alice",
            session_id: "sess-test-alice-primary",
            expires_at: "2026-06-04T00:00:00Z",
            csrf_token: "csrf-dev",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const profiles = await fetchTestingProfiles();
    const session = await activateTestingSession({ profileId: "alice.primary" });

    expect(profiles.ok).toBe(true);
    expect(session.ok).toBe(true);
    const [listUrl, listInit] = fetchMock.mock.calls[0] ?? [];
    expect(String(listUrl)).toContain("/v1/dev/testing/profiles");
    expect(listInit?.method).toBe("GET");
    const [activateUrl, activateInit] = fetchMock.mock.calls[1] ?? [];
    expect(String(activateUrl)).toContain("/v1/dev/testing/sessions");
    expect(activateInit?.method).toBe("POST");
    const headers = new Headers(activateInit?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(activateInit?.body).toBe('{"profile_id":"alice.primary"}');
  });

  it("uses the stored csrf fallback for cross-host local dev cookies", async () => {
    Object.defineProperty(globalThis, "document", {
      configurable: true,
      value: {
        cookie: "",
      },
    });
    (globalThis as { window?: unknown }).window = {
      sessionStorage: new MemoryStorage(),
    };
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          inbound_policy: "same_server",
          offline_delivery_mode: "manual_retry",
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );

    storeCsrfToken("csrf-dev");
    const result = await updateDmPolicy({ inboundPolicy: "same_server" });

    expect(result.ok).toBe(true);
    const [, init] = fetchMock.mock.calls[0] ?? [];
    const headers = new Headers(init?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-dev");
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

  it("loads and updates the DM privacy policy", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            inbound_policy: "friends_only",
            offline_delivery_mode: "best_effort_online",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            inbound_policy: "same_server",
            offline_delivery_mode: "best_effort_online",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const loaded = await fetchDmPolicy();
    const updated = await updateDmPolicy({ inboundPolicy: "same_server" });

    expect(loaded.ok).toBe(true);
    expect(updated.ok).toBe(true);
    const [getUrl, getInit] = fetchMock.mock.calls[0] ?? [];
    expect(String(getUrl)).toContain("/v1/dm/privacy-policy");
    expect(getInit?.method).toBe("GET");
    const [postUrl, postInit] = fetchMock.mock.calls[1] ?? [];
    expect(String(postUrl)).toContain("/v1/dm/privacy-policy");
    expect(postInit?.method).toBe("POST");
    const headers = new Headers(postInit?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(headers.get("content-type")).toBe("application/json");
    expect(postInit?.body).toBe('{"inbound_policy":"same_server"}');
  });
});
