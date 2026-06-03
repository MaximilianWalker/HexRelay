import type { ServerChannelMessage, ServerChannelSummary } from "@/lib/api";
import type { MessageAlignment, MessageBubbleSize, MessageLayout } from "@/lib/workspace-preferences";

export type Member = {
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

export type RoleGroup = {
  description: string;
  label: string;
  members: Member[];
};

export type VoiceChannel = {
  description: string;
  id: string;
  name: string;
  participantIds: string[];
  speakerId: string | null;
};

export type LoadState = "idle" | "loading" | "ready" | "error";

export type AuthorFormatters = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  formatTimestamp: (value: string) => string;
  shortIdentity: (identityId: string) => string;
};

export type ChatPrefs = {
  alignment: MessageAlignment;
  bubbleSize: MessageBubbleSize;
  layout: MessageLayout;
};

export type ChatData = {
  activeChannel: ServerChannelSummary | null;
  channelNotificationCounts: Map<string, number>;
  channels: ServerChannelSummary[];
  messageById: Map<string, ServerChannelMessage>;
  messages: ServerChannelMessage[];
  nextCursor: string | null;
};

export type ChatState = {
  composer: string;
  hasSession: boolean;
  identityId: string;
  mentionIdentityIds: string[];
  messageState: LoadState;
  olderState: LoadState;
  replyTo: ServerChannelMessage | null;
  sendBusy: boolean;
};
