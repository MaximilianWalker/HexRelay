import { Avatar } from "@/components/ui/avatar";
import { cx } from "@/lib/ui/cx";

import type { Profile } from "./types";
import styles from "./card.module.css";

type CardProps = {
  collapsed: boolean;
  profile: Profile;
};

export function Card({ collapsed, profile }: CardProps) {
  return (
    <div className={styles.profile} title={profile.name}>
      <div className={styles.avatarFrame}>
        <Avatar className={styles.avatar} kind="user" size="sm" text={profile.initials} />
        <span className={cx(styles.presence, profile.active && styles.presenceActive)} aria-hidden="true" />
      </div>
      {!collapsed ? (
        <div className={styles.details}>
          <p className={styles.eyebrow}>{profile.active ? "Active profile" : "Profile setup"}</p>
          <p className={styles.name}>{profile.name}</p>
          <p className={styles.status}>
            <span className={cx(styles.statusDot, profile.active && styles.statusDotActive)} aria-hidden="true" />
            {profile.status}
          </p>
        </div>
      ) : null}
    </div>
  );
}
