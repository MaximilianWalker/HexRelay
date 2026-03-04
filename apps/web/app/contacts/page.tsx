"use client";

import { useEffect, useMemo, useState } from "react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  acceptFriendRequest,
  createFriendRequest,
  createInvite,
  declineFriendRequest,
  fetchContacts,
  fetchFriendRequests,
  redeemInvite,
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
  const [inviteToken, setInviteToken] = useState("");
  const [redeemToken, setRedeemToken] = useState("");
  const [redeemFingerprint, setRedeemFingerprint] = useState("hexrelay-local-fingerprint");

  const identityId = useMemo(() => {
    const active = readActivePersonaId();
    if (active) {
      return active;
    }
    return readPersonas()[0]?.id ?? "usr-nora-k";
  }, []);

  const session = useMemo(() => getPersonaSession(identityId), [identityId]);
  const accessToken = session?.accessToken ?? null;

  useEffect(() => {
    let active = true;

    if (!accessToken) {
      return () => {
        active = false;
      };
    }

    const run = async (): Promise<void> => {
      try {
        const [contactsResult, requestsResult] = await Promise.all([
          fetchContacts({ search, onlineOnly, unreadOnly, favoritesOnly, accessToken }),
          fetchFriendRequests({ identityId, accessToken }),
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
  }, [accessToken, favoritesOnly, identityId, onlineOnly, search, unreadOnly]);

  function setFilterState(update: () => void): void {
    setLoading(true);
    setHasError(false);
    update();
  }

  async function refreshRequests(): Promise<void> {
    if (!accessToken) {
      return;
    }

    const result = await fetchFriendRequests({ identityId, accessToken });
    if (result.ok) {
      setFriendRequests(result.data.items);
      return;
    }

    setActionMessage(`${result.code}: ${result.message}`);
  }

  async function handleCreateRequest(targetIdentityId: string): Promise<void> {
    if (!accessToken) {
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
      accessToken,
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
    if (!accessToken) {
      return;
    }

    const previous = friendRequests;
    setBusyRequestId(requestId);
    setActionMessage(null);
    setFriendRequests((items) => items.filter((item) => item.request_id !== requestId));

    const result = await acceptFriendRequest({ requestId, accessToken });
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
    if (!accessToken) {
      return;
    }

    const previous = friendRequests;
    setBusyRequestId(requestId);
    setActionMessage(null);
    setFriendRequests((items) => items.filter((item) => item.request_id !== requestId));

    const result = await declineFriendRequest({ requestId, accessToken });
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

  async function handleCreateInvite(): Promise<void> {
    if (!accessToken) {
      return;
    }

    setActionMessage(null);

    const maxUses = Number.parseInt(inviteMaxUses, 10);
    const result = await createInvite({
      mode: inviteMode,
      maxUses: inviteMode === "multi_use" && Number.isFinite(maxUses) ? maxUses : undefined,
      accessToken,
    });

    if (!result.ok) {
      setActionMessage(`${result.code}: ${result.message}`);
      return;
    }

    setInviteToken(result.data.token);
    setActionMessage("invite_created");
  }

  async function handleRedeemInvite(): Promise<void> {
    setActionMessage(null);
    if (!redeemToken.trim()) {
      setActionMessage("invite_invalid: token is required");
      return;
    }

    const result = await redeemInvite({
      token: redeemToken.trim(),
      nodeFingerprint: redeemFingerprint.trim(),
    });

    if (!result.ok) {
      setActionMessage(`${result.code}: ${result.message}`);
      return;
    }

    setActionMessage("invite_redeemed");
    setRedeemToken("");
  }

  const inboundPending = friendRequests.filter(
    (item) => item.target_identity_id === identityId && item.status === "pending",
  );
  const outboundPending = friendRequests.filter(
    (item) => item.requester_identity_id === identityId && item.status === "pending",
  );

  const visibleContacts = accessToken ? contacts : [];

  const state = !accessToken
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
          <p>invite tools</p>
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
          <button className={styles.pill} onClick={() => void handleCreateInvite()} type="button">
            create invite
          </button>
          {inviteToken ? <p className={styles.meta}>token: {inviteToken}</p> : null}

          <input
            className={styles.search}
            onChange={(event) => setRedeemToken(event.target.value)}
            placeholder="Redeem token"
            value={redeemToken}
          />
          <input
            className={styles.search}
            onChange={(event) => setRedeemFingerprint(event.target.value)}
            placeholder="Node fingerprint"
            value={redeemFingerprint}
          />
          <button className={styles.pill} onClick={() => void handleRedeemInvite()} type="button">
            redeem invite
          </button>
        </div>

        {actionMessage ? <p className={styles.state}>{actionMessage}</p> : null}

        <p className={styles.state}>state: {state}</p>
      </section>
    </WorkspaceShell>
  );
}
