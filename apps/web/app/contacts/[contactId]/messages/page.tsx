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
import { fetchContacts } from "@/lib/api";
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
  const [message, setMessage] = useState("");
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

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

  function handleSend(): void {
    if (!canMessage) {
      setStatusMessage("Finish the contact request before sending a private message.");
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
            This node-routed private-message surface will send E2EE envelopes through the server delivery path.
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
            <p className={styles.state}>Finish the contact request before starting an encrypted conversation.</p>
          ) : null}

          {chatContact ? (
            <>
              <div className={styles.state}>
                <p className={styles.title}>Conversation starts here</p>
                <p className={styles.meta}>
                  The next backend slice will load E2EE DM thread history and send encrypted envelopes from this composer.
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
                  <button className={styles.pill} disabled={!canMessage} onClick={handleSend} type="button">
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
