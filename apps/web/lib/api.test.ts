import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  acceptFriendRequest,
  activateTestingSession,
  createContactInvite,
  createFriendRequest,
  createInvite,
  createServerChannelMessage,
  catchUpDmFanout,
  declineFriendRequest,
  fetchDmPolicy,
  fetchContacts,
  fetchFriendRequests,
  fetchServer,
  fetchServerChannelMessages,
  fetchServerChannels,
  fetchServers,
  fetchTestingProfiles,
  issueAuthChallenge,
  heartbeatDmProfileDevice,
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

  it("supports server workspace detail, channels, and message history endpoints", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            item: {
              id: "fixture-server-atlas",
              name: "Atlas Test Server",
              unread: 2,
              favorite: true,
              muted: false,
            },
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [
              {
                id: "fixture-channel-atlas-general",
                name: "general",
                kind: "text",
                last_message_seq: 3,
              },
            ],
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [
              {
                message_id: "fixture-server-message-general-003",
                channel_id: "fixture-channel-atlas-general",
                author_id: "usr-test-carol",
                channel_seq: 3,
                content: "Reply confirmed, Bob.",
                reply_to_message_id: "fixture-server-message-general-002",
                mentions: ["usr-test-bob"],
                created_at: "2026-05-04T11:12:00Z",
                edited_at: null,
                deleted_at: null,
              },
            ],
            next_cursor: "3",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const server = await fetchServer({ serverId: "fixture-server-atlas" });
    const channels = await fetchServerChannels({ serverId: "fixture-server-atlas" });
    const messages = await fetchServerChannelMessages({
      serverId: "fixture-server-atlas",
      channelId: "fixture-channel-atlas-general",
      cursor: "3",
      limit: 10,
    });

    expect(server.ok).toBe(true);
    expect(channels.ok).toBe(true);
    expect(messages.ok).toBe(true);
    const [serverUrl, serverInit] = fetchMock.mock.calls[0] ?? [];
    const [channelsUrl, channelsInit] = fetchMock.mock.calls[1] ?? [];
    const [messagesUrl, messagesInit] = fetchMock.mock.calls[2] ?? [];
    expect(String(serverUrl)).toContain("/servers/fixture-server-atlas");
    expect(String(channelsUrl)).toContain("/servers/fixture-server-atlas/channels");
    expect(String(messagesUrl)).toContain(
      "/servers/fixture-server-atlas/channels/fixture-channel-atlas-general/messages",
    );
    expect(String(messagesUrl)).toContain("cursor=3");
    expect(String(messagesUrl)).toContain("limit=10");
    expect(serverInit?.method).toBe("GET");
    expect(channelsInit?.method).toBe("GET");
    expect(messagesInit?.method).toBe("GET");
  });

  it("sends csrf and message payload for server channel message creation", async () => {
    const fetchMock = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        JSON.stringify({
          message_id: "scm-created",
          channel_id: "fixture-channel-atlas-general",
          author_id: "usr-test-alice",
          channel_seq: 4,
          content: "Checking in with Bob.",
          reply_to_message_id: "fixture-server-message-general-003",
          mentions: ["usr-test-bob"],
          created_at: "2026-05-04T11:13:00Z",
          edited_at: null,
          deleted_at: null,
        }),
        { status: 201, headers: { "content-type": "application/json" } },
      ),
    );

    const result = await createServerChannelMessage({
      serverId: "fixture-server-atlas",
      channelId: "fixture-channel-atlas-general",
      content: "Checking in with Bob.",
      replyToMessageId: "fixture-server-message-general-003",
      mentionIdentityIds: ["usr-test-bob"],
    });

    expect(result.ok).toBe(true);
    const [url, init] = fetchMock.mock.calls[0] ?? [];
    expect(String(url)).toContain(
      "/servers/fixture-server-atlas/channels/fixture-channel-atlas-general/messages",
    );
    expect(init?.method).toBe("POST");
    const headers = new Headers(init?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(headers.get("content-type")).toBe("application/json");
    expect(init?.body).toBe(
      '{"content":"Checking in with Bob.","reply_to_message_id":"fixture-server-message-general-003","mention_identity_ids":["usr-test-bob"]}',
    );
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
    expect(String(listUrl)).toContain("/dev/testing/profiles");
    expect(listInit?.method).toBe("GET");
    const [activateUrl, activateInit] = fetchMock.mock.calls[1] ?? [];
    expect(String(activateUrl)).toContain("/dev/testing/sessions");
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
          offline_delivery_mode: "encrypted_envelope_catchup",
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
    expect(String(url)).toContain("/contact-invites");
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
    expect(String(url)).toContain("/contact-invites/redeem");
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
            offline_delivery_mode: "encrypted_envelope_catchup",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            inbound_policy: "same_server",
            offline_delivery_mode: "encrypted_envelope_catchup",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const loaded = await fetchDmPolicy();
    const updated = await updateDmPolicy({ inboundPolicy: "same_server" });

    expect(loaded.ok).toBe(true);
    expect(updated.ok).toBe(true);
    const [getUrl, getInit] = fetchMock.mock.calls[0] ?? [];
    expect(String(getUrl)).toContain("/dm/privacy-policy");
    expect(getInit?.method).toBe("GET");
    const [postUrl, postInit] = fetchMock.mock.calls[1] ?? [];
    expect(String(postUrl)).toContain("/dm/privacy-policy");
    expect(postInit?.method).toBe("POST");
    const headers = new Headers(postInit?.headers ?? {});
    expect(headers.get("x-csrf-token")).toBe("csrf-123");
    expect(headers.get("content-type")).toBe("application/json");
    expect(postInit?.body).toBe('{"inbound_policy":"same_server"}');
  });

  it("supports DM profile-device heartbeat and catch-up endpoints", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            identity_id: "usr-test-alice",
            devices: [
              {
                device_id: "web-main",
                active: true,
                last_seen_at: "2026-05-08T15:00:00Z",
              },
            ],
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: "ready",
            reason_code: "fanout_catch_up_ok",
            transport_profile: "encrypted_envelope_node",
            device_id: "web-main",
            replay_count: 1,
            next_cursor: "7",
            deduped_message_ids: [],
            items: [
              {
                envelope_id: "dm-env-1",
                cursor: "7",
                thread_id: "dm-usr-a-usr-b",
                message_id: "msg-1",
                ciphertext: "enc:payload",
                source_device_id: null,
              },
            ],
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      );

    const heartbeat = await heartbeatDmProfileDevice({
      deviceId: "web-main",
      deviceSecret: "secret-web-main",
      active: true,
    });
    const catchUp = await catchUpDmFanout({
      deviceId: "web-main",
      deviceSecret: "secret-web-main",
      cursor: "6",
      limit: 10,
    });

    expect(heartbeat.ok).toBe(true);
    expect(catchUp.ok).toBe(true);
    const [heartbeatUrl, heartbeatInit] = fetchMock.mock.calls[0] ?? [];
    expect(String(heartbeatUrl)).toContain("/dm/profile-devices/heartbeat");
    expect(heartbeatInit?.body).toBe('{"device_id":"web-main","device_secret":"secret-web-main","active":true}');
    const heartbeatHeaders = new Headers(heartbeatInit?.headers ?? {});
    expect(heartbeatHeaders.get("x-csrf-token")).toBe("csrf-123");

    const [catchUpUrl, catchUpInit] = fetchMock.mock.calls[1] ?? [];
    expect(String(catchUpUrl)).toContain("/dm/fanout/catch-up");
    expect(catchUpInit?.body).toBe(
      '{"device_id":"web-main","device_secret":"secret-web-main","cursor":"6","limit":10}',
    );
    const catchUpHeaders = new Headers(catchUpInit?.headers ?? {});
    expect(catchUpHeaders.get("x-csrf-token")).toBe("csrf-123");
  });
});
