"use client";

import { useEffect, useMemo, useState } from "react";
import { QRCodeSVG } from "qrcode.react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  acceptFriendRequest,
  createContactInvite,
  createFriendRequest,
  declineFriendRequest,
  fetchContacts,
  fetchFriendRequests,
  redeemContactInvite,
} from "@/lib/api";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";

import styles from "../surfaces.module.css";

type Contact = {
  id: string;
  name: string;
  status: "online" | "offline" | "away";
  unread: number;
  favorite: boolean;
};

type FriendRequest = {
  request_id: string;
  requester_identity_id: string;
  target_identity_id: string;
  status: string;
};

export default function ContactsPage() {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [friendRequests, setFriendRequests] = useState<FriendRequest[]>([]);
  const [search, setSearch] = useState("");
  const [onlineOnly, setOnlineOnly] = useState(false);
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [loading, setLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const [busyTargetId, setBusyTargetId] = useState<string | null>(null);
  const [busyRequestId, setBusyRequestId] = useState<string | null>(null);
  const [inviteMode, setInviteMode] = useState<"one_time" | "multi_use">("one_time");
  const [inviteMaxUses, setInviteMaxUses] = useState("3");
  const [createdInvite, setCreatedInvite] = useState<{
    token: string;
    mode: string;
    expires_at?: string;
    max_uses?: number;
  } | null>(null);
  const [inviteBusy, setInviteBusy] = useState(false);
  const [redeemToken, setRedeemToken] = useState("");
  const [redeemBusy, setRedeemBusy] = useState(false);
  const [redeemResult, setRedeemResult] = useState<{
    request_id: string;
    requester_identity_id: string;
    status: string;
  } | null>(null);
  const [linkCopied, setLinkCopied] = useState(false);

  const identityId = useMemo(() => {
    const active = readActivePersonaId();
    if (active) {
      return active;
    }
    return readPersonas()[0]?.id ?? "usr-nora-k";
  }, []);

  const session = useMemo(() => getPersonaSession(identityId), [identityId]);
  const hasSession = session !== null;

  useEffect(() => {
    let active = true;

    if (!hasSession) {
      return () => {
        active = false;
      };
    }

    const run = async (): Promise<void> => {
      try {
        const [contactsResult, requestsResult] = await Promise.all([
          fetchContacts({ search, onlineOnly, unreadOnly, favoritesOnly }),
          fetchFriendRequests({ identityId }),
        ]);

        if (!active) {
          return;
        }

        if (!contactsResult.ok || !requestsResult.ok) {
          setContacts([]);
          setFriendRequests([]);
          setActionMessage("contacts_unavailable: failed to refresh contacts state");
          setHasError(true);
          setLoading(false);
          return;
        }

        setContacts(
          contactsResult.data.items.map((item) => ({
            id: item.id,
            name: item.name,
            status: item.status as "online" | "offline" | "away",
            unread: item.unread,
            favorite: item.favorite,
          })),
        );
        setFriendRequests(requestsResult.data.items);
        setActionMessage(null);
        setLoading(false);
      } catch {
        if (!active) {
          return;
        }

        setContacts([]);
        setFriendRequests([]);
        setActionMessage("contacts_unavailable: network request failed");
        setHasError(true);
        setLoading(false);
      }
    };

    void run();

    return () => {
      active = false;
    };
  }, [favoritesOnly, hasSession, identityId, onlineOnly, search, unreadOnly]);

  function setFilterState(update: () => void): void {
    setLoading(true);
    setHasError(false);
    update();
  }

  async function refreshRequests(): Promise<void> {
    if (!hasSession) {
      return;
    }

    const result = await fetchFriendRequests({ identityId });
    if (result.ok) {
      setFriendRequests(result.data.items);
      return;
    }

    setActionMessage(`${result.code}: ${result.message}`);
  }

  async function handleCreateRequest(targetIdentityId: string): Promise<void> {
    if (!hasSession) {
      return;
    }

    setBusyTargetId(targetIdentityId);
    setActionMessage(null);

    const tempRequestId = `tmp-${targetIdentityId}`;
    setFriendRequests((previous) => {
      const alreadyPending = previous.some(
        (item) =>
          item.requester_identity_id === identityId &&
          item.target_identity_id === targetIdentityId &&
          item.status === "pending",
      );
      if (alreadyPending) {
        return previous;
      }

      return [
        ...previous,
        {
          request_id: tempRequestId,
          requester_identity_id: identityId,
          target_identity_id: targetIdentityId,
          status: "pending",
        },
      ];
    });

    const result = await createFriendRequest({
      requesterIdentityId: identityId,
      targetIdentityId,
    });

    if (!result.ok) {
      setFriendRequests((previous) => previous.filter((item) => item.request_id !== tempRequestId));
      setActionMessage(`${result.code}: ${result.message}`);
      setBusyTargetId(null);
      return;
    }

    setBusyTargetId(null);
    setActionMessage("friend_request_sent");
    await refreshRequests();
  }

  async function handleAcceptRequest(requestId: string): Promise<void> {
    if (!hasSession) {
      return;
    }

    const previous = friendRequests;
    setBusyRequestId(requestId);
    setActionMessage(null);
    setFriendRequests((items) => items.filter((item) => item.request_id !== requestId));

    const result = await acceptFriendRequest({ requestId });
    if (!result.ok) {
      setFriendRequests(previous);
      setActionMessage(`${result.code}: ${result.message}`);
      setBusyRequestId(null);
      return;
    }

    setBusyRequestId(null);
    setActionMessage("friend_request_accepted");
    await refreshRequests();
  }

  async function handleDeclineRequest(requestId: string): Promise<void> {
    if (!hasSession) {
      return;
    }

    const previous = friendRequests;
    setBusyRequestId(requestId);
    setActionMessage(null);
    setFriendRequests((items) => items.filter((item) => item.request_id !== requestId));

    const result = await declineFriendRequest({ requestId });
    if (!result.ok) {
      setFriendRequests(previous);
      setActionMessage(`${result.code}: ${result.message}`);
      setBusyRequestId(null);
      return;
    }

    setBusyRequestId(null);
    setActionMessage("friend_request_declined");
    await refreshRequests();
  }

  async function handleCreateContactInvite(): Promise<void> {
    if (!hasSession) {
      return;
    }

    setActionMessage(null);
    setCreatedInvite(null);
    setInviteBusy(true);
    setLinkCopied(false);

    const maxUses = Number.parseInt(inviteMaxUses, 10);
    const result = await createContactInvite({
      mode: inviteMode,
      maxUses: inviteMode === "multi_use" && Number.isFinite(maxUses) ? maxUses : undefined,
    });

    setInviteBusy(false);

    if (!result.ok) {
      setActionMessage(`${result.code}: ${result.message}`);
      return;
    }

    setCreatedInvite({
      token: result.data.token,
      mode: result.data.mode,
      expires_at: result.data.expires_at,
      max_uses: result.data.max_uses,
    });
    setActionMessage("contact_invite_created");
  }

  async function handleRedeemContactInvite(): Promise<void> {
    setActionMessage(null);
    setRedeemResult(null);

    const rawToken = redeemToken.trim();
    if (!rawToken) {
      setActionMessage("invite_invalid: token is required");
      return;
    }

    // Extract token from link format if pasted as full link
    const tokenValue = rawToken.includes("/")
      ? rawToken.split("/").pop() ?? rawToken
      : rawToken;

    setRedeemBusy(true);
    const result = await redeemContactInvite({ token: tokenValue });
    setRedeemBusy(false);

    if (!result.ok) {
      setActionMessage(`${result.code}: ${result.message}`);
      return;
    }

    setRedeemResult({
      request_id: result.data.request_id,
      requester_identity_id: result.data.requester_identity_id,
      status: result.data.status,
    });
    setRedeemToken("");
    setActionMessage("contact_invite_redeemed");
  }

  function buildInviteLink(token: string): string {
    return `hexrelay://contact-invite/${token}`;
  }

  async function handleCopyLink(): Promise<void> {
    if (!createdInvite) {
      return;
    }

    try {
      await navigator.clipboard.writeText(buildInviteLink(createdInvite.token));
      setLinkCopied(true);
    } catch {
      setActionMessage("clipboard_error: failed to copy link");
    }
  }

  const inboundPending = friendRequests.filter(
    (item) => item.target_identity_id === identityId && item.status === "pending",
  );
  const outboundPending = friendRequests.filter(
    (item) => item.requester_identity_id === identityId && item.status === "pending",
  );

  const visibleContacts = hasSession ? contacts : [];

  const state = !hasSession
      ? "error"
    : loading
      ? "loading"
    : hasError
      ? "error"
      : visibleContacts.length === 0
        ? search.trim() || onlineOnly || unreadOnly || favoritesOnly
          ? "search_no_results"
          : "empty"
        : inboundPending.length > 0
          ? "friend_request_inbound"
          : outboundPending.length > 0
            ? "friend_request_pending"
            : "ready";

  return (
    <WorkspaceShell
      activeTabId="contacts"
      subtitle="Contacts hub with friend-request and direct-message entrypoints"
      tabs={[
        { id: "contacts", label: "Contacts Hub" },
        { id: "friends", label: "Friends" },
        { id: "requests", label: "Requests" },
      ]}
      title="Contacts"
    >
      <section>
        <div className={styles.row}>
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setOnlineOnly((value) => !value))}
            type="button"
          >
            online {onlineOnly ? "on" : "off"}
          </button>
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setUnreadOnly((value) => !value))}
            type="button"
          >
            unread {unreadOnly ? "on" : "off"}
          </button>
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setFavoritesOnly((value) => !value))}
            type="button"
          >
            favorites {favoritesOnly ? "on" : "off"}
          </button>
        </div>

        <input
          className={styles.search}
          onChange={(event) =>
            setFilterState(() => {
              setSearch(event.target.value);
            })
          }
          placeholder="Search contacts"
          value={search}
        />

        {visibleContacts.length > 0 ? (
          <div className={styles.grid}>
            {visibleContacts.map((contact) => (
              <article className={styles.card} key={contact.id}>
                <p className={styles.title}>{contact.name}</p>
                <p className={styles.meta}>
                  {contact.status} · unread {contact.unread} · {contact.favorite ? "favorite" : "normal"}
                </p>
                <div className={styles.row}>
                  <button
                    className={styles.pill}
                    onClick={() => void handleCreateRequest(contact.id)}
                    disabled={busyTargetId === contact.id}
                    type="button"
                  >
                    request
                  </button>
                </div>
              </article>
            ))}
          </div>
        ) : null}

        {inboundPending.length > 0 ? (
          <div className={styles.state}>
            inbound requests: {inboundPending.length}
            <div className={styles.row}>
              {inboundPending.map((request) => (
                <span key={request.request_id}>
                  <button
                    className={styles.pill}
                    onClick={() => void handleAcceptRequest(request.request_id)}
                    disabled={busyRequestId === request.request_id}
                    type="button"
                  >
                    accept
                  </button>
                  <button
                    className={styles.pill}
                    onClick={() => void handleDeclineRequest(request.request_id)}
                    disabled={busyRequestId === request.request_id}
                    type="button"
                  >
                    decline
                  </button>
                </span>
              ))}
            </div>
          </div>
        ) : null}

        <div className={styles.state}>
          <p>contact invite — share</p>
          <div className={styles.row}>
            <button
              className={styles.pill}
              onClick={() => setInviteMode("one_time")}
              type="button"
            >
              one-time {inviteMode === "one_time" ? "on" : "off"}
            </button>
            <button
              className={styles.pill}
              onClick={() => setInviteMode("multi_use")}
              type="button"
            >
              multi-use {inviteMode === "multi_use" ? "on" : "off"}
            </button>
          </div>
          {inviteMode === "multi_use" ? (
            <input
              className={styles.search}
              onChange={(event) => setInviteMaxUses(event.target.value)}
              placeholder="Max uses"
              value={inviteMaxUses}
            />
          ) : null}
          <button
            className={styles.pill}
            disabled={inviteBusy}
            onClick={() => void handleCreateContactInvite()}
            type="button"
          >
            {inviteBusy ? "creating..." : "create contact invite"}
          </button>

          {createdInvite ? (
            <div className={styles.card} style={{ marginTop: 12 }}>
              <p className={styles.title}>invite ready</p>
              <p className={styles.meta}>
                mode: {createdInvite.mode}
                {createdInvite.max_uses != null ? ` · max uses: ${createdInvite.max_uses}` : ""}
                {createdInvite.expires_at ? ` · expires: ${createdInvite.expires_at}` : ""}
              </p>
              <p className={styles.meta} style={{ wordBreak: "break-all", marginTop: 6 }}>
                {buildInviteLink(createdInvite.token)}
              </p>
              <div className={styles.row} style={{ marginTop: 8 }}>
                <button
                  className={styles.pill}
                  onClick={() => void handleCopyLink()}
                  type="button"
                >
                  {linkCopied ? "copied" : "copy link"}
                </button>
              </div>
              <div style={{ marginTop: 8, display: "flex", justifyContent: "center" }}>
                <QRCodeSVG
                  value={buildInviteLink(createdInvite.token)}
                  size={160}
                  level="M"
                />
              </div>
            </div>
          ) : null}
        </div>

        <div className={styles.state}>
          <p>contact invite — redeem</p>
          <input
            className={styles.search}
            onChange={(event) => setRedeemToken(event.target.value)}
            placeholder="Paste invite token or link"
            value={redeemToken}
          />
          <button
            className={styles.pill}
            disabled={redeemBusy}
            onClick={() => void handleRedeemContactInvite()}
            type="button"
          >
            {redeemBusy ? "redeeming..." : "redeem contact invite"}
          </button>

          {redeemResult ? (
            <div className={styles.card} style={{ marginTop: 12 }}>
              <p className={styles.title}>friend request created</p>
              <p className={styles.meta}>
                request: {redeemResult.request_id} · status: {redeemResult.status}
              </p>
              <p className={styles.meta}>
                inviter: {redeemResult.requester_identity_id}
              </p>
            </div>
          ) : null}
        </div>

        {actionMessage ? <p className={styles.state}>{actionMessage}</p> : null}

        <p className={styles.state}>state: {state}</p>
      </section>
    </WorkspaceShell>
  );
}
