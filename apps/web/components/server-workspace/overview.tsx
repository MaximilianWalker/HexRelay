import { IconDotsVertical, IconInfoCircle, IconLogout } from "@tabler/icons-react";

import { IconButton } from "@/components/ui/icon-button";
import { Menu, MenuItem } from "@/components/ui/menu";
import type { ServerSummary } from "@/lib/api";

import { Icon } from "./icon";

import styles from "@/app/surfaces.module.css";

type OverviewProps = {
  hasSession: boolean;
  menuOpen: boolean;
  onMenuAction: (label: string) => void;
  onToggleMenu: () => void;
  rules: string[];
  server: ServerSummary;
  tags: string[];
};

export function Overview({
  hasSession,
  menuOpen,
  onMenuAction,
  onToggleMenu,
  rules,
  server,
  tags,
}: OverviewProps) {
  return (
    <section className={styles.overviewStack} aria-label="Server overview">
      <header className={styles.serverHero}>
        <Icon name={server.name} />
        <div className={styles.serverHeroText}>
          <h2 className={styles.serverTitle}>{server.name}</h2>
          <div className={styles.serverTagRow} aria-label="Server tags">
            {tags.map((tag) => (
              <span className={styles.serverTag} key={tag}>
                {tag}
              </span>
            ))}
            {server.unread > 0 ? <span className={styles.serverTag}>{server.unread} unread</span> : null}
          </div>
        </div>
        <div className={styles.serverMenu}>
          <IconButton
            aria-expanded={menuOpen}
            label="Server actions"
            onClick={onToggleMenu}
            title="Server actions"
          >
            <IconDotsVertical className={styles.icon} aria-hidden="true" />
          </IconButton>
          {menuOpen ? (
            <Menu className={styles.serverMenuList} position="absolute">
              <MenuItem onClick={() => onMenuAction("Mark server as read")}>
                Mark as read
              </MenuItem>
              <MenuItem onClick={() => onMenuAction("Mute notifications")}>
                Mute notifications
              </MenuItem>
              <MenuItem
                icon={<IconLogout className={styles.icon} aria-hidden="true" />}
                onClick={() => onMenuAction("Leave server")}
                tone="danger"
              >
                Leave server
              </MenuItem>
            </Menu>
          ) : null}
        </div>
      </header>

      {!hasSession ? (
        <div className={styles.serverAlert}>
          <IconInfoCircle className={styles.icon} aria-hidden="true" />
          <span>Activate a local testing profile to load live server data. Showing seeded Atlas preview data.</span>
        </div>
      ) : null}

      <div className={styles.overviewFeatureGrid}>
        <article className={`${styles.overviewPanel} ${styles.markdownPanel}`}>
          <div className={styles.panelHeader}>
            <h3>About this server</h3>
            <span>server.md</span>
          </div>
          <div className={styles.markdownPreview}>
            <h4>Atlas validation space</h4>
            <p>
              Atlas is the seeded server for reviewing shared channels, mentions, replies, roles, and voice workspace
              behavior before the live server surface is widened.
            </p>
            <ul>
              <li>
                <strong>#general</strong> keeps the default conversation and mention checks.
              </li>
              <li>
                <strong>#ops-lab</strong> tracks workspace, moderation, and voice follow-up notes.
              </li>
            </ul>
            <blockquote>Today: validate the server workspace layout and tab model.</blockquote>
          </div>
        </article>

        <div className={styles.overviewRail}>
          <article className={styles.overviewPanel}>
            <div className={styles.panelHeader}>
              <h3>Pinned announcement</h3>
            </div>
            <p className={styles.meta}>
              Review the server tabs first, then keep feedback in the relevant surface: Chat for text channels, Voice
              for voice rooms, and Settings for admin controls.
            </p>
          </article>

          <article className={styles.overviewPanel}>
            <div className={styles.panelHeader}>
              <h3>Rules</h3>
              <span>{rules.length}</span>
            </div>
            <ul className={styles.overviewList}>
              {rules.map((rule) => (
                <li key={rule}>{rule}</li>
              ))}
            </ul>
          </article>
        </div>
      </div>
    </section>
  );
}
