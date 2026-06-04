import { IconHash, IconInfoCircle, IconRefresh } from "@tabler/icons-react";

import { ChannelRail } from "@/components/chat/channel-rail";
import { Composer } from "@/components/chat/composer";
import { MessageRow } from "@/components/chat/message-row";
import { MessageTimeline } from "@/components/chat/message-timeline";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { ServerChannelMessage } from "@/lib/api";

import { ChannelButton } from "./channel-button";
import type { AuthorFormatters, ChatData, ChatPrefs, ChatState } from "./types";

import styles from "@/app/surfaces.module.css";

type ChatViewProps = {
  data: ChatData;
  formatters: AuthorFormatters;
  onCancelReply: () => void;
  onChangeComposer: (value: string) => void;
  onLoadOlder: () => void;
  onRefresh: () => void;
  onReply: (message: ServerChannelMessage) => void;
  onSelectChannel: (channelId: string) => void;
  onSend: () => void;
  prefs: ChatPrefs;
  state: ChatState;
};

export function ChatView({
  data,
  formatters,
  onCancelReply,
  onChangeComposer,
  onLoadOlder,
  onRefresh,
  onReply,
  onSelectChannel,
  onSend,
  prefs,
  state,
}: ChatViewProps) {
  const { activeChannel, channelNotificationCounts, channels, messageById, messages, nextCursor } = data;
  const { authorHandle, authorLabel, formatTimestamp, shortIdentity } = formatters;
  const { alignment, bubbleSize, layout } = prefs;
  const { composer, hasSession, identityId, mentionIdentityIds, messageState, olderState, replyTo, sendBusy } = state;
  const loadOlderLabel =
    nextCursor && hasSession ? (olderState === "loading" ? "Loading older..." : "Load older messages") : null;

  return (
    <section className={styles.chatGrid} aria-label="Server chat">
      <ChannelRail aria-label="Channels" title="Channels">
        {channels.map((channel) => (
          <ChannelButton
            active={channel.id === activeChannel?.id}
            channel={channel}
            key={channel.id}
            notificationCount={channelNotificationCounts.get(channel.id) ?? 0}
            onSelect={onSelectChannel}
          />
        ))}
      </ChannelRail>

      <article className={styles.chatPanel}>
        <header className={styles.chatHeader}>
          <div>
            <p className={styles.serverSectionLabel}>Channel</p>
            <h3>
              <IconHash className={styles.icon} aria-hidden="true" />
              {activeChannel?.name ?? "No channel"}
            </h3>
            <p className={styles.serverMeta}>{activeChannel ? `${activeChannel.kind} channel` : ""}</p>
          </div>
          <Button
            disabled={!hasSession || !activeChannel || messageState === "loading"}
            icon={<IconRefresh aria-hidden="true" />}
            onClick={onRefresh}
          >
            Refresh
          </Button>
        </header>

        {messageState === "loading" ? <p className={styles.state}>Loading channel history...</p> : null}
        {messageState === "error" ? <p className={styles.state}>Could not load channel history.</p> : null}

        <MessageTimeline
          bubbleSize={bubbleSize}
          layout={layout}
          loadOlderLabel={loadOlderLabel}
          loadingOlder={olderState === "loading"}
          onLoadOlder={onLoadOlder}
        >
          {messages.length > 0 ? (
            messages.map((message) => (
              <MessageRow
                alignment={alignment}
                authorHandle={authorHandle}
                authorLabel={authorLabel}
                bubbleSize={bubbleSize}
                formatTimestamp={formatTimestamp}
                key={message.message_id}
                layout={layout}
                message={message}
                onReply={onReply}
                ownMessage={message.author_id === identityId}
                replyTo={message.reply_to_message_id ? messageById.get(message.reply_to_message_id) : undefined}
                shortIdentity={shortIdentity}
              />
            ))
          ) : (
            <p className={styles.state}>No messages in this channel yet.</p>
          )}
        </MessageTimeline>

        {hasSession ? (
          <Composer
            disabled={!activeChannel || sendBusy}
            hints={
              mentionIdentityIds.length > 0 ? (
                mentionIdentityIds.map((mention) => (
                  <Badge key={mention} size="sm" tone="accent">
                    @{authorHandle(mention)}
                  </Badge>
                ))
              ) : (
                <span className={styles.meta}>Use @alice, @bob, or @carol with the seeded fixture.</span>
              )
            }
            onCancelReply={replyTo ? onCancelReply : undefined}
            onChange={onChangeComposer}
            onSend={onSend}
            placeholder={activeChannel ? `Message #${activeChannel.name}` : "Select a channel"}
            replyLabel={replyTo ? `Replying to ${authorLabel(replyTo.author_id)}` : undefined}
            replyText={replyTo?.content}
            sendLabel={sendBusy ? "Sending..." : "Send"}
            value={composer}
          />
        ) : (
          <div className={styles.composerLocked}>
            <IconInfoCircle className={styles.icon} aria-hidden="true" />
            Activate a local testing profile to send messages.
          </div>
        )}
      </article>
    </section>
  );
}
