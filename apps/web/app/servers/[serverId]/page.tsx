"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import {
  IconArrowLeft,
  IconBell,
  IconBellOff,
  IconHash,
  IconInfoCircle,
  IconMessageCircle,
  IconRefresh,
  IconSend,
  IconServer2,
  IconStar,
} from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  createServerChannelMessage,
  fetchServer,
  fetchServerChannelMessages,
  fetchServerChannels,
  type ServerChannelMessage,
  type ServerChannelSummary,
  type ServerSummary,
} from "@/lib/api";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";

import styles from "../../surfaces.module.css";

const MESSAGE_PAGE_LIMIT = 30;

const SEEDED_IDENTITIES: Record<string, { name: string; handle: string }> = {
  "usr-test-alice": { name: "Alice", handle: "alice" },
  "usr-test-bob": { name: "Bob", handle: "bob" },
  "usr-test-carol": { name: "Carol", handle: "carol" },
};

const PREVIEW_SERVER: ServerSummary = {
  id: "fixture-server-atlas",
  name: "Atlas Test Server",
  unread: 2,
  favorite: true,
  muted: false,
};

const PREVIEW_CHANNELS: ServerChannelSummary[] = [
  {
    id: "fixture-channel-atlas-general",
    name: "general",
    kind: "text",
    last_message_seq: 3,
  },
  {
    id: "fixture-channel-atlas-ops-lab",
    name: "ops-lab",
    kind: "text",
    last_message_seq: 2,
  },
];

const PREVIEW_MESSAGES: ServerChannelMessage[] = [
  {
    message_id: "fixture-server-message-general-001",
    channel_id: "fixture-channel-atlas-general",
    author_id: "usr-test-alice",
    channel_seq: 1,
    content: "Welcome to Atlas.",
    reply_to_message_id: null,
    mentions: [],
    created_at: "2026-05-04T11:10:00Z",
    edited_at: null,
    deleted_at: null,
  },
  {
    message_id: "fixture-server-message-general-002",
    channel_id: "fixture-channel-atlas-general",
    author_id: "usr-test-bob",
    channel_seq: 2,
    content: "Hi Carol, the shared server fixture is ready.",
    reply_to_message_id: null,
    mentions: ["usr-test-carol"],
    created_at: "2026-05-04T11:11:00Z",
    edited_at: null,
    deleted_at: null,
  },
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
  {
    message_id: "fixture-server-message-ops-001",
    channel_id: "fixture-channel-atlas-ops-lab",
    author_id: "usr-test-alice",
    channel_seq: 1,
    content: "Ops lab is online.",
    reply_to_message_id: null,
    mentions: [],
    created_at: "2026-05-04T11:20:00Z",
    edited_at: null,
    deleted_at: null,
  },
  {
    message_id: "fixture-server-message-ops-002",
    channel_id: "fixture-channel-atlas-ops-lab",
    author_id: "usr-test-bob",
    channel_seq: 2,
    content: "Tracking the ops setup with Alice.",
    reply_to_message_id: "fixture-server-message-ops-001",
    mentions: ["usr-test-alice"],
    created_at: "2026-05-04T11:21:00Z",
    edited_at: null,
    deleted_at: null,
  },
];

type LoadState = "idle" | "loading" | "ready" | "error";
type ServerView = "overview" | "chat";

function subscribeBrowserReady(): () => void {
  return () => {};
}

function getBrowserReadySnapshot(): "client" {
  return "client";
}

function getBrowserReadyServerSnapshot(): "server" {
  return "server";
}

function decodePathParam(value: string): string | null {
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

function authorLabel(identityId: string): string {
  return SEEDED_IDENTITIES[identityId]?.name ?? shortIdentity(identityId);
}

function authorHandle(identityId: string): string {
  return SEEDED_IDENTITIES[identityId]?.handle ?? shortIdentity(identityId);
}

function initials(value: string): string {
  const parts = value.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date);
}

function formatApiError(code: string, message: string): string {
  return `${code}: ${message}`;
}

function mergeMessages(
  current: ServerChannelMessage[],
  incoming: ServerChannelMessage[],
): ServerChannelMessage[] {
  const byId = new Map<string, ServerChannelMessage>();
  for (const item of current) {
    byId.set(item.message_id, item);
  }
  for (const item of incoming) {
    byId.set(item.message_id, item);
  }

  return [...byId.values()].sort((a, b) => a.channel_seq - b.channel_seq);
}

function extractMentionIdentityIds(content: string): string[] {
  const matches = new Set<string>();
  const tokens = content.toLowerCase().match(/@[a-z0-9_-]+/g) ?? [];

  for (const token of tokens) {
    const handle = token.slice(1);
    const directIdentity = Object.keys(SEEDED_IDENTITIES).find(
      (identityId) => identityId.toLowerCase() === handle,
    );
    if (directIdentity) {
      matches.add(directIdentity);
      continue;
    }

    const seededIdentity = Object.entries(SEEDED_IDENTITIES).find(
      ([, identity]) => identity.handle === handle,
    )?.[0];
    if (seededIdentity) {
      matches.add(seededIdentity);
    }
  }

  return [...matches].sort();
}

function visibleMessagesForChannel(
  messages: ServerChannelMessage[],
  channelId: string | null,
): ServerChannelMessage[] {
  if (!channelId) {
    return [];
  }

  return messages.filter((message) => message.channel_id === channelId);
}

function latestMessage(messages: ServerChannelMessage[]): ServerChannelMessage | null {
  return messages.reduce<ServerChannelMessage | null>((latest, message) => {
    if (!latest || message.created_at > latest.created_at) {
      return message;
    }

    return latest;
  }, null);
}

function ServerIcon({ name }: { name: string }) {
  return (
    <div className={styles.serverImage} aria-label={`${name} icon`} role="img">
      <div className={styles.serverImageGrid} aria-hidden="true" />
      <span>{initials(name)}</span>
    </div>
  );
}

function ChannelButton({
  channel,
  active,
  onSelect,
}: {
  channel: ServerChannelSummary;
  active: boolean;
  onSelect: (channelId: string) => void;
}) {
  return (
    <button
      aria-pressed={active}
      className={`${styles.channelButton} ${active ? styles.channelButtonActive : ""}`}
      onClick={() => onSelect(channel.id)}
      type="button"
    >
      <IconHash className={styles.icon} aria-hidden="true" />
      <span>{channel.name}</span>
      <span className={styles.channelSeq}>seq {channel.last_message_seq}</span>
    </button>
  );
}

function MessageBubble({
  message,
  replyTo,
  ownMessage,
  onReply,
}: {
  message: ServerChannelMessage;
  replyTo?: ServerChannelMessage;
  ownMessage: boolean;
  onReply: (message: ServerChannelMessage) => void;
}) {
  const deleted = Boolean(message.deleted_at);

  return (
    <article className={`${styles.serverMessage} ${ownMessage ? styles.serverMessageOwn : ""}`}>
      <div className={styles.messageAvatar}>{initials(authorLabel(message.author_id))}</div>
      <div className={styles.messageBody}>
        <div className={styles.messageHeader}>
          <span className={styles.messageAuthor}>{authorLabel(message.author_id)}</span>
          <span className={styles.messageTime}>{formatTimestamp(message.created_at)}</span>
          {message.edited_at ? <span className={styles.messageFlag}>edited</span> : null}
          {deleted ? <span className={styles.messageFlag}>deleted</span> : null}
        </div>
        {replyTo ? (
          <p className={styles.messageReply}>
            Replying to {authorLabel(replyTo.author_id)}: {replyTo.deleted_at ? "deleted message" : replyTo.content}
          </p>
        ) : message.reply_to_message_id ? (
          <p className={styles.messageReply}>Replying to {shortIdentity(message.reply_to_message_id)}</p>
        ) : null}
        <p className={deleted ? styles.messageDeleted : styles.messageContent}>
          {deleted ? "Message deleted" : message.content}
        </p>
        {message.mentions.length > 0 ? (
          <div className={styles.messageMentions} aria-label="Mentions">
            {message.mentions.map((mention) => (
              <span className={styles.mentionToken} key={mention}>
                @{authorHandle(mention)}
              </span>
            ))}
          </div>
        ) : null}
      </div>
      {!deleted ? (
        <button
          aria-label={`Reply to message from ${authorLabel(message.author_id)}`}
          className={styles.messageAction}
          onClick={() => onReply(message)}
          title="Reply"
          type="button"
        >
          <IconMessageCircle className={styles.icon} aria-hidden="true" />
        </button>
      ) : null}
    </article>
  );
}

export default function ServerWorkspacePage() {
  const params = useParams<{ serverId: string }>();
  const serverId = decodePathParam(params.serverId);
  const browserReady = useSyncExternalStore(
    subscribeBrowserReady,
    getBrowserReadySnapshot,
    getBrowserReadyServerSnapshot,
  ) === "client";
  const personas = useMemo(() => (browserReady ? readPersonas() : []), [browserReady]);
  const identityId = useMemo(
    () => (browserReady ? readActivePersonaId() ?? personas[0]?.id ?? "usr-test-alice" : "usr-test-alice"),
    [browserReady, personas],
  );
  const hasSession = useMemo(() => browserReady && getPersonaSession(identityId) !== null, [browserReady, identityId]);
  const [server, setServer] = useState<ServerSummary | null>(null);
  const [channels, setChannels] = useState<ServerChannelSummary[]>([]);
  const [activeChannelId, setActiveChannelId] = useState<string | null>(PREVIEW_CHANNELS[0]?.id ?? null);
  const [messages, setMessages] = useState<ServerChannelMessage[]>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [view, setView] = useState<ServerView>("overview");
  const [workspaceState, setWorkspaceState] = useState<LoadState>("idle");
  const [messageState, setMessageState] = useState<LoadState>("idle");
  const [olderState, setOlderState] = useState<LoadState>("idle");
  const [composer, setComposer] = useState("");
  const [replyTo, setReplyTo] = useState<ServerChannelMessage | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [sendBusy, setSendBusy] = useState(false);

  useEffect(() => {
    let active = true;

    setStatusMessage(null);
    setServer(null);
    setChannels([]);
    setMessages([]);
    setNextCursor(null);

    if (!hasSession || !serverId) {
      setWorkspaceState("idle");
      setMessageState("idle");
      setActiveChannelId((current) => current ?? PREVIEW_CHANNELS[0]?.id ?? null);
      return () => {
        active = false;
      };
    }

    setWorkspaceState("loading");

    const run = async (): Promise<void> => {
      try {
        const [serverResult, channelsResult] = await Promise.all([
          fetchServer({ serverId }),
          fetchServerChannels({ serverId }),
        ]);

        if (!active) {
          return;
        }

        if (!serverResult.ok) {
          setWorkspaceState("error");
          setStatusMessage(formatApiError(serverResult.code, serverResult.message));
          return;
        }

        if (!channelsResult.ok) {
          setWorkspaceState("error");
          setStatusMessage(formatApiError(channelsResult.code, channelsResult.message));
          return;
        }

        setServer(serverResult.data.item);
        setChannels(channelsResult.data.items);
        setActiveChannelId(channelsResult.data.items[0]?.id ?? null);
        setWorkspaceState("ready");
      } catch {
        if (!active) {
          return;
        }

        setWorkspaceState("error");
        setStatusMessage("network_error: Could not load this server.");
      }
    };

    void run();

    return () => {
      active = false;
    };
  }, [hasSession, serverId]);

  useEffect(() => {
    let active = true;

    setMessages([]);
    setNextCursor(null);
    setReplyTo(null);

    if (!hasSession || !serverId || !activeChannelId) {
      setMessageState("idle");
      return () => {
        active = false;
      };
    }

    setMessageState("loading");

    const run = async (): Promise<void> => {
      try {
        const result = await fetchServerChannelMessages({
          serverId,
          channelId: activeChannelId,
          limit: MESSAGE_PAGE_LIMIT,
        });

        if (!active) {
          return;
        }

        if (!result.ok) {
          setMessageState("error");
          setStatusMessage(formatApiError(result.code, result.message));
          return;
        }

        setMessages(mergeMessages([], result.data.items));
        setNextCursor(result.data.next_cursor ?? null);
        setMessageState("ready");
      } catch {
        if (!active) {
          return;
        }

        setMessageState("error");
        setStatusMessage("network_error: Could not load channel history.");
      }
    };

    void run();

    return () => {
      active = false;
    };
  }, [activeChannelId, hasSession, serverId]);

  const visibleServer = server ?? PREVIEW_SERVER;
  const visibleChannels = channels.length > 0 ? channels : PREVIEW_CHANNELS;
  const activeChannel =
    visibleChannels.find((channel) => channel.id === activeChannelId) ?? visibleChannels[0] ?? null;
  const visibleMessages = hasSession
    ? visibleMessagesForChannel(messages, activeChannel?.id ?? null)
    : visibleMessagesForChannel(PREVIEW_MESSAGES, activeChannel?.id ?? null);
  const allVisibleMessages = hasSession && messages.length > 0 ? messages : PREVIEW_MESSAGES;
  const latest = latestMessage(allVisibleMessages);
  const messageById = useMemo(() => {
    const lookup = new Map<string, ServerChannelMessage>();
    for (const message of allVisibleMessages) {
      lookup.set(message.message_id, message);
    }
    return lookup;
  }, [allVisibleMessages]);
  const mentionIdentityIds = useMemo(() => extractMentionIdentityIds(composer), [composer]);

  function selectChannel(channelId: string): void {
    setActiveChannelId(channelId);
    setView("chat");
  }

  async function reloadActiveChannel(): Promise<void> {
    if (!hasSession) {
      setStatusMessage("preview: Activate a local testing profile to refresh live server data.");
      return;
    }
    if (!serverId || !activeChannel) {
      return;
    }

    setMessageState("loading");
    setStatusMessage(null);

    try {
      const result = await fetchServerChannelMessages({
        serverId,
        channelId: activeChannel.id,
        limit: MESSAGE_PAGE_LIMIT,
      });

      if (!result.ok) {
        setMessageState("error");
        setStatusMessage(formatApiError(result.code, result.message));
        return;
      }

      setMessages(mergeMessages([], result.data.items));
      setNextCursor(result.data.next_cursor ?? null);
      setMessageState("ready");
    } catch {
      setMessageState("error");
      setStatusMessage("network_error: Could not reload channel history.");
    }
  }

  async function loadOlderMessages(): Promise<void> {
    if (!serverId || !activeChannel || !nextCursor || olderState === "loading") {
      return;
    }

    setOlderState("loading");
    setStatusMessage(null);

    try {
      const result = await fetchServerChannelMessages({
        serverId,
        channelId: activeChannel.id,
        cursor: nextCursor,
        limit: MESSAGE_PAGE_LIMIT,
      });

      if (!result.ok) {
        setOlderState("error");
        setStatusMessage(formatApiError(result.code, result.message));
        return;
      }

      setMessages((current) => mergeMessages(current, result.data.items));
      setNextCursor(result.data.next_cursor ?? null);
      setOlderState("ready");
    } catch {
      setOlderState("error");
      setStatusMessage("network_error: Could not load older messages.");
    }
  }

  async function sendMessage(): Promise<void> {
    const content = composer.trim();
    if (!hasSession) {
      setStatusMessage("preview: Activate a local testing profile to send messages.");
      return;
    }
    if (!serverId || !activeChannel) {
      return;
    }
    if (!content) {
      setStatusMessage("message_content_invalid: Write a message before sending.");
      return;
    }

    setSendBusy(true);
    setStatusMessage(null);

    try {
      const result = await createServerChannelMessage({
        serverId,
        channelId: activeChannel.id,
        content,
        replyToMessageId: replyTo?.message_id ?? null,
        mentionIdentityIds,
      });

      if (!result.ok) {
        setStatusMessage(formatApiError(result.code, result.message));
        setSendBusy(false);
        return;
      }

      setMessages((current) => mergeMessages(current, [result.data]));
      setChannels((current) =>
        current.map((channel) =>
          channel.id === activeChannel.id
            ? { ...channel, last_message_seq: Math.max(channel.last_message_seq, result.data.channel_seq) }
            : channel,
        ),
      );
      setComposer("");
      setReplyTo(null);
      setStatusMessage(`sent: Message posted to #${activeChannel.name}.`);
    } catch {
      setStatusMessage("network_error: Message could not be sent.");
    } finally {
      setSendBusy(false);
    }
  }

  return (
    <WorkspaceShell
      activeTabId="server"
      subtitle={`${visibleServer.name} server view`}
      tabs={[{ id: "server", label: visibleServer.name, icon: IconServer2 }]}
      title={visibleServer.name}
    >
      <section className={styles.serverPage}>
        <header className={styles.serverHero}>
          <ServerIcon name={visibleServer.name} />
          <div className={styles.serverHeroText}>
            <p className={styles.serverSectionLabel}>Server</p>
            <h2 className={styles.serverTitle}>{visibleServer.name}</h2>
            <p className={styles.serverMeta}>
              Shared seeded validation space for server channels, replies, mentions, and unread state.
            </p>
            <div className={styles.serverStatusRow} aria-label="Server status">
              <span className={styles.statusBadge}>
                <IconStar className={styles.icon} aria-hidden="true" />
                {visibleServer.favorite ? "Favorite" : "Standard"}
              </span>
              <span className={styles.statusBadge}>
                {visibleServer.muted ? (
                  <IconBellOff className={styles.icon} aria-hidden="true" />
                ) : (
                  <IconBell className={styles.icon} aria-hidden="true" />
                )}
                {visibleServer.muted ? "Muted" : "Audible"}
              </span>
              <span className={styles.statusBadge}>{visibleServer.unread} unread</span>
            </div>
          </div>
          <Link className={styles.backButton} href="/servers">
            <IconArrowLeft className={styles.icon} aria-hidden="true" />
            Servers
          </Link>
        </header>

        <div className={styles.serverViewBar} role="group" aria-label="Server views">
          <button
            aria-pressed={view === "overview"}
            className={`${styles.viewButton} ${view === "overview" ? styles.viewButtonActive : ""}`}
            onClick={() => setView("overview")}
            type="button"
          >
            Overview
          </button>
          <button
            aria-pressed={view === "chat"}
            className={`${styles.viewButton} ${view === "chat" ? styles.viewButtonActive : ""}`}
            onClick={() => setView("chat")}
            type="button"
          >
            Chat
          </button>
        </div>

        {!hasSession ? (
          <div className={styles.serverNotice}>
            <IconInfoCircle className={styles.icon} aria-hidden="true" />
            <span>Activate a local testing profile to load live server data. Showing seeded Atlas preview data.</span>
          </div>
        ) : null}

        {workspaceState === "loading" ? <p className={styles.state}>Loading server...</p> : null}
        {workspaceState === "error" ? <p className={styles.state}>Could not load this server.</p> : null}
        {statusMessage ? (
          <p className={styles.statusLine}>
            <IconInfoCircle className={styles.icon} aria-hidden="true" />
            {statusMessage}
          </p>
        ) : null}

        {view === "overview" ? (
          <section className={styles.overviewGrid} aria-label="Server overview">
            <article className={styles.overviewPanel}>
              <div className={styles.panelHeader}>
                <h3>Channels</h3>
                <span>{visibleChannels.length}</span>
              </div>
              <div className={styles.channelStack}>
                {visibleChannels.map((channel) => (
                  <ChannelButton
                    active={channel.id === activeChannel?.id}
                    channel={channel}
                    key={channel.id}
                    onSelect={selectChannel}
                  />
                ))}
              </div>
            </article>

            <article className={styles.overviewPanel}>
              <div className={styles.panelHeader}>
                <h3>Latest activity</h3>
                {latest ? <span>{formatTimestamp(latest.created_at)}</span> : null}
              </div>
              {latest ? (
                <div className={styles.activityPreview}>
                  <div className={styles.messageAvatar}>{initials(authorLabel(latest.author_id))}</div>
                  <div>
                    <p className={styles.messageAuthor}>{authorLabel(latest.author_id)}</p>
                    <p className={styles.messageContent}>{latest.content}</p>
                  </div>
                </div>
              ) : (
                <p className={styles.meta}>No channel activity yet.</p>
              )}
            </article>

            <article className={styles.overviewPanel}>
              <div className={styles.panelHeader}>
                <h3>Validation focus</h3>
              </div>
              <p className={styles.meta}>
                The server-chat fixture exercises two channels, three member identities, mentions, replies,
                unread count, favorite state, and muted state.
              </p>
            </article>
          </section>
        ) : (
          <section className={styles.chatGrid} aria-label="Server chat">
            <aside className={styles.chatChannelRail} aria-label="Channels">
              <div className={styles.panelHeader}>
                <h3>Channels</h3>
              </div>
              <div className={styles.channelStack}>
                {visibleChannels.map((channel) => (
                  <ChannelButton
                    active={channel.id === activeChannel?.id}
                    channel={channel}
                    key={channel.id}
                    onSelect={selectChannel}
                  />
                ))}
              </div>
            </aside>

            <article className={styles.chatPanel}>
              <header className={styles.chatHeader}>
                <div>
                  <p className={styles.serverSectionLabel}>Channel</p>
                  <h3>
                    <IconHash className={styles.icon} aria-hidden="true" />
                    {activeChannel?.name ?? "No channel"}
                  </h3>
                  <p className={styles.serverMeta}>
                    {activeChannel ? `${activeChannel.kind} channel - latest seq ${activeChannel.last_message_seq}` : ""}
                  </p>
                </div>
                <button
                  className={styles.backButton}
                  disabled={!hasSession || !activeChannel || messageState === "loading"}
                  onClick={() => void reloadActiveChannel()}
                  type="button"
                >
                  <IconRefresh className={styles.icon} aria-hidden="true" />
                  Refresh
                </button>
              </header>

              {messageState === "loading" ? <p className={styles.state}>Loading channel history...</p> : null}
              {messageState === "error" ? <p className={styles.state}>Could not load channel history.</p> : null}

              <div className={styles.messageTimeline}>
                {nextCursor && hasSession ? (
                  <button
                    className={styles.loadOlderButton}
                    disabled={olderState === "loading"}
                    onClick={() => void loadOlderMessages()}
                    type="button"
                  >
                    <IconMessageCircle className={styles.icon} aria-hidden="true" />
                    {olderState === "loading" ? "Loading older..." : "Load older messages"}
                  </button>
                ) : null}
                {visibleMessages.length > 0 ? (
                  visibleMessages.map((message) => (
                    <MessageBubble
                      key={message.message_id}
                      message={message}
                      onReply={setReplyTo}
                      ownMessage={message.author_id === identityId}
                      replyTo={message.reply_to_message_id ? messageById.get(message.reply_to_message_id) : undefined}
                    />
                  ))
                ) : (
                  <p className={styles.state}>No messages in this channel yet.</p>
                )}
              </div>

              {hasSession ? (
                <section className={styles.composerPanel} aria-label="Message composer">
                  {replyTo ? (
                    <div className={styles.replyDraft}>
                      <div>
                        <p className={styles.serverSectionLabel}>Replying to {authorLabel(replyTo.author_id)}</p>
                        <p className={styles.meta}>{replyTo.content}</p>
                      </div>
                      <button className={styles.backButton} onClick={() => setReplyTo(null)} type="button">
                        Cancel reply
                      </button>
                    </div>
                  ) : null}

                  <textarea
                    className={styles.composerInput}
                    disabled={!activeChannel || sendBusy}
                    onChange={(event) => setComposer(event.target.value)}
                    placeholder={activeChannel ? `Message #${activeChannel.name}` : "Select a channel"}
                    rows={3}
                    value={composer}
                  />
                  <div className={styles.composerBar}>
                    <div className={styles.composerHints}>
                      {mentionIdentityIds.length > 0 ? (
                        mentionIdentityIds.map((mention) => (
                          <span className={styles.mentionToken} key={mention}>
                            @{authorHandle(mention)}
                          </span>
                        ))
                      ) : (
                        <span className={styles.meta}>Use @alice, @bob, or @carol with the seeded fixture.</span>
                      )}
                    </div>
                    <button
                      className={`${styles.backButton} ${styles.sendButton}`}
                      disabled={!activeChannel || sendBusy}
                      onClick={() => void sendMessage()}
                      type="button"
                    >
                      <IconSend className={styles.icon} aria-hidden="true" />
                      {sendBusy ? "Sending..." : "Send"}
                    </button>
                  </div>
                </section>
              ) : (
                <div className={styles.composerLocked}>
                  <IconInfoCircle className={styles.icon} aria-hidden="true" />
                  Activate a local testing profile to send messages.
                </div>
              )}
            </article>
          </section>
        )}
      </section>
    </WorkspaceShell>
  );
}
