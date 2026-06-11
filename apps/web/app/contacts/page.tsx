"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { useRouter } from "next/navigation";
import { IconUserPlus } from "@tabler/icons-react";

import { ContactAddDialog } from "@/components/hubs/contact-add-dialog";
import { ContactBlockDialog } from "@/components/hubs/contact-block-dialog";
import type { ContactDiscoveryUser } from "@/components/hubs/contact-discovery-results";
import { ContactRequestSection } from "@/components/hubs/contact-request-section";
import { BulkActions } from "@/components/hubs/bulk-actions";
import { ItemActions } from "@/components/hubs/item-actions";
import { Surface } from "@/components/hubs/surface";
import { Toolbar } from "@/components/hubs/toolbar";
import { Badge } from "@/components/ui/display/badge";
import { Button } from "@/components/ui/buttons/button";
import { MainLayout } from "@/components/layout/main";
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
import {
  EMPTY_PERSONA_SNAPSHOT,
  parsePersonaSnapshot,
  readPersonaSnapshot,
  type PersonaRecord,
} from "@/lib/personas";
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

type DiscoveryUser = ContactDiscoveryUser;

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

  const personaSnapshot = useSyncExternalStore(
    subscribeWorkspacePreferences,
    readPersonaSnapshot,
    () => EMPTY_PERSONA_SNAPSHOT,
  );
  const { activePersonaId, personas } = parsePersonaSnapshot(personaSnapshot);
  const identityId = activePersonaId ?? personas[0]?.id ?? "usr-nora-k";

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

  function closePanel(): void {
    setActivePanel(null);
    setActionMessage(null);
  }

  function toggleSelected(itemId: string): void {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(itemId)) {
        next.delete(itemId);
      } else {
        next.add(itemId);
      }
      setSelecting(next.size > 0);
      return next;
    });
  }

  function clearSelection(): void {
    setSelectedIds(new Set());
    setSelecting(false);
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
    <MainLayout
      activeTabId="contacts"
      subtitle="People you know, friend requests, and private conversations"
      tabs={[]}
      title="Contacts"
    >
      <section>
        {actionMessage && activePanel === null ? <p className={styles.state}>{actionMessage}</p> : null}

        {inboundPending.length > 0 ? (
          <ContactRequestSection
            busyRequestId={busyRequestId}
            formatDateTime={formatDateTime}
            identityId={identityId}
            identityLabel={identityLabel}
            kind="inbound"
            onTransition={handleRequestTransition}
            personas={personas}
            requests={inboundPending}
          />
        ) : null}

        {outboundPending.length > 0 ? (
          <ContactRequestSection
            busyRequestId={busyRequestId}
            formatDateTime={formatDateTime}
            identityId={identityId}
            identityLabel={identityLabel}
            kind="outbound"
            onTransition={handleRequestTransition}
            personas={personas}
            requests={outboundPending}
          />
        ) : null}

        <Toolbar
          actions={
            <Button
              disabled={!hasSession}
              icon={<IconUserPlus aria-hidden="true" />}
              onClick={() => openPanel("add")}
            >
              Add contact
            </Button>
          }
          layout={layout}
          mutedOnly={mutedOnly}
          onLayoutChange={(nextLayout) => setHubLayout("contacts", nextLayout)}
          onMutedChange={() => setFilterState(() => setMutedOnly((value) => !value))}
          onPinnedChange={() => setFilterState(() => setPinnedOnly((value) => !value))}
          onSearchChange={(value) =>
            setFilterState(() => {
              setSearch(value);
            })
          }
          onUnreadChange={() => setFilterState(() => setUnreadOnly((value) => !value))}
          pinnedOnly={pinnedOnly}
          search={search}
          searchLabel="Search contacts"
          unreadOnly={unreadOnly}
        />

        {selecting ? (
          <BulkActions
            busy={busy}
            destructiveLabel="Block + Remove"
            onDestructive={() => setBlockTargets(selectedContacts())}
            onDone={clearSelection}
            onMute={() => void updateSelectedContacts("mute")}
            onPin={() => void updateSelectedContacts("pin")}
            onUnmute={() => void updateSelectedContacts("unmute")}
            onUnpin={() => void updateSelectedContacts("unpin")}
            selectedCount={selectedCount}
          />
        ) : null}

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
            <Button icon={<IconUserPlus aria-hidden="true" />} onClick={() => openPanel("add")}>
              Add your first contact
            </Button>
          </section>
        ) : null}

        {visibleContacts.length > 0 ? (
          <Surface
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
                <Badge tone={contact.status === "online" ? "success" : "muted"}>{statusLabel(contact.status)}</Badge>
                {contact.pendingRequest ? <Badge tone="muted">Request pending</Badge> : null}
                {contact.inboundRequest ? <Badge tone="accent">Needs approval</Badge> : null}
              </>
            )}
            renderActions={(contact) => (
              <ItemActions
                busy={busy}
                destructiveLabel="Block + Remove"
                messageAction={{
                  disabled: contact.pendingRequest || contact.inboundRequest,
                  label: "Message",
                  onClick: () => router.push(dmWorkspaceRoute(contact.id)),
                }}
                muted={contact.muted}
                onDestructive={() => setBlockTargets([contact])}
                onToggleMuted={() => void updateOneContact(contact, { muted: !contact.muted })}
                onTogglePinned={() => void updateOneContact(contact, { pinned: !contact.pinned })}
                pendingLabel={contact.pendingRequest || contact.inboundRequest ? "Request pending" : undefined}
                pinned={contact.pinned}
              />
            )}
            selectedIds={selectedIds}
            selecting={selecting}
          />
        ) : null}

        {activePanel === "add" ? (
          <ContactAddDialog
            actionMessage={actionMessage}
            discoveryBusy={discoveryBusy}
            onClose={closePanel}
            onQueryChange={setAddQuery}
            onSearchUsers={() => void handleSearchUsers()}
            onSendFriendRequest={(targetIdentityId) => void handleSendFriendRequest(targetIdentityId)}
            query={addQuery}
            sendBusyIdentityId={sendBusyIdentityId}
            shortIdentity={shortIdentity}
            users={discoveryUsers}
          />
        ) : null}

        {blockTargets.length > 0 ? (
          <ContactBlockDialog
            busy={busy}
            onClose={() => setBlockTargets([])}
            onConfirm={() => void confirmBlockRemove()}
            targetLabel={blockTargets.length === 1 ? (blockTargets[0]?.name ?? "contact") : `${blockTargets.length} contacts`}
          />
        ) : null}
      </section>
    </MainLayout>
  );
}
