"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import {
  IconArrowLeft,
  IconCircleCheck,
  IconInfoCircle,
  IconMessageCircle,
  IconSend,
  IconUser,
} from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  fetchContacts,
  runDmConnectivityPreflight,
  type DmConnectivityPreflightResponse,
} from "@/lib/api";
import {
  preflightReasonLabel,
  readDmPairingImport,
  type DmPairingImportRecord,
} from "@/lib/dm-connectivity";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";

import styles from "../../../surfaces.module.css";

type Contact = {
  id: string;
  name: string;
  status: string;
  unread: number;
  favorite: boolean;
  inboundRequest: boolean;
  pendingRequest: boolean;
};

type ContactLoad = {
  contactId: string;
  contact: Contact | null;
  error: string | null;
};

type PreflightLoad =
  | { state: "idle" }
  | { state: "loading" }
  | { state: "ready"; result: DmConnectivityPreflightResponse }
  | { state: "error"; message: string };

function safeContactId(value: string): string | null {
  try {
    return decodeURIComponent(value);
  } catch {
    return null;
  }
}

function shortIdentity(identityId: string): string {
  if (identityId.length <= 18) {
    return identityId;
  }

  return `${identityId.slice(0, 8)}...${identityId.slice(-6)}`;
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

function formatDateTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

function subscribeBrowserReady(): () => void {
  return () => {};
}

function getBrowserReadySnapshot(): "client" {
  return "client";
}

function getBrowserReadyServerSnapshot(): "server" {
  return "server";
}

export default function ContactMessagesPage() {
  const params = useParams<{ contactId: string }>();
  const contactId = safeContactId(params.contactId);
  const browserReady = useSyncExternalStore(
    subscribeBrowserReady,
    getBrowserReadySnapshot,
    getBrowserReadyServerSnapshot,
  ) === "client";
  const personas = useMemo(() => (browserReady ? readPersonas() : []), [browserReady]);
  const identityId = useMemo(
    () => (browserReady ? readActivePersonaId() ?? personas[0]?.id ?? "usr-nora-k" : "usr-nora-k"),
    [browserReady, personas],
  );
  const hasSession = useMemo(() => browserReady && getPersonaSession(identityId) !== null, [browserReady, identityId]);
  const [contactLoad, setContactLoad] = useState<ContactLoad | null>(null);
  const [preflightRunId, setPreflightRunId] = useState(0);
  const [preflight, setPreflight] = useState<PreflightLoad>({ state: "idle" });
  const [message, setMessage] = useState("");
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const pairingImport = useMemo<DmPairingImportRecord | null>(
    () => (browserReady && contactId ? readDmPairingImport(contactId) : null),
    [browserReady, contactId],
  );

  useEffect(() => {
    let active = true;

    if (!hasSession) {
      return () => {
        active = false;
      };
    }
    if (!contactId) {
      return () => {
        active = false;
      };
    }

    const run = async (): Promise<void> => {
      const result = await fetchContacts({});
      if (!active) {
        return;
      }

      if (!result.ok) {
        setContactLoad({ contactId, contact: null, error: "Could not load this contact right now." });
        return;
      }

      const matched = result.data.items.find((item) => item.id === contactId);
      setContactLoad({
        contactId,
        contact: matched
          ? {
              id: matched.id,
              name: matched.name,
              status: matched.status,
              unread: matched.unread,
              favorite: matched.favorite,
              inboundRequest: matched.inbound_request,
              pendingRequest: matched.pending_request,
            }
          : null,
        error: null,
      });
    };

    void run();

    return () => {
      active = false;
    };
  }, [contactId, hasSession]);

  const currentLoad = contactLoad?.contactId === contactId ? contactLoad : null;
  const contact = currentLoad?.contact ?? null;
  const loading = hasSession && contactId !== null && currentLoad === null;
  const loadError = currentLoad?.error ?? null;
  const title = contact?.name ?? shortIdentity(contactId ?? "Unknown contact");
  const chatContact = contact && !contact.inboundRequest && !contact.pendingRequest ? contact : null;
  const canMessage = chatContact !== null;

  useEffect(() => {
    let active = true;

    if (!hasSession || !chatContact) {
      return () => {
        active = false;
      };
    }

    const run = async (): Promise<void> => {
      setPreflight({ state: "loading" });
      const result = await runDmConnectivityPreflight({
        peerIdentityId: chatContact.id,
        pairingEnvelopePresent: pairingImport !== null,
        localBindAllowed: true,
        peerReachableHint: chatContact.status === "online",
      });

      if (!active) {
        return;
      }

      if (!result.ok) {
        setPreflight({ state: "error", message: result.message });
        return;
      }

      setPreflight({ state: "ready", result: result.data });
    };

    void run();

    return () => {
      active = false;
    };
  }, [chatContact, hasSession, pairingImport, preflightRunId]);

  const preflightResult = preflight.state === "ready" ? preflight.result : null;
  const directPathReady = preflightResult?.status === "ready";

  function handleSend(): void {
    if (!directPathReady) {
      setStatusMessage("Resolve direct-connect preflight before sending a private message.");
      return;
    }

    setStatusMessage(
      message.trim()
        ? "Message composer is ready, but delivery is not wired in this build yet."
        : "Write a message first.",
    );
  }

  return (
    <WorkspaceShell
      activeTabId="contacts"
      subtitle={`Private conversation with ${title}`}
      tabs={[
        { id: "contacts", label: "All contacts", icon: IconUser },
        { id: "messages", label: "Private chat", icon: IconMessageCircle },
      ]}
      title="Private Chat"
    >
      <section className={styles.channelLayout}>
        <aside className={styles.channelRail}>
          <Link className={styles.pill} href="/contacts">
            <IconArrowLeft className={styles.icon} aria-hidden="true" />
            Back to contacts
          </Link>
          <div className={styles.cardHeader}>
            <div className={styles.avatar}>{contactInitials(title)}</div>
            <div>
              <p className={styles.title}>{title}</p>
              <p className={styles.meta}>{shortIdentity(contactId ?? "Unknown contact")}</p>
            </div>
          </div>
          <div className={styles.row}>
            {contact ? (
              <span className={contact.status === "online" ? styles.badge : styles.badgeMuted}>
                <IconCircleCheck className={styles.icon} aria-hidden="true" />
                {contact.status}
              </span>
            ) : null}
            {contact?.favorite ? <span className={styles.badgeMuted}>Favorite</span> : null}
            {contact?.pendingRequest ? <span className={styles.badgeMuted}>Request pending</span> : null}
            {contact?.inboundRequest ? <span className={styles.badge}>Needs approval</span> : null}
          </div>
          <p className={styles.meta}>
            This is the first private chat surface. Message delivery will connect to the DM transport next.
          </p>
        </aside>

        <article className={styles.channelMain}>
          {loading ? <p className={styles.state}>Loading conversation...</p> : null}
          {!hasSession ? <p className={styles.state}>Create or select a profile before messaging contacts.</p> : null}
          {!contactId ? <p className={styles.state}>This contact link is invalid.</p> : null}
          {loadError ? <p className={styles.state}>{loadError}</p> : null}
          {!loading && hasSession && contactId && !loadError && !contact ? (
            <p className={styles.state}>This contact was not found in your current contacts list.</p>
          ) : null}
          {contact && !canMessage ? (
            <p className={styles.state}>Finish the contact request before starting a private chat.</p>
          ) : null}

          {chatContact ? (
            <>
              <div className={styles.card} style={{ marginTop: 12 }}>
                <p className={styles.title}>
                  <IconInfoCircle className={styles.icon} aria-hidden="true" /> Direct-connect preflight
                </p>
                <p className={styles.meta}>
                  Combines session pairing state, local app hints, and trusted DM policy checks before this chat uses direct-only transport.
                </p>
                <div className={styles.row}>
                  {pairingImport ? (
                    <span className={styles.badge}>
                      <IconCircleCheck className={styles.icon} aria-hidden="true" /> Pairing imported
                    </span>
                  ) : (
                    <span className={styles.badgeMuted}>Pairing not imported</span>
                  )}
                  <span className={chatContact.status === "online" ? styles.badge : styles.badgeMuted}>
                    Peer {chatContact.status}
                  </span>
                  <span className={styles.badgeMuted}>Transport: direct only</span>
                </div>

                {pairingImport ? (
                  <details className={styles.compactDetails}>
                    <summary>
                      <IconInfoCircle className={styles.icon} aria-hidden="true" /> Pairing details
                    </summary>
                    <p className={styles.meta}>Expires: {formatDateTime(pairingImport.expiresAt)}</p>
                    <p className={styles.meta} style={{ wordBreak: "break-all" }}>
                      Fingerprint: {pairingImport.inviterIdentityKey.fingerprint}
                    </p>
                    <p className={styles.meta} style={{ wordBreak: "break-all" }}>
                      Endpoints: {pairingImport.endpointHints.join(", ") || "No endpoint hints"}
                    </p>
                  </details>
                ) : null}

                {preflight.state === "loading" ? <p className={styles.state}>Running preflight...</p> : null}
                {preflight.state === "error" ? <p className={styles.state}>{preflight.message}</p> : null}
                {preflightResult ? (
                  <div className={styles.state}>
                    <p className={styles.title}>{preflightReasonLabel(preflightResult.reason_code)}</p>
                    <p className={styles.meta}>Reason code: {preflightResult.reason_code}</p>
                    <ul>
                      {preflightResult.remediation.map((step) => (
                        <li className={styles.meta} key={step}>{step}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}

                <button className={styles.pill} onClick={() => setPreflightRunId((value) => value + 1)} type="button">
                  <IconInfoCircle className={styles.icon} aria-hidden="true" /> Rerun preflight
                </button>
              </div>

              <div className={styles.state}>
                <p className={styles.title}>Conversation starts here</p>
                <p className={styles.meta}>
                  {directPathReady
                    ? "The direct path is ready. The next backend slice will load DM thread history and send messages from this composer."
                    : "The private chat route exists, but direct-connect troubleshooting must pass before sending."}
                </p>
              </div>

              <div className={styles.card} style={{ marginTop: 12 }}>
                <p className={styles.title}>
                  <IconMessageCircle className={styles.icon} aria-hidden="true" /> Message composer
                </p>
                <textarea
                  className={styles.search}
                  id="dm-message-composer"
                  name="message"
                  onChange={(event) => setMessage(event.target.value)}
                  placeholder={`Message ${title}`}
                  rows={4}
                  value={message}
                />
                <div className={styles.row}>
                  <button className={styles.pill} disabled={!directPathReady} onClick={handleSend} type="button">
                    <IconSend className={styles.icon} aria-hidden="true" />
                    Send
                  </button>
                </div>
              </div>
            </>
          ) : null}

          {statusMessage ? (
            <p className={styles.state}>
              <IconInfoCircle className={styles.icon} aria-hidden="true" /> {statusMessage}
            </p>
          ) : null}

          <details className={styles.compactDetails}>
            <summary>
              <IconUser className={styles.icon} aria-hidden="true" /> Contact details
            </summary>
            <p className={styles.meta} style={{ wordBreak: "break-all" }}>
              Contact ID: {contactId ?? "Invalid contact link"}
            </p>
          </details>
        </article>
      </section>
    </WorkspaceShell>
  );
}
