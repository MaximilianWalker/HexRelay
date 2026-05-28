import { Avatar } from "@/components/ui/avatar";
import { cx } from "@/lib/ui/cx";

import type { WorkspaceProfile } from "./workspace-profile-types";
import styles from "./workspace-profile-card.module.css";

type WorkspaceProfileCardProps = {
  collapsed: boolean;
  profile: WorkspaceProfile;
};

export function WorkspaceProfileCard({ collapsed, profile }: WorkspaceProfileCardProps) {
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
