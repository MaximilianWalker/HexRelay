"use client";

import { useParams } from "next/navigation";
import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import {
  IconCircleCheck,
  IconInfoCircle,
  IconMessageCircle,
  IconPinned,
  IconPinnedOff,
  IconVolume,
  IconVolumeOff,
} from "@tabler/icons-react";

import { Composer } from "@/components/chat/composer";
import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Notice } from "@/components/ui/notice";
import { WorkspaceShell } from "@/components/workspace-shell";
import { fetchContacts, updateContactPreferences } from "@/lib/api";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";

import styles from "../../../surfaces.module.css";

type Contact = {
  id: string;
  name: string;
  status: string;
  unread: number;
  pinned: boolean;
  muted: boolean;
  inboundRequest: boolean;
  pendingRequest: boolean;
};

type ContactLoad = {
  contactId: string;
  contact: Contact | null;
  error: string | null;
};

type ContactPreferenceAction = "pin" | "mute";

type ContactApiItem = {
  id: string;
  name: string;
  status: string;
  unread: number;
  pinned: boolean;
  muted: boolean;
  inbound_request?: boolean;
  pending_request?: boolean;
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

function statusLabel(status: string): string {
  if (status === "online") {
    return "Online";
  }
  if (status === "away") {
    return "Away";
  }

  return "Offline";
}

function mapContact(item: ContactApiItem): Contact {
  return {
    id: item.id,
    name: item.name,
    status: item.status,
    unread: item.unread,
    pinned: item.pinned,
    muted: item.muted,
    inboundRequest: item.inbound_request ?? false,
    pendingRequest: item.pending_request ?? false,
  };
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
  const [busyPreferenceAction, setBusyPreferenceAction] = useState<ContactPreferenceAction | null>(null);

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
        contact: matched ? mapContact(matched) : null,
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
  const contactBadges = contact ? (
    <>
      <Badge
        icon={<IconCircleCheck className={styles.icon} aria-hidden="true" />}
        tone={contact.status === "online" ? "success" : "muted"}
      >
        {statusLabel(contact.status)}
      </Badge>
      {contact.pinned ? <Badge tone="accent">Pinned</Badge> : null}
      {contact.muted ? <Badge tone="muted">Muted</Badge> : null}
      {contact.pendingRequest ? <Badge tone="warning">Request pending</Badge> : null}
      {contact.inboundRequest ? <Badge tone="warning">Needs approval</Badge> : null}
    </>
  ) : (
    <Badge tone="warning">Unknown contact</Badge>
  );

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

  async function updateContactPreference(
    update: { pinned?: boolean; muted?: boolean },
    action: ContactPreferenceAction,
  ): Promise<void> {
    if (!contact) {
      return;
    }

    setBusyPreferenceAction(action);
    setStatusMessage(null);

    const result = await updateContactPreferences({ contactId: contact.id, ...update });
    if (!result.ok) {
      setStatusMessage(result.message);
      setBusyPreferenceAction(null);
      return;
    }

    const updatedContact = mapContact(result.data);
    setContactLoad({ contactId: updatedContact.id, contact: updatedContact, error: null });
    setStatusMessage(
      update.pinned !== undefined
        ? updatedContact.pinned
          ? "Contact pinned."
          : "Contact unpinned."
        : updatedContact.muted
          ? "Conversation muted."
          : "Conversation unmuted.",
    );
    setBusyPreferenceAction(null);
  }

  return (
    <WorkspaceShell
      activeTabId="contacts"
      subtitle={`Private conversation with ${title}`}
      tabs={[]}
      title="Private Chat"
      workspaceTab={{ imageLabel: title, label: title, unread: contact?.unread }}
    >
      <article className={styles.conversationPage}>
        <header className={styles.conversationTopbar}>
          <div className={styles.conversationIdentity}>
            <Avatar kind="user" label={title} size="md" text={contactInitials(title)} />
            <div className={styles.conversationCopy}>
              <div className={styles.conversationTitleRow}>
                <h2 className={styles.conversationTitle}>{title}</h2>
                <div className={styles.conversationBadges}>{contactBadges}</div>
              </div>
              <p className={styles.conversationMeta}>{shortIdentity(contactId ?? "Unknown contact")}</p>
            </div>
          </div>

          <div className={styles.conversationActions}>
            {contact ? (
              <>
                <Button
                  disabled={busyPreferenceAction !== null}
                  icon={
                    contact.pinned ? (
                      <IconPinnedOff className={styles.icon} aria-hidden="true" />
                    ) : (
                      <IconPinned className={styles.icon} aria-hidden="true" />
                    )
                  }
                  onClick={() => void updateContactPreference({ pinned: !contact.pinned }, "pin")}
                  size="sm"
                >
                  {contact.pinned ? "Unpin" : "Pin"}
                </Button>
                <Button
                  disabled={busyPreferenceAction !== null}
                  icon={
                    contact.muted ? (
                      <IconVolume className={styles.icon} aria-hidden="true" />
                    ) : (
                      <IconVolumeOff className={styles.icon} aria-hidden="true" />
                    )
                  }
                  onClick={() => void updateContactPreference({ muted: !contact.muted }, "mute")}
                  size="sm"
                >
                  {contact.muted ? "Unmute" : "Mute"}
                </Button>
              </>
            ) : null}
          </div>
        </header>

        <section className={styles.conversationBody}>
          {loading ? <Notice>Loading conversation...</Notice> : null}
          {!hasSession ? <Notice tone="warning">Create or select a profile before messaging contacts.</Notice> : null}
          {!contactId ? <Notice tone="danger">This contact link is invalid.</Notice> : null}
          {loadError ? <Notice tone="danger">{loadError}</Notice> : null}
          {!loading && hasSession && contactId && !loadError && !contact ? (
            <Notice tone="warning">This contact was not found in your current contacts list.</Notice>
          ) : null}
          {contact && !canMessage ? (
            <Notice tone="warning">Finish the contact request before starting an encrypted conversation.</Notice>
          ) : null}

          {chatContact ? (
            <>
              <div className={styles.conversationEmptyState}>
                <IconMessageCircle className={styles.conversationEmptyIcon} aria-hidden="true" />
                <div>
                  <p className={styles.title}>Conversation starts here</p>
                  <p className={styles.meta}>
                    The next backend slice will load E2EE DM thread history and send encrypted envelopes from this
                    composer.
                  </p>
                </div>
              </div>

              <Composer
                disabled={!canMessage}
                hints={
                  <span className={styles.conversationComposerHint}>
                    Server-routed E2EE envelopes will use the delivery path for this contact.
                  </span>
                }
                onChange={setMessage}
                onSend={handleSend}
                placeholder={`Message ${title}`}
                sendLabel="Send"
                value={message}
              />
            </>
          ) : null}

          {statusMessage ? (
            <Notice icon={<IconInfoCircle className={styles.icon} aria-hidden="true" />}>{statusMessage}</Notice>
          ) : null}

          <details className={styles.conversationDetails}>
            <summary>Contact details</summary>
            <p className={styles.conversationDetailText}>
              Contact ID: {contactId ?? "Invalid contact link"}
            </p>
          </details>
        </section>
      </article>
    </WorkspaceShell>
  );
}
