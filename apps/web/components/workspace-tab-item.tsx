import Link from "next/link";
import type { DragEvent, KeyboardEvent, MouseEvent } from "react";
import { IconX } from "@tabler/icons-react";

import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cx } from "@/lib/ui/cx";
import { initials } from "@/lib/ui/initials";
import type { WorkspaceTab } from "@/lib/workspace-tabs";

import styles from "./workspace-tab-item.module.css";

type WorkspaceTabItemProps = {
  active: boolean;
  dragging: boolean;
  onClose: (tab: WorkspaceTab) => void;
  onContextMenu: (event: MouseEvent<HTMLElement>, tab: WorkspaceTab) => void;
  onDragEnd: () => void;
  onDragStart: (tab: WorkspaceTab, event: DragEvent<HTMLElement>) => void;
  onDrop: (tab: WorkspaceTab, event: DragEvent<HTMLElement>) => void;
  onKeyboardContextMenu: (event: KeyboardEvent<HTMLElement>, tab: WorkspaceTab) => void;
  tab: WorkspaceTab;
};

function unreadCount(value: number | undefined): number {
  if (!Number.isFinite(value) || !value || value <= 0) {
    return 0;
  }

  return Math.floor(value);
}

export function WorkspaceTabItem({
  active,
  dragging,
  onClose,
  onContextMenu,
  onDragEnd,
  onDragStart,
  onDrop,
  onKeyboardContextMenu,
  tab,
}: WorkspaceTabItemProps) {
  const imageLabel = tab.imageLabel ?? tab.label;
  const isServer = tab.kind === "server";
  const unread = unreadCount(tab.unread);

  return (
    <div
      className={cx(styles.tab, active && styles.active, tab.pinned && styles.pinned)}
      data-workspace-tab-id={tab.id}
      draggable
      onContextMenu={(event) => onContextMenu(event, tab)}
      onDragEnd={onDragEnd}
      onDragOver={(event) => {
        if (dragging) {
          event.preventDefault();
        }
      }}
      onDragStart={(event) => onDragStart(tab, event)}
      onDrop={(event) => {
        event.preventDefault();
        onDrop(tab, event);
      }}
      role="listitem"
    >
      <Link
        aria-current={active ? "page" : undefined}
        aria-label={`${tab.kind === "dm" ? "Conversation" : "Server"}: ${tab.label}`}
        className={styles.link}
        href={tab.href}
        onKeyDown={(event) => onKeyboardContextMenu(event, tab)}
      >
        <Avatar
          className={cx(styles.image, isServer ? styles.imageServer : styles.imageContact)}
          aria-hidden="true"
          kind={isServer ? "server" : "user"}
          text={initials(imageLabel)}
        />
        <span className={styles.label}>{tab.label}</span>
      </Link>
      <div className={styles.actions}>
        {isServer && unread > 0 ? (
          <Badge className={styles.badge} aria-label={`${unread} unread notifications`} tone="accent">
            {unread}
          </Badge>
        ) : null}
        {!tab.pinned ? (
          <Button
            aria-label={`Close ${tab.label}`}
            className={styles.closeButton}
            onClick={() => onClose(tab)}
            size="icon"
            title="Close tab"
          >
            <IconX className={styles.icon} aria-hidden="true" />
          </Button>
        ) : null}
      </div>
    </div>
  );
}
