import { IconVolume } from "@tabler/icons-react";

import { ListActionButton } from "@/components/ui/list-action-button";

import type { VoiceChannel } from "./types";

type VoiceChannelButtonProps = {
  active: boolean;
  channel: VoiceChannel;
  onSelect: (channelId: string) => void;
};

export function VoiceChannelButton({ active, channel, onSelect }: VoiceChannelButtonProps) {
  const connectedCount = channel.participantIds.length;

  return (
    <ListActionButton
      active={active}
      badge={connectedCount > 0 ? connectedCount : undefined}
      badgeLabel={connectedCount > 0 ? `${connectedCount} connected users` : undefined}
      icon={<IconVolume aria-hidden="true" />}
      onClick={() => onSelect(channel.id)}
    >
      {channel.name}
    </ListActionButton>
  );
}
