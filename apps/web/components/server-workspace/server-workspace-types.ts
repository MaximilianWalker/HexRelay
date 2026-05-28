import type { ServerChannelMessage, ServerChannelSummary } from "@/lib/api";
import type { MessageAlignment, MessageBubbleSize, MessageLayout } from "@/lib/workspace-preferences";

export type ServerMember = {
  identityId: string;
  joinedAt: string;
  lastActive: string;
  muted: boolean;
  pinned: boolean;
  presence: "online" | "away";
  role: string;
  title: string;
  unread: number;
};

export type ServerRoleGroup = {
  description: string;
  label: string;
  members: ServerMember[];
};

export type ServerVoiceChannel = {
  description: string;
  id: string;
  name: string;
  participantIds: string[];
  speakerId: string | null;
};

export type ServerWorkspaceLoadState = "idle" | "loading" | "ready" | "error";

export type ServerAuthorFormatters = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  formatTimestamp: (value: string) => string;
  shortIdentity: (identityId: string) => string;
};

export type ServerChatPrefs = {
  alignment: MessageAlignment;
  bubbleSize: MessageBubbleSize;
  layout: MessageLayout;
};

export type ServerChatData = {
  activeChannel: ServerChannelSummary | null;
  channelNotificationCounts: Map<string, number>;
  channels: ServerChannelSummary[];
  messageById: Map<string, ServerChannelMessage>;
  messages: ServerChannelMessage[];
  nextCursor: string | null;
};

export type ServerChatState = {
  composer: string;
  hasSession: boolean;
  identityId: string;
  mentionIdentityIds: string[];
  messageState: ServerWorkspaceLoadState;
  olderState: ServerWorkspaceLoadState;
  replyTo: ServerChannelMessage | null;
  sendBusy: boolean;
};
