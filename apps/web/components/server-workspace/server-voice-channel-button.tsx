import { IconVolume } from "@tabler/icons-react";

import type { ServerVoiceChannel } from "./server-workspace-types";

import styles from "@/app/surfaces.module.css";

type ServerVoiceChannelButtonProps = {
  active: boolean;
  channel: ServerVoiceChannel;
  onSelect: (channelId: string) => void;
};

export function ServerVoiceChannelButton({ active, channel, onSelect }: ServerVoiceChannelButtonProps) {
  const connectedCount = channel.participantIds.length;

  return (
    <button
      aria-pressed={active}
      className={`${styles.channelButton} ${active ? styles.channelButtonActive : ""}`}
      onClick={() => onSelect(channel.id)}
      type="button"
    >
      <IconVolume className={styles.icon} aria-hidden="true" />
      <span>{channel.name}</span>
      {connectedCount > 0 ? (
        <span className={styles.channelBadge} aria-label={`${connectedCount} connected users`}>
          {connectedCount}
        </span>
      ) : null}
    </button>
  );
}
