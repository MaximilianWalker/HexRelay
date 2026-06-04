import { IconBell, IconBellOff, IconClock, IconStar } from "@tabler/icons-react";

import { Badge } from "@/components/ui/badge";

import type { Member } from "./types";

import styles from "@/app/surfaces.module.css";
import { initials } from "@/lib/ui/initials";

type MemberCardProps = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  current: boolean;
  formatTimestamp: (value: string) => string;
  member: Member;
};

export function MemberCard({
  authorHandle,
  authorLabel,
  current,
  formatTimestamp,
  member,
}: MemberCardProps) {
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
          {current ? (
            <Badge size="sm" tone="accent">
              You
            </Badge>
          ) : null}
        </div>
        <p>@{authorHandle(member.identityId)}</p>
        <p>{member.title}</p>
        <span>{member.lastActive}</span>
      </div>
      <div className={styles.memberMetaStack}>
        <Badge icon={<IconClock aria-hidden="true" />} size="sm">
          Joined {formatTimestamp(member.joinedAt)}
        </Badge>
        <Badge icon={member.muted ? <IconBellOff aria-hidden="true" /> : <IconBell aria-hidden="true" />} size="sm">
          {member.muted ? "Muted" : "Audible"}
        </Badge>
        {member.pinned ? (
          <Badge icon={<IconStar aria-hidden="true" />} size="sm" tone="accent">
            Pinned
          </Badge>
        ) : null}
        {member.unread > 0 ? (
          <Badge aria-label={`${member.unread} unread notifications`} size="sm" tone="accent">
            {member.unread}
          </Badge>
        ) : null}
      </div>
    </article>
  );
}
