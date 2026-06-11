"use client";

import { IconPinned, IconPinnedOff, IconTrash, IconVolume, IconVolumeOff } from "@tabler/icons-react";

import { Badge } from "@/components/ui/display/badge";
import { Button } from "@/components/ui/buttons/button";
import { Toolbar } from "@/components/ui/surfaces/toolbar";

export function BulkActions({
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
      <Button disabled={disabled} icon={<IconPinned aria-hidden="true" />} onClick={onPin}>
        Pin
      </Button>
      <Button disabled={disabled} icon={<IconPinnedOff aria-hidden="true" />} onClick={onUnpin}>
        Unpin
      </Button>
      <Button disabled={disabled} icon={<IconVolumeOff aria-hidden="true" />} onClick={onMute}>
        Mute
      </Button>
      <Button disabled={disabled} icon={<IconVolume aria-hidden="true" />} onClick={onUnmute}>
        Unmute
      </Button>
      <Button disabled={disabled} icon={<IconTrash aria-hidden="true" />} onClick={onDestructive} variant="danger">
        {destructiveLabel}
      </Button>
      <Button disabled={busy} onClick={onDone}>
        Done
      </Button>
    </Toolbar>
  );
}
