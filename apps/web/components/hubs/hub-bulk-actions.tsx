"use client";

import { IconPinned, IconPinnedOff, IconTrash, IconVolume, IconVolumeOff } from "@tabler/icons-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Toolbar } from "@/components/ui/toolbar";

import styles from "./hubs.module.css";

export function HubBulkActions({
  busy,
  destructiveLabel,
  onDestructive,
  onDone,
  onMute,
  onPin,
  onUnmute,
  onUnpin,
  selectedCount,
}: {
  busy: boolean;
  destructiveLabel: string;
  onDestructive: () => void;
  onDone: () => void;
  onMute: () => void;
  onPin: () => void;
  onUnmute: () => void;
  onUnpin: () => void;
  selectedCount: number;
}) {
  const disabled = busy || selectedCount === 0;

  return (
    <Toolbar>
      <Badge tone="accent">{selectedCount} selected</Badge>
      <Button disabled={disabled} icon={<IconPinned className={styles.icon} aria-hidden="true" />} onClick={onPin}>
        Pin
      </Button>
      <Button disabled={disabled} icon={<IconPinnedOff className={styles.icon} aria-hidden="true" />} onClick={onUnpin}>
        Unpin
      </Button>
      <Button disabled={disabled} icon={<IconVolumeOff className={styles.icon} aria-hidden="true" />} onClick={onMute}>
        Mute
      </Button>
      <Button disabled={disabled} icon={<IconVolume className={styles.icon} aria-hidden="true" />} onClick={onUnmute}>
        Unmute
      </Button>
      <Button disabled={disabled} icon={<IconTrash className={styles.icon} aria-hidden="true" />} onClick={onDestructive} variant="danger">
        {destructiveLabel}
      </Button>
      <Button disabled={busy} onClick={onDone}>
        Done
      </Button>
    </Toolbar>
  );
}
