import { IconHash } from "@tabler/icons-react";

import type { ServerChannelSummary } from "@/lib/api";

import styles from "@/app/surfaces.module.css";

type ServerChannelButtonProps = {
  active: boolean;
  channel: ServerChannelSummary;
  notificationCount: number;
  onSelect: (channelId: string) => void;
};

export function ServerChannelButton({ active, channel, notificationCount, onSelect }: ServerChannelButtonProps) {
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
