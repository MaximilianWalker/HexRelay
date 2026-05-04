"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
import { QRCodeSVG } from "qrcode.react";
import {
  IconCircleCheck,
  IconCircleCheckFilled,
  IconClock,
  IconCopy,
  IconInfoCircle,
  IconLink,
  IconLock,
  IconMessageCircle,
  IconMessageCircleFilled,
  IconQrcode,
  IconSearch,
  IconShare3,
  IconStar,
  IconStarFilled,
  IconUser,
  IconUserPlus,
  IconUsers,
  IconX,
} from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  acceptFriendRequest,
  createContactInvite,
  createDmPairingEnvelope,
  declineFriendRequest,
  fetchContacts,
  fetchFriendRequests,
  importDmPairingEnvelope,
  redeemContactInvite,
} from "@/lib/api";
import { buildDmPairingLink, parseDmPairingInput } from "@/lib/dm-pairing";
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

type ActivePanel = "add" | "share" | null;

function buildInviteLink(token: string): string {
  return `hexrelay://contact-invite/${token}`;
}

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

function inviteModeDescription(mode: "one_time" | "multi_use"): string {
  if (mode === "multi_use") {
    return "Good for a small group or when you want the same invite to work more than once.";
  }

  return "Best for one person. It stops working after it is used.";
}

function formatDateTime(value?: string): string {
  if (!value) {
    return "No expiry shown";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

function formatApiError(code: string, message: string): string {
  const normalized = `${code} ${message}`.toLowerCase();

  if (normalized.includes("expired")) {
    return "This invite has expired. Ask for a new one.";
  }
  if (normalized.includes("invalid") || normalized.includes("parse")) {
    return "This invite does not look valid. Check the link or code and try again.";
  }
  if (normalized.includes("replay") || normalized.includes("already")) {
    return "This invite was already used or this contact action already exists.";
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

function extractInviteToken(rawToken: string): string | null {
  try {
    const maybeUrl = new URL(rawToken);
    if (maybeUrl.protocol === "hexrelay:" && maybeUrl.hostname === "contact-invite") {
      return maybeUrl.pathname.replace(/^\/+/, "").split("/").filter(Boolean)[0] ?? null;
    }
  } catch {
    // Not a URL; continue with token/path parsing below.
  }

  const withoutQueryOrFragment = rawToken.split(/[?#]/)[0];
  if (!withoutQueryOrFragment) {
    return null;
  }

  if (!withoutQueryOrFragment.includes("/")) {
    return withoutQueryOrFragment;
  }

  const segments = withoutQueryOrFragment.split("/").filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : null;
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
  const [activePanel, setActivePanel] = useState<ActivePanel>(null);
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
    target_identity_id: string;
    status: string;
  } | null>(null);
  const [linkCopied, setLinkCopied] = useState(false);
  const [pairingEndpointHints, setPairingEndpointHints] = useState("");
  const [pairingExpiresInSeconds, setPairingExpiresInSeconds] = useState("300");
  const [pairingBusy, setPairingBusy] = useState(false);
  const [pairingCopied, setPairingCopied] = useState(false);
  const [createdPairing, setCreatedPairing] = useState<{
    envelope: string;
    short_code: string;
    expires_at: string;
    pairing_nonce: string;
  } | null>(null);
  const [pairingImportValue, setPairingImportValue] = useState("");
  const [pairingImportBusy, setPairingImportBusy] = useState(false);
  const [pairingImportResult, setPairingImportResult] = useState<{
    inviter_identity_id: string;
    endpoint_hints: string[];
    imported_at: string;
    expires_at: string;
  } | null>(null);

  const personas = useMemo(() => readPersonas(), []);
  const identityId = useMemo(() => {
    const active = readActivePersonaId();
    if (active) {
      return active;
    }
    return personas[0]?.id ?? "usr-nora-k";
  }, [personas]);

  const activePersonaName = personas.find((persona) => persona.id === identityId)?.name ?? "your profile";
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

  function openPanel(panel: ActivePanel): void {
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
    setActionMessage("Contact request accepted.");
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
    setActionMessage("Contact request declined.");
    await refreshRequests();
  }

  async function handleCreateContactInvite(): Promise<void> {
    if (!hasSession) {
      setActionMessage("Create or select a profile before managing contacts.");
      return;
    }

    setActionMessage(null);
    setCreatedInvite(null);
    setLinkCopied(false);

    try {
      const maxUses = Number.parseInt(inviteMaxUses, 10);
      if (inviteMode === "multi_use" && (!Number.isInteger(maxUses) || maxUses < 1)) {
        setActionMessage("Reusable invites need at least one use.");
        return;
      }

      setInviteBusy(true);
      const result = await createContactInvite({
        mode: inviteMode,
        maxUses: inviteMode === "multi_use" && Number.isFinite(maxUses) ? maxUses : undefined,
      });

      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        return;
      }

      setCreatedInvite({
        token: result.data.token,
        mode: result.data.mode,
        expires_at: result.data.expires_at,
        max_uses: result.data.max_uses,
      });
      setActionMessage("Invite ready. Send it to someone you want to add.");
    } catch {
      setActionMessage("Could not create an invite. Try again in a moment.");
    } finally {
      setInviteBusy(false);
    }
  }

  async function handleRedeemContactInvite(): Promise<void> {
    if (!hasSession) {
      setActionMessage("Create or select a profile before managing contacts.");
      return;
    }

    setActionMessage(null);
    setRedeemResult(null);

    const tokenValue = extractInviteToken(redeemToken.trim());
    if (!tokenValue) {
      setActionMessage("Paste an invite link or code first.");
      return;
    }

    setRedeemBusy(true);
    try {
      const result = await redeemContactInvite({ token: tokenValue });

      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        return;
      }

      setRedeemResult({
        request_id: result.data.request_id,
        requester_identity_id: result.data.requester_identity_id,
        target_identity_id: result.data.target_identity_id,
        status: result.data.status,
      });
      setRedeemToken("");
      setActionMessage("Contact request sent. They will appear in your contacts after it is accepted.");
      await refreshRequests();
    } catch {
      setActionMessage("Could not use that invite. Try again in a moment.");
    } finally {
      setRedeemBusy(false);
    }
  }

  async function handleCopyLink(): Promise<void> {
    if (!createdInvite) {
      return;
    }

    try {
      await navigator.clipboard.writeText(buildInviteLink(createdInvite.token));
      setLinkCopied(true);
      setActionMessage("Invite copied. Send it to someone you want to add.");
    } catch {
      setActionMessage("Could not copy the invite link.");
    }
  }

  async function handleCreateAdvancedInvite(): Promise<void> {
    if (!hasSession) {
      setActionMessage("Create or select a profile before managing contacts.");
      return;
    }

    const endpointHints = pairingEndpointHints
      .split(/\r?\n|,/)
      .map((value) => value.trim())
      .filter(Boolean);
    const expiresInSeconds = Number.parseInt(pairingExpiresInSeconds.trim(), 10);

    if (endpointHints.length === 0) {
      setActionMessage("Add at least one connection address first.");
      return;
    }

    setPairingBusy(true);
    setActionMessage(null);
    setPairingCopied(false);

    try {
      const result = await createDmPairingEnvelope({
        endpointHints,
        expiresInSeconds: Number.isFinite(expiresInSeconds) ? expiresInSeconds : undefined,
      });

      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        return;
      }

      setCreatedPairing(result.data);
      setPairingImportResult(null);
      setActionMessage("Advanced invite ready. Share it only with the person you want to connect with.");
    } catch {
      setActionMessage("Could not create advanced connection details.");
    } finally {
      setPairingBusy(false);
    }
  }

  async function handleCopyAdvancedInvite(): Promise<void> {
    if (!createdPairing) {
      return;
    }

    try {
      await navigator.clipboard.writeText(buildDmPairingLink(createdPairing.envelope));
      setPairingCopied(true);
      setActionMessage("Advanced invite copied.");
    } catch {
      setActionMessage("Could not copy the advanced invite.");
    }
  }

  async function handleImportAdvancedInvite(): Promise<void> {
    if (!hasSession) {
      setActionMessage("Create or select a profile before managing contacts.");
      return;
    }

    const envelope = parseDmPairingInput(pairingImportValue);
    if (!envelope) {
      setActionMessage("Paste advanced connection details first.");
      return;
    }

    setPairingImportBusy(true);
    setActionMessage(null);

    try {
      const result = await importDmPairingEnvelope({ envelope });
      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        return;
      }

      setPairingImportResult(result.data);
      setPairingImportValue("");
      setActionMessage("Advanced connection details saved.");
    } catch {
      setActionMessage("Could not read those advanced connection details.");
    } finally {
      setPairingImportBusy(false);
    }
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
      subtitle="People you know, contact requests, and simple invite tools"
      tabs={[
        { id: "contacts", label: "All contacts" },
        { id: "requests", label: "Requests" },
        { id: "invites", label: "Invites" },
      ]}
      tabActions={
        <>
          <button className={styles.pill} disabled={!hasSession} onClick={() => openPanel("add")} type="button">
            <IconUserPlus className={styles.icon} aria-hidden="true" />
            Add contact
          </button>
          <button className={styles.pill} disabled={!hasSession} onClick={() => openPanel("share")} type="button">
            <IconShare3 className={styles.icon} aria-hidden="true" />
            Share invite
          </button>
        </>
      }
      title="Contacts"
    >
      <section>
        {activePanel === "add" ? (
          <section className={styles.state} aria-label="Add contact">
            <p className={styles.title}>Add contact</p>
            <p className={styles.meta}>Paste an invite link or code from someone you know.</p>
            <div className={styles.inputWrap}>
              <IconLink className={styles.inputIcon} aria-hidden="true" />
            <input
              className={styles.search}
              onChange={(event) => setRedeemToken(event.target.value)}
              placeholder="Invite link or code"
              value={redeemToken}
            />
            </div>
            <div className={styles.row}>
              <button
                className={styles.pill}
                disabled={redeemBusy}
                onClick={() => void handleRedeemContactInvite()}
                type="button"
              >
                <IconCircleCheck className={styles.icon} aria-hidden="true" />
                {redeemBusy ? "Checking invite..." : "Continue"}
              </button>
              <button className={styles.pill} onClick={() => openPanel(null)} type="button">
                <IconX className={styles.icon} aria-hidden="true" />
                Close
              </button>
            </div>

            {redeemResult ? (
              <div className={styles.card} style={{ marginTop: 12 }}>
                <p className={styles.title}>Request sent</p>
                <p className={styles.meta}>
                  Your request was sent to {identityLabel(redeemResult.target_identity_id, identityId, personas)}.
                  They need to accept before they appear in your contacts.
                </p>
                <details className={styles.state}>
                  <summary><IconInfoCircle className={styles.icon} aria-hidden="true" /> Show technical details</summary>
                  <p className={styles.meta}>Request ID: {redeemResult.request_id}</p>
                  <p className={styles.meta}>Status: {redeemResult.status}</p>
                </details>
              </div>
            ) : null}

            <details className={styles.state}>
              <summary><IconLock className={styles.icon} aria-hidden="true" /> Advanced connection setup</summary>
              <p className={styles.meta}>
                Use this only if someone sent you manual direct-connection details.
              </p>
              <textarea
                className={styles.search}
                onChange={(event) => setPairingImportValue(event.target.value)}
                placeholder="Paste advanced invite or connection details"
                rows={3}
                value={pairingImportValue}
              />
              <button
                className={styles.pill}
                disabled={pairingImportBusy}
                onClick={() => void handleImportAdvancedInvite()}
                type="button"
              >
                <IconCircleCheck className={styles.icon} aria-hidden="true" />
                {pairingImportBusy ? "Reading details..." : "Use advanced invite"}
              </button>

              {pairingImportResult ? (
                <div className={styles.card} style={{ marginTop: 12 }}>
                  <p className={styles.title}>Advanced connection saved</p>
                  <p className={styles.meta}>
                    From {identityLabel(pairingImportResult.inviter_identity_id, identityId, personas)}.
                  </p>
                  <details className={styles.state}>
                    <summary><IconInfoCircle className={styles.icon} aria-hidden="true" /> Show technical details</summary>
                    <p className={styles.meta}>Imported: {formatDateTime(pairingImportResult.imported_at)}</p>
                    <p className={styles.meta}>Expires: {formatDateTime(pairingImportResult.expires_at)}</p>
                    <p className={styles.meta} style={{ wordBreak: "break-all" }}>
                      Connection details: {pairingImportResult.endpoint_hints.join(", ")}
                    </p>
                  </details>
                </div>
              ) : null}
            </details>
          </section>
        ) : null}

        {activePanel === "share" ? (
          <section className={styles.state} aria-label="Share invite">
            <p className={styles.title}>Share invite</p>
            <p className={styles.meta}>Create an invite for {activePersonaName}. Send it to someone you want to add.</p>
            <div className={styles.row}>
              <button className={styles.pill} onClick={() => setInviteMode("one_time")} type="button">
                <IconUser className={styles.icon} aria-hidden="true" />
                Single-use {inviteMode === "one_time" ? "selected" : ""}
              </button>
              <button className={styles.pill} onClick={() => setInviteMode("multi_use")} type="button">
                <IconUsers className={styles.icon} aria-hidden="true" />
                Reusable {inviteMode === "multi_use" ? "selected" : ""}
              </button>
            </div>
            <p className={styles.meta}>{inviteModeDescription(inviteMode)}</p>
            {inviteMode === "multi_use" ? (
              <input
                className={styles.search}
                min={1}
                onChange={(event) => setInviteMaxUses(event.target.value)}
                placeholder="How many people can use it?"
                step={1}
                type="number"
                value={inviteMaxUses}
              />
            ) : null}
            <div className={styles.row}>
              <button
                className={styles.pill}
                disabled={inviteBusy}
                onClick={() => void handleCreateContactInvite()}
                type="button"
              >
                <IconShare3 className={styles.icon} aria-hidden="true" />
                {inviteBusy ? "Creating invite..." : "Create invite"}
              </button>
              <button className={styles.pill} onClick={() => openPanel(null)} type="button">
                <IconX className={styles.icon} aria-hidden="true" />
                Close
              </button>
            </div>

            {createdInvite ? (
              <div className={styles.card} style={{ marginTop: 12 }}>
                <p className={styles.title}>Invite ready</p>
                <p className={styles.meta}>Copy the link or show the QR code to the person you want to add.</p>
                <p className={styles.meta} style={{ wordBreak: "break-all", marginTop: 6 }}>
                  {buildInviteLink(createdInvite.token)}
                </p>
                <div className={styles.row} style={{ marginTop: 8 }}>
                  <button className={styles.pill} onClick={() => void handleCopyLink()} type="button">
                    <IconCopy className={styles.icon} aria-hidden="true" />
                    {linkCopied ? "Copied" : "Copy invite link"}
                  </button>
                </div>
                <div style={{ marginTop: 8, display: "flex", justifyContent: "center" }}>
                  <QRCodeSVG value={buildInviteLink(createdInvite.token)} size={160} level="M" />
                </div>
                <details className={styles.state}>
                  <summary><IconQrcode className={styles.icon} aria-hidden="true" /> Invite settings</summary>
                  <p className={styles.meta}>
                    Type: {createdInvite.mode === "multi_use" ? "Reusable" : "Single-use"}
                  </p>
                  {createdInvite.max_uses != null ? (
                    <p className={styles.meta}>Max uses: {createdInvite.max_uses}</p>
                  ) : null}
                  <p className={styles.meta}>Expires: {formatDateTime(createdInvite.expires_at)}</p>
                </details>
              </div>
            ) : null}

            <details className={styles.state}>
              <summary><IconLock className={styles.icon} aria-hidden="true" /> Advanced connection setup</summary>
              <p className={styles.meta}>
                Use this for manual direct-connection setup. Most people should use the normal invite above.
              </p>
              <textarea
                className={styles.search}
                onChange={(event) => setPairingEndpointHints(event.target.value)}
                placeholder="Connection addresses, one per line"
                rows={3}
                value={pairingEndpointHints}
              />
              <input
                className={styles.search}
                onChange={(event) => setPairingExpiresInSeconds(event.target.value)}
                placeholder="Expires in seconds"
                value={pairingExpiresInSeconds}
              />
              <button
                className={styles.pill}
                disabled={pairingBusy}
                onClick={() => void handleCreateAdvancedInvite()}
                type="button"
              >
                <IconLock className={styles.icon} aria-hidden="true" />
                {pairingBusy ? "Creating advanced invite..." : "Create advanced invite"}
              </button>

              {createdPairing ? (
                <div className={styles.card} style={{ marginTop: 12 }}>
                  <p className={styles.title}>Advanced invite ready</p>
                  <p className={styles.meta}>Share this only with the person you want to connect with.</p>
                  <div className={styles.row} style={{ marginTop: 8 }}>
                    <button className={styles.pill} onClick={() => void handleCopyAdvancedInvite()} type="button">
                      <IconCopy className={styles.icon} aria-hidden="true" />
                      {pairingCopied ? "Copied" : "Copy advanced invite"}
                    </button>
                  </div>
                  <div style={{ marginTop: 8, display: "flex", justifyContent: "center" }}>
                    <QRCodeSVG value={buildDmPairingLink(createdPairing.envelope)} level="M" size={160} />
                  </div>
                  <details className={styles.state}>
                    <summary><IconInfoCircle className={styles.icon} aria-hidden="true" /> Show technical details</summary>
                    <p className={styles.meta}>Short code: {createdPairing.short_code}</p>
                    <p className={styles.meta}>Expires: {formatDateTime(createdPairing.expires_at)}</p>
                    <p className={styles.meta}>Security detail: {createdPairing.pairing_nonce}</p>
                    <p className={styles.meta} style={{ wordBreak: "break-all" }}>
                      Raw invite: {buildDmPairingLink(createdPairing.envelope)}
                    </p>
                  </details>
                </div>
              ) : null}
            </details>
          </section>
        ) : null}

        {actionMessage ? <p className={styles.state}>{actionMessage}</p> : null}

        {inboundPending.length > 0 ? (
          <section className={styles.state} aria-label="Contact requests">
            <p className={styles.title}>Contact requests</p>
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
                    <p className={styles.meta}>Accept to add them to your contacts.</p>
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
            <p className={styles.meta}>People who still need to accept your contact request.</p>
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
            <p className={styles.meta}>Add someone you know to start a private conversation.</p>
            <div className={styles.row} style={{ marginTop: 10 }}>
              <button className={styles.pill} onClick={() => openPanel("add")} type="button">
                <IconUserPlus className={styles.icon} aria-hidden="true" />
                Add your first contact
              </button>
              <button className={styles.pill} onClick={() => openPanel("share")} type="button">
                <IconShare3 className={styles.icon} aria-hidden="true" />
                Share your invite
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
                    {contact.unread > 0 ? <span className={styles.badge}><IconMessageCircle className={styles.icon} aria-hidden="true" />{contact.unread} unread</span> : null}
                    {contact.favorite ? <span className={styles.badgeMuted}><IconStar className={styles.icon} aria-hidden="true" />Favorite</span> : null}
                    {contact.pendingRequest ? <span className={styles.badgeMuted}><IconClock className={styles.icon} aria-hidden="true" />Request pending</span> : null}
                    {contact.inboundRequest ? <span className={styles.badge}><IconInfoCircle className={styles.icon} aria-hidden="true" />Needs approval</span> : null}
                  </div>
                  <details className={styles.compactDetails}>
                    <summary><IconInfoCircle className={styles.icon} aria-hidden="true" /> Contact details</summary>
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
