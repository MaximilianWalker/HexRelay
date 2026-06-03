import { IconHash } from "@tabler/icons-react";

import { ListActionButton } from "@/components/ui/list-action-button";
import type { ServerChannelSummary } from "@/lib/api";

type ChannelButtonProps = {
  active: boolean;
  channel: ServerChannelSummary;
  notificationCount: number;
  onSelect: (channelId: string) => void;
};

export function ChannelButton({ active, channel, notificationCount, onSelect }: ChannelButtonProps) {
  return (
    <ListActionButton
      active={active}
      badge={notificationCount > 0 ? notificationCount : undefined}
      badgeLabel={notificationCount > 0 ? `${notificationCount} unseen mentions` : undefined}
      icon={<IconHash aria-hidden="true" />}
      onClick={() => onSelect(channel.id)}
    >
      {channel.name}
    </ListActionButton>
  );
}
