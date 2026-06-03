"use client";

import { useParams } from "next/navigation";
import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { IconInfoCircle, IconMessageCircle, IconSettings, IconUsers, IconVolume } from "@tabler/icons-react";

import { ChatView } from "@/components/server-workspace/chat-view";
import { Overview } from "@/components/server-workspace/overview";
import { SettingsView } from "@/components/server-workspace/settings-view";
import { UsersView } from "@/components/server-workspace/users-view";
import { VoiceView } from "@/components/server-workspace/voice-view";
import type {
  LoadState,
  Member,
  RoleGroup,
  VoiceChannel,
} from "@/components/server-workspace/types";
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
import {
  readMessageAlignment,
  readMessageBubbleSize,
  readMessageLayout,
  subscribeWorkspacePreferences,
  type MessageAlignment,
  type MessageBubbleSize,
  type MessageLayout,
} from "@/lib/workspace-preferences";

import styles from "../../surfaces.module.css";

const MESSAGE_PAGE_LIMIT = 30;

const SEEDED_IDENTITIES: Record<string, { name: string; handle: string }> = {
  "usr-test-alice": { name: "Alice", handle: "alice" },
  "usr-test-bob": { name: "Bob", handle: "bob" },
  "usr-test-carol": { name: "Carol", handle: "carol" },
};

const PREVIEW_SERVER: ServerSummary = {
  id: "hexrelay-local-server",
  name: "Atlas Test Server",
  unread: 2,
  pinned: true,
  muted: false,
};

const PREVIEW_SERVER_TAGS = ["Validation", "Community", "Voice QA"];

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

const PREVIEW_MEMBERS: Member[] = [
  {
    identityId: "usr-test-alice",
    role: "Admins",
    title: "Server owner",
    presence: "online",
    pinned: true,
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
    pinned: false,
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
    pinned: false,
    muted: true,
    unread: 0,
    joinedAt: "2026-05-04T11:03:00Z",
    lastActive: "Confirmed reply flow",
  },
];

const PREVIEW_VOICE_CHANNELS: VoiceChannel[] = [
  {
    id: "fixture-voice-atlas-lobby",
    name: "Lobby",
    description: "Open drop-in room for validation calls.",
    participantIds: ["usr-test-alice", "usr-test-bob"],
    speakerId: "usr-test-alice",
  },
  {
    id: "fixture-voice-atlas-ops-room",
    name: "Ops room",
    description: "Focused room for workspace and moderation checks.",
    participantIds: [],
    speakerId: null,
  },
];

type ServerView = "overview" | "users" | "chat" | "voice" | "settings";
type ChannelMessageCache = Record<string, ServerChannelMessage[]>;

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

function readCachedChannelMessages(cache: ChannelMessageCache, channelId: string | null): ServerChannelMessage[] {
  return channelId ? (cache[channelId] ?? []) : [];
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
  const messageLayout = useSyncExternalStore<MessageLayout>(
    subscribeWorkspacePreferences,
    readMessageLayout,
    () => "bubble-cards",
  );
  const messageBubbleSize = useSyncExternalStore<MessageBubbleSize>(
    subscribeWorkspacePreferences,
    readMessageBubbleSize,
    () => "comfortable",
  );
  const messageAlignment = useSyncExternalStore<MessageAlignment>(
    subscribeWorkspacePreferences,
    readMessageAlignment,
    () => "conversation-sides",
  );
  const [server, setServer] = useState<ServerSummary | null>(null);
  const [channels, setChannels] = useState<ServerChannelSummary[]>([]);
  const [activeChannelId, setActiveChannelId] = useState<string | null>(null);
  const [messagesByChannel, setMessagesByChannel] = useState<ChannelMessageCache>({});
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [activeVoiceChannelId, setActiveVoiceChannelId] = useState<string | null>(
    PREVIEW_VOICE_CHANNELS[0]?.id ?? null,
  );
  const [view, setView] = useState<ServerView>("overview");
  const [workspaceState, setWorkspaceState] = useState<LoadState>("idle");
  const [messageState, setMessageState] = useState<LoadState>("idle");
  const [olderState, setOlderState] = useState<LoadState>("idle");
  const [composer, setComposer] = useState("");
  const [replyTo, setReplyTo] = useState<ServerChannelMessage | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [sendBusy, setSendBusy] = useState(false);
  const [serverMenuOpen, setServerMenuOpen] = useState(false);

  useEffect(() => {
    let active = true;

    setStatusMessage(null);
    setServer(null);
    setChannels([]);
    setMessagesByChannel({});
    setNextCursor(null);

    if (!hasSession || !serverId) {
      setWorkspaceState("idle");
      setMessageState("idle");
      setActiveChannelId((current) => current ?? PREVIEW_CHANNELS[0]?.id ?? null);
      return () => {
        active = false;
      };
    }

    setActiveChannelId(null);
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

    setMessagesByChannel((current) => {
      if (!activeChannelId || !current[activeChannelId]) {
        return current;
      }

      const next = { ...current };
      delete next[activeChannelId];
      return next;
    });
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

        setMessagesByChannel((current) => ({
          ...current,
          [activeChannelId]: mergeMessages([], result.data.items),
        }));
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
  const visibleChannels = hasSession ? channels : PREVIEW_CHANNELS;
  const activeChannel =
    visibleChannels.find((channel) => channel.id === activeChannelId) ?? visibleChannels[0] ?? null;
  const activeVoiceChannel =
    PREVIEW_VOICE_CHANNELS.find((channel) => channel.id === activeVoiceChannelId) ?? PREVIEW_VOICE_CHANNELS[0] ?? null;
  const cachedMessages = useMemo(() => Object.values(messagesByChannel).flat(), [messagesByChannel]);
  const visibleMessages = hasSession
    ? readCachedChannelMessages(messagesByChannel, activeChannel?.id ?? null)
    : visibleMessagesForChannel(PREVIEW_MESSAGES, activeChannel?.id ?? null);
  const allVisibleMessages = hasSession ? cachedMessages : PREVIEW_MESSAGES;
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
  const roleGroups = useMemo<RoleGroup[]>(
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
  const authorFormatters = useMemo(
    () => ({
      authorHandle,
      authorLabel,
      formatTimestamp,
      shortIdentity,
    }),
    [],
  );
  const chatPrefs = useMemo(
    () => ({
      alignment: messageAlignment,
      bubbleSize: messageBubbleSize,
      layout: messageLayout,
    }),
    [messageAlignment, messageBubbleSize, messageLayout],
  );
  const chatData = useMemo(
    () => ({
      activeChannel,
      channelNotificationCounts,
      channels: visibleChannels,
      messageById,
      messages: visibleMessages,
      nextCursor,
    }),
    [activeChannel, channelNotificationCounts, messageById, nextCursor, visibleChannels, visibleMessages],
  );
  const chatState = useMemo(
    () => ({
      composer,
      hasSession,
      identityId,
      mentionIdentityIds,
      messageState,
      olderState,
      replyTo,
      sendBusy,
    }),
    [composer, hasSession, identityId, mentionIdentityIds, messageState, olderState, replyTo, sendBusy],
  );

  useEffect(() => {
    if (!canManageServer && view === "settings") {
      setView("overview");
    }
  }, [canManageServer, view]);

  function selectChannel(channelId: string): void {
    setActiveChannelId(channelId);
    setView("chat");
  }

  function selectVoiceChannel(channelId: string): void {
    setActiveVoiceChannelId(channelId);
  }

  function handleServerMenuAction(label: string): void {
    setServerMenuOpen(false);
    setStatusMessage(`preview: ${label} is available as a menu action, but is not wired in this validation build.`);
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

      setMessagesByChannel((current) => ({
        ...current,
        [activeChannel.id]: mergeMessages([], result.data.items),
      }));
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

      setMessagesByChannel((current) => ({
        ...current,
        [activeChannel.id]: mergeMessages(current[activeChannel.id] ?? [], result.data.items),
      }));
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

      setMessagesByChannel((current) => ({
        ...current,
        [activeChannel.id]: mergeMessages(current[activeChannel.id] ?? [], [result.data]),
      }));
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
      workspaceTab={{
        imageLabel: visibleServer.name,
        label: visibleServer.name,
        unread: visibleServer.unread,
      }}
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
          <Overview
            hasSession={hasSession}
            menuOpen={serverMenuOpen}
            onMenuAction={handleServerMenuAction}
            onToggleMenu={() => setServerMenuOpen((current) => !current)}
            rules={PREVIEW_RULES}
            server={visibleServer}
            tags={PREVIEW_SERVER_TAGS}
          />
        ) : view === "users" ? (
          <UsersView
            authorHandle={authorHandle}
            authorLabel={authorLabel}
            formatTimestamp={formatTimestamp}
            hasSession={hasSession}
            identityId={identityId}
            roleGroups={roleGroups}
          />
        ) : view === "chat" ? (
          <ChatView
            data={chatData}
            formatters={authorFormatters}
            onCancelReply={() => setReplyTo(null)}
            onChangeComposer={setComposer}
            onLoadOlder={() => void loadOlderMessages()}
            onRefresh={() => void reloadActiveChannel()}
            onReply={setReplyTo}
            onSelectChannel={selectChannel}
            onSend={() => void sendMessage()}
            prefs={chatPrefs}
            state={chatState}
          />
        ) : view === "voice" ? (
          <VoiceView
            activeChannel={activeVoiceChannel}
            authorHandle={authorHandle}
            authorLabel={authorLabel}
            channels={PREVIEW_VOICE_CHANNELS}
            onSelectChannel={selectVoiceChannel}
          />
        ) : (
          <SettingsView
            canManageServer={canManageServer}
            channels={visibleChannels}
            hasSession={hasSession}
            memberCount={PREVIEW_MEMBERS.length}
            server={visibleServer}
            voiceChannelCount={PREVIEW_VOICE_CHANNELS.length}
          />
        )}
      </section>
    </WorkspaceShell>
  );
}
