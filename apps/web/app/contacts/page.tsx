"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import {
  IconCircleCheck,
  IconCircleCheckFilled,
  IconClock,
  IconInfoCircle,
  IconMessageCircle,
  IconMessageCircleFilled,
  IconSearch,
  IconStar,
  IconStarFilled,
  IconUserPlus,
  IconUsers,
  IconX,
} from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  acceptFriendRequest,
  cancelFriendRequest,
  createFriendRequest,
  declineFriendRequest,
  fetchContacts,
  fetchDiscoveryUsers,
  fetchFriendRequests,
} from "@/lib/api";
import { readActivePersonaId, readPersonas, type PersonaRecord } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";

import styles from "../surfaces.module.css";

type Contact = {
  id: string;
  name: string;
  status: "online" | "offline" | "away";
  unread: number;
  favorite: boolean;
  inboundRequest: boolean;
  pendingRequest: boolean;
};

type FriendRequest = {
  request_id: string;
  requester_identity_id: string;
  target_identity_id: string;
  status: string;
  created_at?: string;
};

type DiscoveryUser = {
  identity_id: string;
  display_name: string;
  relationship_state: string;
  shared_server_count: number;
  can_send_friend_request: boolean;
  has_pending_inbound_request: boolean;
  has_pending_outbound_request: boolean;
};

function shortIdentity(identityId: string): string {
  if (identityId.length <= 18) {
    return identityId;
  }

  return `${identityId.slice(0, 8)}...${identityId.slice(-6)}`;
}

function identityLabel(identityId: string, activeIdentityId: string, personas: PersonaRecord[]): string {
  if (identityId === activeIdentityId) {
    return "You";
  }

  return personas.find((persona) => persona.id === identityId)?.name ?? shortIdentity(identityId);
}

function contactInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function statusLabel(status: Contact["status"]): string {
  if (status === "online") {
    return "Online";
  }
  if (status === "away") {
    return "Away";
  }

  return "Offline";
}

function mapContact(item: {
  id: string;
  name: string;
  status: string;
  unread: number;
  favorite: boolean;
  inbound_request?: boolean;
  pending_request?: boolean;
}): Contact {
  return {
    id: item.id,
    name: item.name,
    status: item.status as Contact["status"],
    unread: item.unread,
    favorite: item.favorite,
    inboundRequest: item.inbound_request ?? false,
    pendingRequest: item.pending_request ?? false,
  };
}

function formatDateTime(value?: string): string {
  if (!value) {
    return "No date shown";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

function formatApiError(code: string, message: string): string {
  const normalized = `${code} ${message}`.toLowerCase();

  if (normalized.includes("blocked")) {
    return "That request is blocked by current relationship settings.";
  }
  if (normalized.includes("already") || normalized.includes("exists")) {
    return "A request with this user already exists.";
  }
  if (normalized.includes("invalid") || normalized.includes("not found")) {
    return "That user or request could not be found.";
  }
  if (normalized.includes("rate")) {
    return "Too many attempts. Wait a moment and try again.";
  }
  if (normalized.includes("unauthorized") || normalized.includes("session")) {
    return "Your session is not ready. Try signing in again.";
  }
  if (normalized.includes("network") || normalized.includes("unavailable")) {
    return "Could not reach HexRelay right now. Try again in a moment.";
  }

  return "Something went wrong. Try again in a moment.";
}

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
  const [busyRequestId, setBusyRequestId] = useState<string | null>(null);
  const [activePanel, setActivePanel] = useState<"add" | null>(null);
  const [addQuery, setAddQuery] = useState("");
  const [discoveryUsers, setDiscoveryUsers] = useState<DiscoveryUser[]>([]);
  const [discoveryBusy, setDiscoveryBusy] = useState(false);
  const [sendBusyIdentityId, setSendBusyIdentityId] = useState<string | null>(null);

  const personas = useMemo(() => readPersonas(), []);
  const identityId = useMemo(() => {
    const active = readActivePersonaId();
    if (active) {
      return active;
    }
    return personas[0]?.id ?? "usr-nora-k";
  }, [personas]);

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
          setActionMessage("Could not refresh contacts. Try again in a moment.");
          setHasError(true);
          setLoading(false);
          return;
        }

        setContacts(contactsResult.data.items.map(mapContact));
        setFriendRequests(requestsResult.data.items);
        setActionMessage(null);
        setHasError(false);
        setLoading(false);
      } catch {
        if (!active) {
          return;
        }

        setContacts([]);
        setFriendRequests([]);
        setActionMessage("Could not reach contacts. Check your connection and try again.");
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

  function openPanel(panel: "add" | null): void {
    setActivePanel((current) => (current === panel ? null : panel));
    setActionMessage(null);
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

    setActionMessage(formatApiError(result.code, result.message));
  }

  async function refreshContactsAndRequests(): Promise<void> {
    if (!hasSession) {
      return;
    }

    const [contactsResult, requestsResult] = await Promise.all([
      fetchContacts({ search, onlineOnly, unreadOnly, favoritesOnly }),
      fetchFriendRequests({ identityId }),
    ]);

    if (!contactsResult.ok) {
      setActionMessage(formatApiError(contactsResult.code, contactsResult.message));
      return;
    }
    if (!requestsResult.ok) {
      setActionMessage(formatApiError(requestsResult.code, requestsResult.message));
      return;
    }

    setContacts(contactsResult.data.items.map(mapContact));
    setFriendRequests(requestsResult.data.items);
    setHasError(false);
  }

  async function handleSearchUsers(): Promise<void> {
    if (!hasSession) {
      setActionMessage("Create or select a profile before managing contacts.");
      return;
    }

    const query = addQuery.trim();
    if (!query) {
      setActionMessage("Enter a name or identity id first.");
      setDiscoveryUsers([]);
      return;
    }

    setDiscoveryBusy(true);
    setActionMessage(null);
    const result = await fetchDiscoveryUsers({ query, scope: "global", limit: 8 });
    setDiscoveryBusy(false);

    if (!result.ok) {
      setDiscoveryUsers([]);
      setActionMessage(formatApiError(result.code, result.message));
      return;
    }

    setDiscoveryUsers(result.data.items.filter((item) => item.identity_id !== identityId));
    if (result.data.items.length === 0) {
      setActionMessage("No users matched that search.");
    }
  }

  async function handleSendFriendRequest(targetIdentityId: string): Promise<void> {
    if (!hasSession) {
      setActionMessage("Create or select a profile before managing contacts.");
      return;
    }

    const target = targetIdentityId.trim();
    if (!target) {
      setActionMessage("Enter a user identity id first.");
      return;
    }
    if (target === identityId) {
      setActionMessage("You cannot send a friend request to yourself.");
      return;
    }

    setSendBusyIdentityId(target);
    setActionMessage(null);
    const result = await createFriendRequest({
      requesterIdentityId: identityId,
      targetIdentityId: target,
    });
    setSendBusyIdentityId(null);

    if (!result.ok) {
      setActionMessage(formatApiError(result.code, result.message));
      return;
    }

    setActionMessage("Friend request sent.");
    setAddQuery("");
    setDiscoveryUsers((items) =>
      items.map((item) =>
        item.identity_id === target
          ? { ...item, can_send_friend_request: false, has_pending_outbound_request: true }
          : item,
      ),
    );
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
      setActionMessage(formatApiError(result.code, result.message));
      setBusyRequestId(null);
      return;
    }

    setBusyRequestId(null);
    setActionMessage("Friend request accepted.");
    await refreshContactsAndRequests();
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
      setActionMessage(formatApiError(result.code, result.message));
      setBusyRequestId(null);
      return;
    }

    setBusyRequestId(null);
    setActionMessage("Friend request declined.");
    await refreshRequests();
  }

  async function handleCancelRequest(requestId: string): Promise<void> {
    if (!hasSession) {
      return;
    }

    const previous = friendRequests;
    setBusyRequestId(requestId);
    setActionMessage(null);
    setFriendRequests((items) => items.filter((item) => item.request_id !== requestId));

    const result = await cancelFriendRequest({ requestId });
    if (!result.ok) {
      setFriendRequests(previous);
      setActionMessage(formatApiError(result.code, result.message));
      setBusyRequestId(null);
      return;
    }

    setBusyRequestId(null);
    setActionMessage("Friend request cancelled.");
  }

  const inboundPending = friendRequests.filter(
    (item) => item.target_identity_id === identityId && item.status === "pending",
  );
  const outboundPending = friendRequests.filter(
    (item) => item.requester_identity_id === identityId && item.status === "pending",
  );

  const visibleContacts = hasSession ? contacts : [];
  const pageState = !hasSession
    ? "error"
    : loading
      ? "loading"
      : hasError
        ? "error"
        : visibleContacts.length === 0
          ? search.trim() || onlineOnly || unreadOnly || favoritesOnly
            ? "search_no_results"
            : "empty"
          : "ready";
  const OnlineFilterIcon = onlineOnly ? IconCircleCheckFilled : IconCircleCheck;
  const UnreadFilterIcon = unreadOnly ? IconMessageCircleFilled : IconMessageCircle;
  const FavoritesFilterIcon = favoritesOnly ? IconStarFilled : IconStar;

  return (
    <WorkspaceShell
      activeTabId="contacts"
      subtitle="People you know, friend requests, and private conversations"
      tabs={[
        { id: "contacts", label: "All contacts", icon: IconUsers },
        { id: "requests", label: "Requests", icon: IconClock },
      ]}
      tabActions={
        <button className={styles.pill} disabled={!hasSession} onClick={() => openPanel("add")} type="button">
          <IconUserPlus className={styles.icon} aria-hidden="true" />
          Add contact
        </button>
      }
      title="Contacts"
    >
      <section>
        {activePanel === "add" ? (
          <section className={styles.state} aria-label="Add contact">
            <p className={styles.title}>Add contact</p>
            <p className={styles.meta}>Search users or enter an identity id to send a friend request.</p>
            <div className={styles.inputWrap}>
              <IconSearch className={styles.inputIcon} aria-hidden="true" />
              <input
                aria-label="User search or identity id"
                className={styles.search}
                onChange={(event) => setAddQuery(event.target.value)}
                placeholder="Name or identity id"
                value={addQuery}
              />
            </div>
            <div className={styles.row}>
              <button
                className={styles.pill}
                disabled={discoveryBusy}
                onClick={() => void handleSearchUsers()}
                type="button"
              >
                <IconSearch className={styles.icon} aria-hidden="true" />
                {discoveryBusy ? "Searching..." : "Search"}
              </button>
              <button
                className={styles.pill}
                disabled={sendBusyIdentityId === addQuery.trim()}
                onClick={() => void handleSendFriendRequest(addQuery)}
                type="button"
              >
                <IconUserPlus className={styles.icon} aria-hidden="true" />
                Send request
              </button>
              <button className={styles.pill} onClick={() => openPanel(null)} type="button">
                <IconX className={styles.icon} aria-hidden="true" />
                Close
              </button>
            </div>

            {discoveryUsers.length > 0 ? (
              <div className={styles.grid} style={{ marginTop: 10 }}>
                {discoveryUsers.map((user) => (
                  <article className={styles.card} key={user.identity_id}>
                    <div className={styles.cardHeader}>
                      <div className={styles.avatar}>{contactInitials(user.display_name)}</div>
                      <div>
                        <p className={styles.title}>{user.display_name}</p>
                        <p className={styles.meta}>{shortIdentity(user.identity_id)}</p>
                      </div>
                    </div>
                    <div className={styles.row}>
                      {user.shared_server_count > 0 ? (
                        <span className={styles.badgeMuted}>{user.shared_server_count} shared servers</span>
                      ) : null}
                      {user.has_pending_outbound_request ? <span className={styles.badgeMuted}>Request pending</span> : null}
                      {user.has_pending_inbound_request ? <span className={styles.badge}>Needs approval</span> : null}
                    </div>
                    <div className={styles.row} style={{ marginTop: 8 }}>
                      <button
                        className={styles.pill}
                        disabled={!user.can_send_friend_request || sendBusyIdentityId === user.identity_id}
                        onClick={() => void handleSendFriendRequest(user.identity_id)}
                        type="button"
                      >
                        <IconUserPlus className={styles.icon} aria-hidden="true" />
                        {user.can_send_friend_request ? "Send request" : "Unavailable"}
                      </button>
                    </div>
                  </article>
                ))}
              </div>
            ) : null}
          </section>
        ) : null}

        {actionMessage ? <p className={styles.state}>{actionMessage}</p> : null}

        {inboundPending.length > 0 ? (
          <section className={styles.state} aria-label="Friend requests">
            <p className={styles.title}>Friend requests</p>
            <p className={styles.meta}>People waiting for your approval.</p>
            <div className={styles.grid} style={{ marginTop: 10 }}>
              {inboundPending.map((request) => {
                const requesterName = identityLabel(request.requester_identity_id, identityId, personas);

                return (
                  <article className={styles.card} key={request.request_id}>
                    <div className={styles.cardHeader}>
                      <div className={styles.avatar}>{contactInitials(requesterName)}</div>
                      <div>
                        <p className={styles.title}>{requesterName}</p>
                        <p className={styles.meta}>Wants to add you</p>
                      </div>
                    </div>
                    <div className={styles.row}>
                      <span className={styles.badge}>Needs your approval</span>
                      {request.created_at ? <span className={styles.badgeMuted}>Sent {formatDateTime(request.created_at)}</span> : null}
                    </div>
                    <div className={styles.row} style={{ marginTop: 8 }}>
                      <button
                        className={styles.pill}
                        onClick={() => void handleAcceptRequest(request.request_id)}
                        disabled={busyRequestId === request.request_id}
                        type="button"
                      >
                        <IconCircleCheck className={styles.icon} aria-hidden="true" />
                        Accept
                      </button>
                      <button
                        className={styles.pill}
                        onClick={() => void handleDeclineRequest(request.request_id)}
                        disabled={busyRequestId === request.request_id}
                        type="button"
                      >
                        <IconX className={styles.icon} aria-hidden="true" />
                        Decline
                      </button>
                    </div>
                  </article>
                );
              })}
            </div>
          </section>
        ) : null}

        {outboundPending.length > 0 ? (
          <section className={styles.state} aria-label="Sent requests">
            <p className={styles.title}>Sent requests</p>
            <p className={styles.meta}>People who still need to accept your friend request.</p>
            <div className={styles.grid} style={{ marginTop: 10 }}>
              {outboundPending.map((request) => {
                const targetName = identityLabel(request.target_identity_id, identityId, personas);

                return (
                  <article className={styles.card} key={request.request_id}>
                    <div className={styles.cardHeader}>
                      <div className={styles.avatar}>{contactInitials(targetName)}</div>
                      <div>
                        <p className={styles.title}>{targetName}</p>
                        <p className={styles.meta}>Waiting for them to accept</p>
                      </div>
                    </div>
                    <div className={styles.row}>
                      <span className={styles.badgeMuted}>Pending</span>
                      {request.created_at ? <span className={styles.badgeMuted}>Sent {formatDateTime(request.created_at)}</span> : null}
                    </div>
                    <div className={styles.row} style={{ marginTop: 8 }}>
                      <button
                        className={styles.pill}
                        onClick={() => void handleCancelRequest(request.request_id)}
                        disabled={busyRequestId === request.request_id}
                        type="button"
                      >
                        <IconX className={styles.icon} aria-hidden="true" />
                        Cancel
                      </button>
                    </div>
                  </article>
                );
              })}
            </div>
          </section>
        ) : null}

        <div className={styles.row}>
          <button
            aria-pressed={onlineOnly}
            className={`${styles.pill} ${onlineOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setOnlineOnly((value) => !value))}
            type="button"
          >
            <OnlineFilterIcon className={styles.icon} aria-hidden="true" />
            Online
          </button>
          <button
            aria-pressed={unreadOnly}
            className={`${styles.pill} ${unreadOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setUnreadOnly((value) => !value))}
            type="button"
          >
            <UnreadFilterIcon className={styles.icon} aria-hidden="true" />
            Unread
          </button>
          <button
            aria-pressed={favoritesOnly}
            className={`${styles.pill} ${favoritesOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setFavoritesOnly((value) => !value))}
            type="button"
          >
            <FavoritesFilterIcon className={styles.icon} aria-hidden="true" />
            Favorites
          </button>
        </div>

        <div className={styles.inputWrap}>
          <IconSearch className={styles.inputIcon} aria-hidden="true" />
          <input
            aria-label="Search contacts"
            className={styles.search}
            onChange={(event) =>
              setFilterState(() => {
                setSearch(event.target.value);
              })
            }
            placeholder="Search contacts"
            value={search}
          />
        </div>

        {pageState === "loading" ? <p className={styles.state}>Loading contacts...</p> : null}

        {pageState === "error" ? (
          <p className={styles.state}>
            {hasSession
              ? "Could not load contacts. Try again in a moment."
              : "Create or select a profile before managing contacts."}
          </p>
        ) : null}

        {pageState === "search_no_results" ? (
          <p className={styles.state}>No contacts match your search or filters.</p>
        ) : null}

        {pageState === "empty" ? (
          <section className={styles.state} aria-label="No contacts">
            <p className={styles.title}>No contacts yet</p>
            <p className={styles.meta}>Search for someone to send a friend request.</p>
            <div className={styles.row} style={{ marginTop: 10 }}>
              <button className={styles.pill} onClick={() => openPanel("add")} type="button">
                <IconUserPlus className={styles.icon} aria-hidden="true" />
                Add your first contact
              </button>
            </div>
          </section>
        ) : null}

        {visibleContacts.length > 0 ? (
          <section aria-label="Your contacts">
            <div className={styles.contactGrid}>
              {visibleContacts.map((contact) => (
                <article className={styles.card} key={contact.id}>
                  <div className={styles.cardHeader}>
                    <div className={styles.avatar}>{contactInitials(contact.name)}</div>
                    <div>
                      <p className={styles.title}>{contact.name}</p>
                      <p className={styles.meta}>{shortIdentity(contact.id)}</p>
                    </div>
                  </div>
                  <div className={styles.row}>
                    <span className={contact.status === "online" ? styles.badge : styles.badgeMuted}>
                      <IconCircleCheck className={styles.icon} aria-hidden="true" />
                      {statusLabel(contact.status)}
                    </span>
                    {contact.unread > 0 ? (
                      <span className={styles.badge}>
                        <IconMessageCircle className={styles.icon} aria-hidden="true" />
                        {contact.unread} unread
                      </span>
                    ) : null}
                    {contact.favorite ? (
                      <span className={styles.badgeMuted}>
                        <IconStar className={styles.icon} aria-hidden="true" />
                        Favorite
                      </span>
                    ) : null}
                    {contact.pendingRequest ? (
                      <span className={styles.badgeMuted}>
                        <IconClock className={styles.icon} aria-hidden="true" />
                        Request pending
                      </span>
                    ) : null}
                    {contact.inboundRequest ? (
                      <span className={styles.badge}>
                        <IconInfoCircle className={styles.icon} aria-hidden="true" />
                        Needs approval
                      </span>
                    ) : null}
                  </div>
                  <details className={styles.compactDetails}>
                    <summary>
                      <IconInfoCircle className={styles.icon} aria-hidden="true" /> Contact details
                    </summary>
                    <p className={styles.meta} style={{ wordBreak: "break-all" }}>
                      Contact ID: {contact.id}
                    </p>
                  </details>
                  <div className={styles.row} style={{ marginTop: 8 }}>
                    {contact.pendingRequest || contact.inboundRequest ? (
                      <button className={styles.pill} disabled type="button">
                        <IconClock className={styles.icon} aria-hidden="true" />
                        Request pending
                      </button>
                    ) : (
                      <Link
                        className={styles.pill}
                        href={`/contacts/${encodeURIComponent(contact.id)}/messages`}
                      >
                        <IconMessageCircle className={styles.icon} aria-hidden="true" />
                        Message
                      </Link>
                    )}
                  </div>
                </article>
              ))}
            </div>
          </section>
        ) : null}
      </section>
    </WorkspaceShell>
  );
}
