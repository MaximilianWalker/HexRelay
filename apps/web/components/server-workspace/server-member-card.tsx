import { IconBell, IconBellOff, IconClock, IconStar } from "@tabler/icons-react";

import type { ServerMember } from "./server-workspace-types";

import styles from "@/app/surfaces.module.css";
import { initials } from "@/lib/ui/initials";

type ServerMemberCardProps = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  current: boolean;
  formatTimestamp: (value: string) => string;
  member: ServerMember;
};

export function ServerMemberCard({
  authorHandle,
  authorLabel,
  current,
  formatTimestamp,
  member,
}: ServerMemberCardProps) {
  const name = authorLabel(member.identityId);
  const presenceLabel = member.presence === "online" ? "Online" : "Away";

  return (
    <article className={`${styles.memberCard} ${current ? styles.memberCardCurrent : ""}`}>
      <div className={styles.memberAvatarWrap}>
        <div className={styles.memberAvatar}>{initials(name)}</div>
        <span
          aria-label={presenceLabel}
          className={`${styles.presenceDot} ${
            member.presence === "online" ? styles.presenceOnline : styles.presenceAway
          }`}
          role="img"
        />
      </div>
      <div className={styles.memberInfo}>
        <div className={styles.memberNameRow}>
          <h4>{name}</h4>
          {current ? <span className={styles.memberBadge}>You</span> : null}
        </div>
        <p>@{authorHandle(member.identityId)}</p>
        <p>{member.title}</p>
        <span>{member.lastActive}</span>
      </div>
      <div className={styles.memberMetaStack}>
        <span>
          <IconClock className={styles.icon} aria-hidden="true" />
          Joined {formatTimestamp(member.joinedAt)}
        </span>
        <span>
          {member.muted ? (
            <IconBellOff className={styles.icon} aria-hidden="true" />
          ) : (
            <IconBell className={styles.icon} aria-hidden="true" />
          )}
          {member.muted ? "Muted" : "Audible"}
        </span>
        {member.pinned ? (
          <span>
            <IconStar className={styles.icon} aria-hidden="true" />
            Pinned
          </span>
        ) : null}
        {member.unread > 0 ? <strong>{member.unread}</strong> : null}
      </div>
    </article>
  );
}
