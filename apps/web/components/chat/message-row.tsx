import { IconMessageCircle } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { IconButton } from "@/components/ui/icon-button";
import type { ServerChannelMessage } from "@/lib/api";
import type { MessageLayout } from "@/lib/workspace-preferences";
import { cx } from "@/lib/ui/cx";

import styles from "./chat.module.css";

export function MessageRow({
  authorHandle,
  authorLabel,
  formatTimestamp,
  message,
  onReply,
  ownMessage,
  replyTo,
  shortIdentity,
  layout,
}: {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  formatTimestamp: (value: string) => string;
  layout: MessageLayout;
  message: ServerChannelMessage;
  onReply: (message: ServerChannelMessage) => void;
  ownMessage: boolean;
  replyTo?: ServerChannelMessage;
  shortIdentity: (identityId: string) => string;
}) {
  const deleted = Boolean(message.deleted_at);
  const continuous = layout === "continuous-feed";

  return (
    <article
      className={cx(styles.messageRow, ownMessage && !continuous && styles.messageOwn, continuous && styles.messageContinuous)}
    >
      <Avatar kind="user" size="sm" text={initials(authorLabel(message.author_id))} />
      <div className={styles.messageBody}>
        <div className={styles.messageHeader}>
          <span className={styles.messageAuthor}>{authorLabel(message.author_id)}</span>
          <span className={styles.messageTime}>{formatTimestamp(message.created_at)}</span>
          {message.edited_at ? <Badge tone="muted">edited</Badge> : null}
          {deleted ? <Badge tone="muted">deleted</Badge> : null}
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
          <div className={styles.mentions} aria-label="Mentions">
            {message.mentions.map((mention) => (
              <Badge tone="accent" key={mention}>
                @{authorHandle(mention)}
              </Badge>
            ))}
          </div>
        ) : null}
      </div>
      {!deleted ? (
        <IconButton label={`Reply to message from ${authorLabel(message.author_id)}`} onClick={() => onReply(message)}>
          <IconMessageCircle className={styles.icon} aria-hidden="true" />
        </IconButton>
      ) : null}
    </article>
  );
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}
