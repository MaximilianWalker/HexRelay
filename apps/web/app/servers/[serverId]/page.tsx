"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import {
  IconArrowLeft,
  IconBell,
  IconBellOff,
  IconCircleCheck,
  IconClock,
  IconHash,
  IconInfoCircle,
  IconMessageCircle,
  IconRefresh,
  IconSend,
  IconSettings,
  IconShieldCheck,
  IconStar,
  IconUsers,
  IconVolume,
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

const PREVIEW_RULES = [
  "Keep validation notes in the right channel.",
  "Use mentions when a specific seeded user needs to verify a flow.",
  "Move voice-session issues into the voice tab once that surface is ready.",
];

const PREVIEW_ROLE_SUMMARY = [
  { label: "Admins", names: "Alice" },
  { label: "Maintainers", names: "Bob" },
  { label: "Members", names: "Carol" },
];

const PREVIEW_ROLE_DESCRIPTIONS: Record<string, string> = {
  Admins: "Full server management and settings access.",
  Maintainers: "Coordinate channel, workspace, and validation activity.",
  Members: "Participate in chat, voice, and fixture validation.",
};

const PREVIEW_MEMBERS = [
  {
    identityId: "usr-test-alice",
    role: "Admins",
    title: "Server owner",
    presence: "online",
    favorite: true,
    muted: false,
    unread: 2,
    joinedAt: "2026-05-04T11:01:00Z",
    lastActive: "Reviewing #ops-lab",
  },
  {
    identityId: "usr-test-bob",
    role: "Maintainers",
    title: "Fixture maintainer",
    presence: "online",
    favorite: false,
    muted: false,
    unread: 1,
    joinedAt: "2026-05-04T11:02:00Z",
    lastActive: "Tracking setup notes",
  },
  {
    identityId: "usr-test-carol",
    role: "Members",
    title: "Validation member",
    presence: "away",
    favorite: false,
    muted: true,
    unread: 0,
    joinedAt: "2026-05-04T11:03:00Z",
    lastActive: "Confirmed reply flow",
  },
];

type PreviewMember = (typeof PREVIEW_MEMBERS)[number];
type LoadState = "idle" | "loading" | "ready" | "error";
type ServerView = "overview" | "users" | "chat" | "voice" | "settings";

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
  notificationCount,
  onSelect,
}: {
  channel: ServerChannelSummary;
  active: boolean;
  notificationCount: number;
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
      {notificationCount > 0 ? (
        <span className={styles.channelBadge} aria-label={`${notificationCount} unseen mentions`}>
          {notificationCount}
        </span>
      ) : null}
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

function MemberCard({ member, current }: { member: PreviewMember; current: boolean }) {
  const name = authorLabel(member.identityId);
  const presenceLabel = member.presence === "online" ? "Online" : "Away";

  return (
    <article className={`${styles.memberCard} ${current ? styles.memberCardCurrent : ""}`}>
      <div className={styles.memberAvatarWrap}>
        <div className={styles.memberAvatar}>{initials(name)}</div>
        <span
          aria-label={presenceLabel}
          className={`${styles.presenceDot} ${member.presence === "online" ? styles.presenceOnline : styles.presenceAway}`}
          role="img"
        />
      </div>
      <div className={styles.memberInfo}>
        <div className={styles.memberNameRow}>
          <h4>{name}</h4>
          {current ? <span className={styles.memberBadge}>You</span> : null}
        </div>
        <p>@{authorHandle(member.identityId)}</p>
        <p>{member.title}</p>
        <span>{member.lastActive}</span>
      </div>
      <div className={styles.memberMetaStack}>
        <span>
          <IconClock className={styles.icon} aria-hidden="true" />
          Joined {formatTimestamp(member.joinedAt)}
        </span>
        <span>
          {member.muted ? (
            <IconBellOff className={styles.icon} aria-hidden="true" />
          ) : (
            <IconBell className={styles.icon} aria-hidden="true" />
          )}
          {member.muted ? "Muted" : "Audible"}
        </span>
        {member.favorite ? (
          <span>
            <IconStar className={styles.icon} aria-hidden="true" />
            Favorite
          </span>
        ) : null}
        {member.unread > 0 ? <strong>{member.unread}</strong> : null}
      </div>
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
  const channelNotificationCounts = useMemo(() => {
    const counts = new Map<string, number>();
    for (const message of allVisibleMessages) {
      if (message.mentions.includes(identityId)) {
        counts.set(message.channel_id, (counts.get(message.channel_id) ?? 0) + 1);
      }
    }
    return counts;
  }, [allVisibleMessages, identityId]);
  const messageById = useMemo(() => {
    const lookup = new Map<string, ServerChannelMessage>();
    for (const message of allVisibleMessages) {
      lookup.set(message.message_id, message);
    }
    return lookup;
  }, [allVisibleMessages]);
  const mentionIdentityIds = useMemo(() => extractMentionIdentityIds(composer), [composer]);
  const canManageServer = !hasSession || identityId === "usr-test-alice";
  const roleGroups = useMemo(
    () =>
      PREVIEW_ROLE_SUMMARY.map((role) => ({
        ...role,
        description: PREVIEW_ROLE_DESCRIPTIONS[role.label] ?? "Server role",
        members: PREVIEW_MEMBERS.filter((member) => member.role === role.label),
      })),
    [],
  );
  const serverTabs = useMemo(
    () =>
      [
        { id: "overview" as const, label: "Overview", icon: IconInfoCircle },
        { id: "users" as const, label: "Users", icon: IconUsers },
        { id: "chat" as const, label: "Chat", icon: IconMessageCircle },
        { id: "voice" as const, label: "Voice", icon: IconVolume },
        ...(canManageServer ? [{ id: "settings" as const, label: "Settings", icon: IconSettings }] : []),
      ].map((tab) => ({
        ...tab,
        onSelect: () => setView(tab.id),
      })),
    [canManageServer],
  );

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
      activeTabId={view}
      subtitle={`${visibleServer.name} server view`}
      tabs={serverTabs}
      title={visibleServer.name}
    >
      <section className={styles.serverPage}>
        {workspaceState === "loading" ? <p className={styles.state}>Loading server...</p> : null}
        {workspaceState === "error" ? <p className={styles.state}>Could not load this server.</p> : null}
        {statusMessage ? (
          <p className={styles.statusLine}>
            <IconInfoCircle className={styles.icon} aria-hidden="true" />
            {statusMessage}
          </p>
        ) : null}

        {view === "overview" ? (
          <section className={styles.overviewStack} aria-label="Server overview">
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

            {!hasSession ? (
              <div className={styles.serverNotice}>
                <IconInfoCircle className={styles.icon} aria-hidden="true" />
                <span>Activate a local testing profile to load live server data. Showing seeded Atlas preview data.</span>
              </div>
            ) : null}

            <div className={styles.overviewFeatureGrid}>
              <article className={`${styles.overviewPanel} ${styles.markdownPanel}`}>
                <div className={styles.panelHeader}>
                  <h3>About this server</h3>
                  <span>server.md</span>
                </div>
                <div className={styles.markdownPreview}>
                  <h4>Atlas validation space</h4>
                  <p>
                    Atlas is the seeded server for reviewing shared channels, mentions, replies, roles, and voice
                    workspace behavior before the live server surface is widened.
                  </p>
                  <ul>
                    <li>
                      <strong>#general</strong> keeps the default conversation and mention checks.
                    </li>
                    <li>
                      <strong>#ops-lab</strong> tracks workspace, moderation, and voice follow-up notes.
                    </li>
                  </ul>
                  <blockquote>Today: validate the server workspace layout and tab model.</blockquote>
                </div>
              </article>

              <div className={styles.overviewRail}>
                <article className={styles.overviewPanel}>
                  <div className={styles.panelHeader}>
                    <h3>Pinned announcement</h3>
                  </div>
                  <p className={styles.meta}>
                    Review the server tabs first, then use #ops-lab for notes about voice, settings, and role grouping.
                  </p>
                </article>

                <article className={styles.overviewPanel}>
                  <div className={styles.panelHeader}>
                    <h3>Roles</h3>
                    <span>{PREVIEW_ROLE_SUMMARY.length}</span>
                  </div>
                  <div className={styles.roleSummaryList}>
                    {PREVIEW_ROLE_SUMMARY.map((role) => (
                      <div className={styles.roleSummaryItem} key={role.label}>
                        <span>{role.label}</span>
                        <strong>{role.names}</strong>
                      </div>
                    ))}
                  </div>
                </article>
              </div>
            </div>

            <section className={styles.overviewGrid} aria-label="Server summary">
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
                      notificationCount={channelNotificationCounts.get(channel.id) ?? 0}
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
                  <h3>Rules</h3>
                  <span>{PREVIEW_RULES.length}</span>
                </div>
                <ul className={styles.overviewList}>
                  {PREVIEW_RULES.map((rule) => (
                    <li key={rule}>{rule}</li>
                  ))}
                </ul>
              </article>
            </section>
          </section>
        ) : view === "users" ? (
          <section className={styles.usersView} aria-label="Server users">
            <header className={styles.usersHeader}>
              <div>
                <p className={styles.serverSectionLabel}>Members</p>
                <h2>Server users</h2>
                <p className={styles.serverMeta}>
                  Seeded server-chat memberships grouped by role, with profile, presence, and per-member server state.
                </p>
              </div>
              <div className={styles.usersStats} aria-label="Member summary">
                <span>
                  <IconUsers className={styles.icon} aria-hidden="true" />
                  {PREVIEW_MEMBERS.length} members
                </span>
                <span>
                  <IconShieldCheck className={styles.icon} aria-hidden="true" />
                  {roleGroups.length} roles
                </span>
                <span>
                  <IconCircleCheck className={styles.icon} aria-hidden="true" />
                  {PREVIEW_MEMBERS.filter((member) => member.presence === "online").length} online
                </span>
              </div>
            </header>

            {!hasSession ? (
              <div className={styles.serverNotice}>
                <IconInfoCircle className={styles.icon} aria-hidden="true" />
                <span>Showing seeded Atlas membership data until a local testing profile loads live server state.</span>
              </div>
            ) : null}

            <div className={styles.roleGroups}>
              {roleGroups.map((group) => (
                <section className={styles.roleGroup} key={group.label} aria-label={`${group.label} members`}>
                  <div className={styles.roleGroupHeader}>
                    <div>
                      <h3>{group.label}</h3>
                      <p>{group.description}</p>
                    </div>
                    <span>{group.members.length}</span>
                  </div>
                  <div className={styles.memberList}>
                    {group.members.map((member) => (
                      <MemberCard
                        current={member.identityId === identityId}
                        key={member.identityId}
                        member={member}
                      />
                    ))}
                  </div>
                </section>
              ))}
            </div>
          </section>
        ) : view === "chat" ? (
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
                    notificationCount={channelNotificationCounts.get(channel.id) ?? 0}
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
                    {activeChannel ? `${activeChannel.kind} channel` : ""}
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
        ) : view === "voice" ? (
          <section className={styles.chatGrid} aria-label="Server voice">
            <aside className={styles.chatChannelRail} aria-label="Voice channels">
              <div className={styles.panelHeader}>
                <h3>Voice channels</h3>
              </div>
              <div className={styles.channelStack}>
                <button className={styles.channelButton} type="button">
                  <IconVolume className={styles.icon} aria-hidden="true" />
                  <span>Lobby</span>
                </button>
                <button className={styles.channelButton} type="button">
                  <IconVolume className={styles.icon} aria-hidden="true" />
                  <span>Ops room</span>
                </button>
              </div>
            </aside>

            <article className={styles.chatPanel}>
              <header className={styles.chatHeader}>
                <div>
                  <p className={styles.serverSectionLabel}>Voice</p>
                  <h3>
                    <IconVolume className={styles.icon} aria-hidden="true" />
                    Lobby
                  </h3>
                  <p className={styles.serverMeta}>No active speakers in the seeded preview.</p>
                </div>
              </header>
            </article>
          </section>
        ) : (
          <section className={styles.overviewGrid} aria-label="Server settings">
            <article className={styles.overviewPanel}>
              <div className={styles.panelHeader}>
                <h3>Server settings</h3>
              </div>
              <p className={styles.meta}>Atlas Test Server</p>
            </article>
          </section>
        )}
      </section>
    </WorkspaceShell>
  );
}
