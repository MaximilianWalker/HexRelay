import { IconCircleCheck, IconInfoCircle, IconShieldCheck, IconUsers } from "@tabler/icons-react";

import { MemberCard } from "./member-card";
import type { RoleGroup } from "./types";

import styles from "@/app/surfaces.module.css";

type UsersViewProps = {
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  formatTimestamp: (value: string) => string;
  hasSession: boolean;
  identityId: string;
  roleGroups: RoleGroup[];
};

export function UsersView({
  authorHandle,
  authorLabel,
  formatTimestamp,
  hasSession,
  identityId,
  roleGroups,
}: UsersViewProps) {
  const members = roleGroups.flatMap((group) => group.members);
  const onlineCount = members.filter((member) => member.presence === "online").length;

  return (
    <section className={styles.usersView} aria-label="Server users">
      <header className={styles.usersHeader}>
        <div>
          <p className={styles.serverSectionLabel}>Members</p>
          <h2>Server users</h2>
          <p className={styles.serverMeta}>
            Seeded server-chat memberships grouped by role, with profile, presence, and per-member server state.
          </p>
        </div>
        <div className={styles.usersStats} aria-label="Member summary">
          <span>
            <IconUsers className={styles.icon} aria-hidden="true" />
            {members.length} members
          </span>
          <span>
            <IconShieldCheck className={styles.icon} aria-hidden="true" />
            {roleGroups.length} roles
          </span>
          <span>
            <IconCircleCheck className={styles.icon} aria-hidden="true" />
            {onlineCount} online
          </span>
        </div>
      </header>

      {!hasSession ? (
        <div className={styles.serverNotice}>
          <IconInfoCircle className={styles.icon} aria-hidden="true" />
          <span>Showing seeded Atlas membership data until a local testing profile loads live server state.</span>
        </div>
      ) : null}

      <div className={styles.roleGroups}>
        {roleGroups.map((group) => (
          <section className={styles.roleGroup} key={group.label} aria-label={`${group.label} members`}>
            <div className={styles.roleGroupHeader}>
              <div>
                <h3>{group.label}</h3>
                <p>{group.description}</p>
              </div>
              <span>{group.members.length}</span>
            </div>
            <div className={styles.memberList}>
              {group.members.map((member) => (
                <MemberCard
                  authorHandle={authorHandle}
                  authorLabel={authorLabel}
                  current={member.identityId === identityId}
                  formatTimestamp={formatTimestamp}
                  key={member.identityId}
                  member={member}
                />
              ))}
            </div>
          </section>
        ))}
      </div>
    </section>
  );
}
