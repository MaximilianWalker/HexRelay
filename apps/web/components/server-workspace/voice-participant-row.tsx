import { IconMicrophone } from "@tabler/icons-react";

import { Badge } from "@/components/ui/badge";

import styles from "@/app/surfaces.module.css";
import { initials } from "@/lib/ui/initials";

type VoiceParticipantRowProps = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  identityId: string;
  speaking: boolean;
};

export function VoiceParticipantRow({
  authorHandle,
  authorLabel,
  identityId,
  speaking,
}: VoiceParticipantRowProps) {
  const name = authorLabel(identityId);

  return (
    <article className={`${styles.voiceParticipant} ${speaking ? styles.voiceParticipantSpeaking : ""}`}>
      <div className={styles.memberAvatar}>{initials(name)}</div>
      <div>
        <h4>{name}</h4>
        <p>@{authorHandle(identityId)}</p>
      </div>
      <Badge icon={<IconMicrophone aria-hidden="true" />} size="sm" tone={speaking ? "accent" : "neutral"}>
        {speaking ? "Speaking" : "Connected"}
      </Badge>
    </article>
  );
}
