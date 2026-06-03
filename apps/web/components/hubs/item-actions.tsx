import { IconClock, IconMessageCircle, IconPinned, IconPinnedOff, IconTrash, IconVolume, IconVolumeOff } from "@tabler/icons-react";

import { Button } from "@/components/ui/button";

import styles from "./styles.module.css";

type ItemActionsProps = {
  busy: boolean;
  destructiveLabel: string;
  messageAction?: {
    disabled?: boolean;
    label: string;
    onClick: () => void;
  };
  muted: boolean;
  onDestructive: () => void;
  onToggleMuted: () => void;
  onTogglePinned: () => void;
  pendingLabel?: string;
  pinned: boolean;
};

export function ItemActions({
  busy,
  destructiveLabel,
  messageAction,
  muted,
  onDestructive,
  onToggleMuted,
  onTogglePinned,
  pendingLabel,
  pinned,
}: ItemActionsProps) {
  return (
    <>
      {pendingLabel ? (
        <Button disabled icon={<IconClock className={styles.icon} aria-hidden="true" />} size="sm">
          {pendingLabel}
        </Button>
      ) : messageAction ? (
        <Button
          disabled={messageAction.disabled}
          icon={<IconMessageCircle className={styles.icon} aria-hidden="true" />}
          onClick={messageAction.onClick}
          size="sm"
        >
          {messageAction.label}
        </Button>
      ) : null}
      <Button
        disabled={busy}
        icon={
          pinned ? (
            <IconPinnedOff className={styles.icon} aria-hidden="true" />
          ) : (
            <IconPinned className={styles.icon} aria-hidden="true" />
          )
        }
        onClick={onTogglePinned}
        size="sm"
      >
        {pinned ? "Unpin" : "Pin"}
      </Button>
      <Button
        disabled={busy}
        icon={
          muted ? (
            <IconVolume className={styles.icon} aria-hidden="true" />
          ) : (
            <IconVolumeOff className={styles.icon} aria-hidden="true" />
          )
        }
        onClick={onToggleMuted}
        size="sm"
      >
        {muted ? "Unmute" : "Mute"}
      </Button>
      <Button
        disabled={busy}
        icon={<IconTrash className={styles.icon} aria-hidden="true" />}
        onClick={onDestructive}
        size="sm"
        variant="danger"
      >
        {destructiveLabel}
      </Button>
    </>
  );
}
