"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  IconClock,
  IconInfoCircle,
  IconLayoutGrid,
  IconList,
  IconMessageCircle,
  IconPinned,
  IconPinnedOff,
  IconSearch,
  IconTrash,
  IconUserPlus,
  IconUsers,
  IconVolume,
  IconVolumeOff,
  IconX,
} from "@tabler/icons-react";

import { HubSurface } from "@/components/hub-surface";
import { WorkspaceShell } from "@/components/workspace-shell";
import {
  acceptFriendRequest,
  blockRemoveContact,
  cancelFriendRequest,
  createFriendRequest,
  declineFriendRequest,
  fetchContacts,
  fetchDiscoveryUsers,
  fetchFriendRequests,
  updateContactPreferences,
} from "@/lib/api";
import type { HubLayout } from "@/lib/hub-state";
import { dmWorkspaceRoute } from "@/lib/navigation-routes";
import { readActivePersonaId, readPersonas, type PersonaRecord } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";
import { readHubLayout, setHubLayout, subscribeWorkspacePreferences } from "@/lib/workspace-preferences";
import { closeWorkspaceTabsForContact } from "@/lib/workspace-tabs";

import styles from "../surfaces.module.css";

type Contact = {
  id: string;
  name: string;
  status: "online" | "offline" | "away";
  unread: number;
  pinned: boolean;
  muted: boolean;
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
  pinned: boolean;
  muted: boolean;
  inbound_request?: boolean;
  pending_request?: boolean;
}): Contact {
  return {
    id: item.id,
    name: item.name,
    status: item.status as Contact["status"],
    unread: item.unread,
    pinned: item.pinned,
    muted: item.muted,
    inboundRequest: item.inbound_request ?? false,
    pendingRequest: item.pending_request ?? false,
  };
}

function formatDateTime(value?: string): string {
  if (!value) {
    return "No date shown";
  }

  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
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
  if (normalized.includes("session")) {
    return "Your session is not ready. Try signing in again.";
  }
  if (normalized.includes("network") || normalized.includes("unavailable")) {
    return "Could not reach HexRelay right now. Try again in a moment.";
  }

  return message || "Something went wrong. Try again in a moment.";
}

export default function ContactsPage() {
  const router = useRouter();
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [friendRequests, setFriendRequests] = useState<FriendRequest[]>([]);
  const [search, setSearch] = useState("");
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [pinnedOnly, setPinnedOnly] = useState(false);
  const [mutedOnly, setMutedOnly] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [selecting, setSelecting] = useState(false);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const [busyRequestId, setBusyRequestId] = useState<string | null>(null);
  const [activePanel, setActivePanel] = useState<"add" | null>(null);
  const [addQuery, setAddQuery] = useState("");
  const [discoveryUsers, setDiscoveryUsers] = useState<DiscoveryUser[]>([]);
  const [discoveryBusy, setDiscoveryBusy] = useState(false);
  const [sendBusyIdentityId, setSendBusyIdentityId] = useState<string | null>(null);
  const [blockTargets, setBlockTargets] = useState<Contact[]>([]);
  const layout = useSyncExternalStore<HubLayout>(
    subscribeWorkspacePreferences,
    () => readHubLayout("contacts"),
    () => "cards",
  );

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

  const refreshContactsAndRequests = useCallback(async (): Promise<void> => {
    if (!hasSession) {
      setContacts([]);
      setFriendRequests([]);
      setLoading(false);
      return;
    }

    try {
      const [contactsResult, requestsResult] = await Promise.all([
        fetchContacts({ search, unreadOnly, pinnedOnly, mutedOnly }),
        fetchFriendRequests({ identityId }),
      ]);

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
      setContacts([]);
      setFriendRequests([]);
      setActionMessage("Could not reach contacts. Check your connection and try again.");
      setHasError(true);
      setLoading(false);
    }
  }, [hasSession, identityId, mutedOnly, pinnedOnly, search, unreadOnly]);

  useEffect(() => {
    let active = true;

    queueMicrotask(() => {
      if (!active) {
        return;
      }

      void refreshContactsAndRequests();
    });

    return () => {
      active = false;
    };
  }, [refreshContactsAndRequests]);

  function setFilterState(update: () => void): void {
    setLoading(true);
    setHasError(false);
    update();
  }

  function openPanel(panel: "add" | null): void {
    setActivePanel((current) => (current === panel ? null : panel));
    setActionMessage(null);
  }

  function toggleSelected(itemId: string): void {
    setSelecting(true);
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(itemId)) {
        next.delete(itemId);
      } else {
        next.add(itemId);
      }
      return next;
    });
  }

  function selectedContacts(): Contact[] {
    return contacts.filter((contact) => selectedIds.has(contact.id));
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
    const target = targetIdentityId.trim();
    if (!target || target === identityId) {
      setActionMessage(target ? "You cannot send a friend request to yourself." : "Enter a user identity id first.");
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
    await refreshContactsAndRequests();
  }

  async function handleRequestTransition(
    requestId: string,
    action: "accept" | "decline" | "cancel",
  ): Promise<void> {
    const previous = friendRequests;
    setBusyRequestId(requestId);
    setActionMessage(null);
    setFriendRequests((items) => items.filter((item) => item.request_id !== requestId));

    const result =
      action === "accept"
        ? await acceptFriendRequest({ requestId })
        : action === "decline"
          ? await declineFriendRequest({ requestId })
          : await cancelFriendRequest({ requestId });

    if (!result.ok) {
      setFriendRequests(previous);
      setActionMessage(formatApiError(result.code, result.message));
      setBusyRequestId(null);
      return;
    }

    setBusyRequestId(null);
    setActionMessage(
      action === "accept"
        ? "Friend request accepted."
        : action === "decline"
          ? "Friend request declined."
          : "Friend request cancelled.",
    );
    await refreshContactsAndRequests();
  }

  async function updateOneContact(contact: Contact, update: { pinned?: boolean; muted?: boolean }): Promise<void> {
    setBusy(true);
    setActionMessage(null);
    const result = await updateContactPreferences({ contactId: contact.id, ...update });
    setBusy(false);

    if (!result.ok) {
      setActionMessage(formatApiError(result.code, result.message));
      return;
    }

    const updated = mapContact(result.data);
    setContacts((items) => items.map((item) => (item.id === contact.id ? updated : item)));
  }

  async function updateSelectedContacts(update: "pin" | "unpin" | "mute" | "unmute"): Promise<void> {
    const targets = selectedContacts();
    if (targets.length === 0) {
      return;
    }

    setBusy(true);
    setActionMessage(null);
    for (const contact of targets) {
      const result = await updateContactPreferences({
        contactId: contact.id,
        pinned: update === "pin" ? true : update === "unpin" ? false : undefined,
        muted: update === "mute" ? true : update === "unmute" ? false : undefined,
      });
      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        setBusy(false);
        return;
      }
      const updated = mapContact(result.data);
      setContacts((items) => items.map((item) => (item.id === contact.id ? updated : item)));
    }
    setBusy(false);
  }

  async function confirmBlockRemove(): Promise<void> {
    setBusy(true);
    setActionMessage(null);
    for (const contact of blockTargets) {
      const result = await blockRemoveContact({ contactId: contact.id });
      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        setBusy(false);
        return;
      }
      closeWorkspaceTabsForContact(contact.id);
      setContacts((items) => items.filter((item) => item.id !== contact.id));
    }
    setSelectedIds(new Set());
    setSelecting(false);
    setBlockTargets([]);
    setBusy(false);
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
          ? search.trim() || unreadOnly || pinnedOnly || mutedOnly
            ? "search_no_results"
            : "empty"
          : "ready";
  const selectedCount = selectedIds.size;

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
              <button className={styles.pill} disabled={discoveryBusy} onClick={() => void handleSearchUsers()} type="button">
                <IconSearch className={styles.icon} aria-hidden="true" />
                {discoveryBusy ? "Searching..." : "Search"}
              </button>
              <button className={styles.pill} disabled={sendBusyIdentityId === addQuery.trim()} onClick={() => void handleSendFriendRequest(addQuery)} type="button">
                <IconUserPlus className={styles.icon} aria-hidden="true" />
                Send request
              </button>
              <button className={styles.pill} onClick={() => openPanel(null)} type="button">
                <IconX className={styles.icon} aria-hidden="true" />
                Close
              </button>
            </div>

            {discoveryUsers.length > 0 ? (
              <div className={styles.hubGrid} style={{ marginTop: 10 }}>
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
                      {user.shared_server_count > 0 ? <span className={styles.badgeMuted}>{user.shared_server_count} shared servers</span> : null}
                      {user.has_pending_outbound_request ? <span className={styles.badgeMuted}>Request pending</span> : null}
                      {user.has_pending_inbound_request ? <span className={styles.badge}>Needs approval</span> : null}
                    </div>
                    <button
                      className={styles.pill}
                      disabled={!user.can_send_friend_request || sendBusyIdentityId === user.identity_id}
                      onClick={() => void handleSendFriendRequest(user.identity_id)}
                      type="button"
                    >
                      <IconUserPlus className={styles.icon} aria-hidden="true" />
                      {user.can_send_friend_request ? "Send request" : "Unavailable"}
                    </button>
                  </article>
                ))}
              </div>
            ) : null}
          </section>
        ) : null}

        {actionMessage ? <p className={styles.state}>{actionMessage}</p> : null}

        {inboundPending.length > 0 ? (
          <RequestSection
            busyRequestId={busyRequestId}
            identityId={identityId}
            kind="inbound"
            onTransition={handleRequestTransition}
            personas={personas}
            requests={inboundPending}
          />
        ) : null}

        {outboundPending.length > 0 ? (
          <RequestSection
            busyRequestId={busyRequestId}
            identityId={identityId}
            kind="outbound"
            onTransition={handleRequestTransition}
            personas={personas}
            requests={outboundPending}
          />
        ) : null}

        <div className={styles.row}>
          <button
            aria-pressed={pinnedOnly}
            className={`${styles.pill} ${pinnedOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setPinnedOnly((value) => !value))}
            type="button"
          >
            <IconPinned className={styles.icon} aria-hidden="true" />
            Pinned
          </button>
          <button
            aria-pressed={unreadOnly}
            className={`${styles.pill} ${unreadOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setUnreadOnly((value) => !value))}
            type="button"
          >
            <IconMessageCircle className={styles.icon} aria-hidden="true" />
            Unread
          </button>
          <button
            aria-pressed={mutedOnly}
            className={`${styles.pill} ${mutedOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setMutedOnly((value) => !value))}
            type="button"
          >
            <IconVolumeOff className={styles.icon} aria-hidden="true" />
            Muted
          </button>
          <button className={styles.pill} onClick={() => setHubLayout("contacts", layout === "cards" ? "list" : "cards")} type="button">
            {layout === "cards" ? <IconList className={styles.icon} aria-hidden="true" /> : <IconLayoutGrid className={styles.icon} aria-hidden="true" />}
            {layout === "cards" ? "List" : "Cards"}
          </button>
          <button
            aria-pressed={selecting}
            className={`${styles.pill} ${selecting ? styles.pillActive : ""}`}
            onClick={() => {
              setSelecting((value) => !value);
              setSelectedIds(new Set());
            }}
            type="button"
          >
            Select{selectedCount > 0 ? ` (${selectedCount})` : ""}
          </button>
        </div>

        {selecting ? (
          <div className={styles.row}>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedContacts("pin")} type="button">
              <IconPinned className={styles.icon} aria-hidden="true" />
              Pin
            </button>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedContacts("unpin")} type="button">
              <IconPinnedOff className={styles.icon} aria-hidden="true" />
              Unpin
            </button>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedContacts("mute")} type="button">
              <IconVolumeOff className={styles.icon} aria-hidden="true" />
              Mute
            </button>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedContacts("unmute")} type="button">
              <IconVolume className={styles.icon} aria-hidden="true" />
              Unmute
            </button>
            <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy || selectedCount === 0} onClick={() => setBlockTargets(selectedContacts())} type="button">
              <IconTrash className={styles.icon} aria-hidden="true" />
              Block + Remove
            </button>
          </div>
        ) : null}

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
            {hasSession ? "Could not load contacts. Try again in a moment." : "Create or select a profile before managing contacts."}
          </p>
        ) : null}
        {pageState === "search_no_results" ? <p className={styles.state}>No contacts match your search or filters.</p> : null}
        {pageState === "empty" ? (
          <section className={styles.state} aria-label="No contacts">
            <p className={styles.title}>No contacts yet</p>
            <p className={styles.meta}>Search for someone to send a friend request.</p>
            <button className={styles.pill} onClick={() => openPanel("add")} type="button">
              <IconUserPlus className={styles.icon} aria-hidden="true" />
              Add your first contact
            </button>
          </section>
        ) : null}

        {visibleContacts.length > 0 ? (
          <HubSurface
            items={visibleContacts}
            layout={layout}
            noun="contact"
            onOpen={(contact) => {
              if (!contact.pendingRequest && !contact.inboundRequest) {
                router.push(dmWorkspaceRoute(contact.id));
              }
            }}
            onToggleSelected={toggleSelected}
            renderBadges={(contact) => (
              <>
                <span className={contact.status === "online" ? styles.badge : styles.badgeMuted}>{statusLabel(contact.status)}</span>
                {contact.pendingRequest ? <span className={styles.badgeMuted}>Request pending</span> : null}
                {contact.inboundRequest ? <span className={styles.badge}>Needs approval</span> : null}
              </>
            )}
            renderActions={(contact) => (
              <>
                {contact.pendingRequest || contact.inboundRequest ? (
                  <button className={styles.pill} disabled type="button">
                    <IconClock className={styles.icon} aria-hidden="true" />
                    Request pending
                  </button>
                ) : (
                  <Link className={styles.pill} href={dmWorkspaceRoute(contact.id)}>
                    <IconMessageCircle className={styles.icon} aria-hidden="true" />
                    Message
                  </Link>
                )}
                <button className={styles.pill} disabled={busy} onClick={() => void updateOneContact(contact, { pinned: !contact.pinned })} type="button">
                  {contact.pinned ? <IconPinnedOff className={styles.icon} aria-hidden="true" /> : <IconPinned className={styles.icon} aria-hidden="true" />}
                  {contact.pinned ? "Unpin" : "Pin"}
                </button>
                <button className={styles.pill} disabled={busy} onClick={() => void updateOneContact(contact, { muted: !contact.muted })} type="button">
                  {contact.muted ? <IconVolume className={styles.icon} aria-hidden="true" /> : <IconVolumeOff className={styles.icon} aria-hidden="true" />}
                  {contact.muted ? "Unmute" : "Mute"}
                </button>
                <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy} onClick={() => setBlockTargets([contact])} type="button">
                  <IconTrash className={styles.icon} aria-hidden="true" />
                  Block + Remove
                </button>
              </>
            )}
            selectedIds={selectedIds}
            selecting={selecting}
          />
        ) : null}

        {blockTargets.length > 0 ? (
          <div className={styles.dialogBackdrop} role="presentation">
            <section aria-label="Block and remove confirmation" className={styles.dialog}>
              <p className={styles.title}>Block + Remove {blockTargets.length === 1 ? blockTargets[0]?.name : `${blockTargets.length} contacts`}?</p>
              <p className={styles.meta}>This blocks the user, removes the contact relationship, and keeps existing DM history.</p>
              <div className={styles.row}>
                <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy} onClick={() => void confirmBlockRemove()} type="button">
                  Block + Remove
                </button>
                <button className={styles.pill} disabled={busy} onClick={() => setBlockTargets([])} type="button">
                  Cancel
                </button>
              </div>
            </section>
          </div>
        ) : null}
      </section>
    </WorkspaceShell>
  );
}

function RequestSection({
  kind,
  requests,
  identityId,
  personas,
  busyRequestId,
  onTransition,
}: {
  kind: "inbound" | "outbound";
  requests: FriendRequest[];
  identityId: string;
  personas: PersonaRecord[];
  busyRequestId: string | null;
  onTransition: (requestId: string, action: "accept" | "decline" | "cancel") => Promise<void>;
}) {
  return (
    <section className={styles.state} aria-label={kind === "inbound" ? "Friend requests" : "Sent requests"}>
      <p className={styles.title}>{kind === "inbound" ? "Friend requests" : "Sent requests"}</p>
      <div className={styles.hubGrid} style={{ marginTop: 10 }}>
        {requests.map((request) => {
          const peerId = kind === "inbound" ? request.requester_identity_id : request.target_identity_id;
          const peerName = identityLabel(peerId, identityId, personas);

          return (
            <article className={styles.card} key={request.request_id}>
              <div className={styles.cardHeader}>
                <div className={styles.avatar}>{contactInitials(peerName)}</div>
                <div>
                  <p className={styles.title}>{peerName}</p>
                  <p className={styles.meta}>{kind === "inbound" ? "Wants to add you" : "Waiting for them to accept"}</p>
                </div>
              </div>
              <div className={styles.row}>
                <span className={kind === "inbound" ? styles.badge : styles.badgeMuted}>
                  {kind === "inbound" ? "Needs your approval" : "Pending"}
                </span>
                {request.created_at ? <span className={styles.badgeMuted}>Sent {formatDateTime(request.created_at)}</span> : null}
              </div>
              <div className={styles.row}>
                {kind === "inbound" ? (
                  <>
                    <button className={styles.pill} disabled={busyRequestId === request.request_id} onClick={() => void onTransition(request.request_id, "accept")} type="button">
                      <IconInfoCircle className={styles.icon} aria-hidden="true" />
                      Accept
                    </button>
                    <button className={styles.pill} disabled={busyRequestId === request.request_id} onClick={() => void onTransition(request.request_id, "decline")} type="button">
                      <IconX className={styles.icon} aria-hidden="true" />
                      Decline
                    </button>
                  </>
                ) : (
                  <button className={styles.pill} disabled={busyRequestId === request.request_id} onClick={() => void onTransition(request.request_id, "cancel")} type="button">
                    <IconX className={styles.icon} aria-hidden="true" />
                    Cancel
                  </button>
                )}
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
