import { IconHash, IconInfoCircle, IconShieldCheck, IconUsers, IconVolume } from "@tabler/icons-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { ServerChannelSummary, ServerSummary } from "@/lib/api";

import { Icon } from "./icon";

import styles from "@/app/surfaces.module.css";

type SettingsViewProps = {
  canManageServer: boolean;
  channels: ServerChannelSummary[];
  hasSession: boolean;
  memberCount: number;
  server: ServerSummary;
  voiceChannelCount: number;
};

export function SettingsView({
  canManageServer,
  channels,
  hasSession,
  memberCount,
  server,
  voiceChannelCount,
}: SettingsViewProps) {
  if (!canManageServer) {
    return (
      <section className={styles.settingsView} aria-label="Server settings unavailable">
        <article className={styles.overviewPanel}>
          <div className={styles.panelHeader}>
            <h3>Settings unavailable</h3>
          </div>
          <p className={styles.meta}>Server settings are only visible to server admins.</p>
        </article>
      </section>
    );
  }

  return (
    <section className={styles.settingsView} aria-label="Server settings">
      <header className={styles.usersHeader}>
        <div>
          <p className={styles.serverSectionLabel}>Admin</p>
          <h2>Server settings</h2>
          <p className={styles.serverMeta}>
            Preview-only controls for server identity, member access, channel policy, and destructive actions.
          </p>
        </div>
        <div className={styles.usersStats} aria-label="Settings summary">
          <Badge icon={<IconShieldCheck aria-hidden="true" />} size="sm" tone="success">
            Admin visible
          </Badge>
          <Badge icon={<IconUsers aria-hidden="true" />} size="sm">
            {memberCount} members
          </Badge>
          <Badge icon={<IconHash aria-hidden="true" />} size="sm">
            {channels.length} text
          </Badge>
          <Badge icon={<IconVolume aria-hidden="true" />} size="sm">
            {voiceChannelCount} voice
          </Badge>
        </div>
      </header>

      {!hasSession ? (
        <div className={styles.serverAlert}>
          <IconInfoCircle className={styles.icon} aria-hidden="true" />
          <span>Showing seeded admin settings. Live changes are disabled until server admin APIs are available.</span>
        </div>
      ) : null}

      <div className={styles.settingsGrid}>
        <article className={`${styles.overviewPanel} ${styles.settingsPanelWide}`}>
          <div className={styles.panelHeader}>
            <h3>Server identity</h3>
            <span>Preview</span>
          </div>
          <div className={styles.settingsIdentity}>
            <Icon name={server.name} />
            <div>
              <h4>{server.name}</h4>
              <p>Server image, display name, and markdown overview source.</p>
            </div>
          </div>
          <div className={styles.settingsFieldGrid}>
            <div className={styles.settingsField}>
              <span>Server name</span>
              <strong>{server.name}</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Overview markdown</span>
              <strong>server.md</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Default tab</span>
              <strong>Overview</strong>
            </div>
          </div>
          <div className={styles.settingsActions}>
            <Button disabled variant="primary">
              Save changes
            </Button>
            <Button disabled>
              Upload image
            </Button>
          </div>
        </article>

        <article className={styles.overviewPanel}>
          <div className={styles.panelHeader}>
            <h3>Access</h3>
            <span>Roles</span>
          </div>
          <div className={styles.settingsFieldGrid}>
            <div className={styles.settingsField}>
              <span>Owner</span>
              <strong>Alice</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Settings tab</span>
              <strong>Admins only</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Invite scope</span>
              <strong>Join eligibility</strong>
            </div>
          </div>
        </article>

        <article className={styles.overviewPanel}>
          <div className={styles.panelHeader}>
            <h3>Channels</h3>
            <span>Policy</span>
          </div>
          <div className={styles.settingsFieldGrid}>
            <div className={styles.settingsField}>
              <span>Text channels</span>
              <strong>{channels.length}</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Voice channels</span>
              <strong>{voiceChannelCount}</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Unread markers</span>
              <strong>Mentions only</strong>
            </div>
          </div>
        </article>

        <article className={styles.overviewPanel}>
          <div className={styles.panelHeader}>
            <h3>Moderation</h3>
            <span>Seeded</span>
          </div>
          <div className={styles.settingsFieldGrid}>
            <div className={styles.settingsField}>
              <span>Message edits</span>
              <strong>Deferred</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Message deletes</span>
              <strong>Deferred</strong>
            </div>
            <div className={styles.settingsField}>
              <span>Audit log</span>
              <strong>Future API</strong>
            </div>
          </div>
        </article>

        <article className={`${styles.overviewPanel} ${styles.dangerPanel}`}>
          <div className={styles.panelHeader}>
            <h3>Danger zone</h3>
            <span>Locked</span>
          </div>
          <p className={styles.meta}>
            Transfer and delete actions stay disabled in the validation page until real admin mutations and
            confirmations exist.
          </p>
          <div className={styles.settingsActions}>
            <Button disabled>
              Transfer ownership
            </Button>
            <Button disabled>
              Delete server
            </Button>
          </div>
        </article>
      </div>
    </section>
  );
}
