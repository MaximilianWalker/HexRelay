import { IconMicrophone } from "@tabler/icons-react";

import styles from "@/app/surfaces.module.css";

type ServerVoiceParticipantRowProps = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  identityId: string;
  speaking: boolean;
};

function initials(value: string): string {
  const parts = value.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export function ServerVoiceParticipantRow({
  authorHandle,
  authorLabel,
  identityId,
  speaking,
}: ServerVoiceParticipantRowProps) {
  const name = authorLabel(identityId);

  return (
    <article className={`${styles.voiceParticipant} ${speaking ? styles.voiceParticipantSpeaking : ""}`}>
      <div className={styles.memberAvatar}>{initials(name)}</div>
      <div>
        <h4>{name}</h4>
        <p>@{authorHandle(identityId)}</p>
      </div>
      <span>
        <IconMicrophone className={styles.icon} aria-hidden="true" />
        {speaking ? "Speaking" : "Connected"}
      </span>
    </article>
  );
}
